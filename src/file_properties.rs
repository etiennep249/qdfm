use slint::{ComponentHandle, Model, SharedString, VecModel, Weak};
use std::{
    cmp::Ordering,
    fs::{set_permissions, Metadata, Permissions},
    io::{Error, ErrorKind},
    os::unix::fs::{lchown, MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    rc::Rc,
    sync::mpsc::channel,
    thread::sleep,
    time::{Duration, UNIX_EPOCH},
};
use walkdir::WalkDir;

use crate::{
    callbacks::{filemanager::set_current_tab_file, properties::set_split_bitmask},
    core::{
        get_all_groups, get_all_users, get_file_encoding, get_file_magic_type, get_file_metadata,
        get_gid, get_uid, Group,
    },
    ui::*,
    utils::{
        error_handling::{log_error, log_error_str},
        types::{format_date, i32_to_i64, i64_to_i32},
    },
};

pub fn setup_properties(
    items: Vec<FileItem>,
    prop_adp: PropertiesAdapter,
    prop_win: Weak<PropertiesWindow>,
) {
    /*
     *      Setup the data to show
     * */

    //TODO: Show some sort of loading animation while we wait for these operations on slow pcs.
    //TODO: specify changing owner only works as root
    //TODO: Make changing owner work if you are owner

    let _meta: Metadata;
    //Get metadata to pass around
    let metadata = if items.len() != 1 {
        None
    } else {
        let meta = get_file_metadata(&items[0].path);
        if meta.is_err() {
            return;
        }
        _meta = meta.unwrap();
        Some(&_meta)
    };
    if let Some(vec) = prop_adp
        .get_files()
        .as_any()
        .downcast_ref::<VecModel<FileItem>>()
    {
        vec.set_vec(items.clone());
    } else {
        prop_adp.set_files(Rc::new(VecModel::from(items.clone())).into());
    }

    setup_properties_info(&items, &prop_adp, metadata);
    setup_properties_permissions(&items, &prop_adp, metadata);
    setup_properties_advanced(&prop_adp, metadata);

    //calculate_directory_size also calculates multiple files
    //Still overkill for a single file though.
    if (metadata.is_some() && metadata.unwrap().is_dir()) || metadata.is_none() {
        calculate_directory_size(&items, &prop_adp, prop_win);
    }

    /*Reset State*/
    prop_adp.set_selected_tab_idx(0);
    prop_adp.set_perm_bits_dirty(false);
    prop_adp.set_uid_dirty(false);
    prop_adp.set_gid_dirty(false);
    //TODO: reset focus too

    if items.len() > 1 {
        let mut loc = items[0].path.rsplit_once("/").unwrap().0;
        if loc.is_empty() {
            loc = "/".into();
        }
        prop_adp.set_location(loc.into());
    }
}

///If items.len() == 1, then metadata MUST be Some
pub fn setup_properties_info(
    items: &Vec<FileItem>,
    prop_adp: &PropertiesAdapter,
    metadata: Option<&Metadata>,
) {
    if items.len() == 1 {
        let item = &items[0];
        let metadata = metadata.unwrap();
        prop_adp.set_filename((*item.file_name).to_string().into());

        /*Access time*/
        prop_adp.set_atime(format_date(metadata.atime() as i64));

        /*Creation time*/
        let created = metadata.created();
        let created = if created.is_err() {
            0
        } else {
            created
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        };
        prop_adp.set_ctime(format_date(created as i64));

        /*File Type*/
        prop_adp.set_type(get_file_magic_type(&item.path).into());

        /*Encoding*/
        if item.is_dir {
            prop_adp.set_encoding("N/A".into());
        } else {
            prop_adp.set_encoding(get_file_encoding(&item.path).into());
        }
    } else {
        prop_adp.set_filename("Multiple selections".into());
        //TODO: If they are the same, show the value
        /*Access time*/
        prop_adp.set_atime("N/A".into());

        /*Creation time*/

        prop_adp.set_ctime("N/A".into());

        /*File Type*/
        //Count the files, directories and links in the selection
        let mut file_count = 0;
        let mut dir_count = 0;
        let mut link_count = 0;
        for f in items {
            if f.is_link {
                link_count += 1;
            } else if f.is_dir {
                dir_count += 1;
            } else if !f.is_dir {
                file_count += 1;
            }
        }
        let mut s = String::new()
            + &(if file_count > 1 {
                file_count.to_string() + " files, "
            } else if file_count > 0 {
                file_count.to_string() + " file, "
            } else {
                String::new()
            })
            + &(if dir_count > 1 {
                dir_count.to_string() + " directories, "
            } else if dir_count > 0 {
                dir_count.to_string() + " directory, "
            } else {
                String::new()
            })
            + &(if link_count > 1 {
                link_count.to_string() + " symlinks, "
            } else if link_count > 0 {
                link_count.to_string() + " symlink, "
            } else {
                String::new()
            });

        //', ' => '.'
        s.replace_range((s.len() - 2)..s.len(), ".");
        prop_adp.set_type(s.into());

        prop_adp.set_encoding("N/A".into());
    }
}
///If items.len() == 1, then metadata MUST be Some
///files MUST NOT BE EMPTY
pub fn setup_properties_permissions(
    files: &Vec<FileItem>,
    prop_adp: &PropertiesAdapter,
    metadata: Option<&Metadata>,
) {
    if files.is_empty() {
        return;
    } else if files.len() == 1 {
        let metadata = metadata.unwrap();
        let file_uid = metadata.uid();
        let file_gid = metadata.gid();

        let uid = get_uid();
        let is_root = uid == 0;

        let mut username: String = String::from("");
        /*Users*/
        if let Ok(users) = get_all_users() {
            let file_username = users.get(&file_uid).unwrap();
            if let Some(user) = users.get(&uid) {
                username = String::from(user);
            }
            if is_root {
                prop_adp.set_is_root(true);
                let mut user_list: Vec<&String> = users.values().collect();
                user_list.sort_unstable_by(|a, b| {
                    if *a == file_username || **a == username {
                        Ordering::Less
                    } else if *b == file_username || **b == username {
                        Ordering::Greater
                    } else {
                        a.cmp(b)
                    }
                });

                prop_adp.set_owners(
                    Rc::new(VecModel::from(
                        user_list
                            .iter()
                            .map(|user| SharedString::from(*user))
                            .collect::<Vec<SharedString>>(),
                    ))
                    .into(),
                );
            }
            prop_adp.set_owner_value(file_username.into());
        }
        /*Groups*/
        if let Ok(groups) = get_all_groups() {
            let mut group_list: Vec<&Group> = groups.values().collect();
            let gid = get_gid();

            group_list.sort_unstable_by(|a, b| {
                if a.gid == file_gid || a.gid == gid {
                    Ordering::Less
                } else if b.gid == file_gid || b.gid == gid {
                    Ordering::Greater
                } else {
                    a.name.cmp(&b.name)
                }
            });

            let mut file_group = "";
            for g in group_list.iter() {
                if g.gid == file_gid {
                    file_group = &g.name;
                    prop_adp.set_group_value((&g.name).into());
                }
            }

            prop_adp.set_groups(
                Rc::new(VecModel::from(
                    group_list
                        .iter()
                        .filter(|g| {
                            is_root
                                || g.members.contains(&username)
                                || g.name == file_group
                                || g.name == username
                        })
                        .map(|group| SharedString::from(&group.name))
                        .collect::<Vec<SharedString>>(),
                ))
                .into(),
            );
        }
        /*Permission triplets*/
        let perm = metadata.permissions().mode() & 0o777;
        set_split_bitmask(prop_adp, perm as i32);
        prop_adp.set_perm_bits_str(format!("{:o}", perm).into());

    //Multiple files
    } else {
        //First we try to see if they all have the same values for some fields
        //If so, then we show them as if for a single file
        //If not, they will be blank, but still modifiable (and will affect all)

        let first_metadata = get_file_metadata(&files[0].path);
        if first_metadata.is_err() {
            log_error_str(&format!(
                "Could not get metadata for {}. Aborting.",
                files[0].path
            ));
            return;
        }
        let first_metadata = first_metadata.unwrap();

        //Will be none unless all files have the same
        let mut file_gid = Some(first_metadata.gid());
        let mut file_uid = Some(first_metadata.uid());
        let mut perm = Some(first_metadata.permissions().mode());

        for f in files {
            if let Ok(meta) = get_file_metadata(&f.path) {
                //Gid
                if file_gid != None && file_gid != Some(meta.gid()) {
                    file_gid = None;
                }
                //Gid
                if file_uid != None && file_uid != Some(meta.uid()) {
                    file_uid = None;
                }
                //Perms
                if perm != None && perm != Some(meta.permissions().mode()) {
                    perm = None;
                }
            } else {
                log_error_str(&format!("Could not get metadata for {}. Aborting.", f.path));
                return;
            }
        }

        let user_uid = get_uid();
        let is_root = user_uid == 0;
        let mut username: String = String::from("");

        /*Users*/
        if let Ok(users) = get_all_users() {
            let file_username = if file_uid.is_none() {
                ""
            } else {
                users.get(&file_uid.unwrap()).unwrap()
            };
            if let Some(user) = users.get(&user_uid) {
                username = String::from(user);
            }
            if is_root {
                prop_adp.set_is_root(true);
                let mut user_list: Vec<&String> = users.values().collect();
                user_list.sort_unstable_by(|a, b| {
                    if *a == file_username || **a == username {
                        Ordering::Less
                    } else if *b == file_username || **b == username {
                        Ordering::Greater
                    } else {
                        a.cmp(b)
                    }
                });

                prop_adp.set_owners(
                    Rc::new(VecModel::from(
                        user_list
                            .iter()
                            .map(|user| SharedString::from(*user))
                            .collect::<Vec<SharedString>>(),
                    ))
                    .into(),
                );
            }
            prop_adp.set_owner_value(file_username.into());
        }
        /*Groups*/
        if let Ok(groups) = get_all_groups() {
            let mut group_list: Vec<&Group> = groups.values().collect();
            let gid = get_gid();

            group_list.sort_unstable_by(|a, b| {
                if Some(a.gid) == file_gid || a.gid == gid {
                    Ordering::Less
                } else if Some(b.gid) == file_gid || b.gid == gid {
                    Ordering::Greater
                } else {
                    a.name.cmp(&b.name)
                }
            });

            let mut file_group = "";
            for g in group_list.iter() {
                if Some(g.gid) == file_gid {
                    file_group = &g.name;
                    prop_adp.set_group_value((&g.name).into());
                }
            }

            prop_adp.set_groups(
                Rc::new(VecModel::from(
                    group_list
                        .iter()
                        .filter(|g| {
                            is_root
                                || g.members.contains(&username)
                                || g.name == file_group
                                || g.name == username
                        })
                        .map(|group| SharedString::from(&group.name))
                        .collect::<Vec<SharedString>>(),
                ))
                .into(),
            );
        }
        /*Permission triplets*/
        if let Some(mut perm) = perm {
            perm = perm & 0o777;
            set_split_bitmask(prop_adp, perm as i32);
            prop_adp.set_perm_bits_str(format!("{:o}", perm).into());
        } else {
            set_split_bitmask(prop_adp, 0);
            prop_adp.set_perm_bits_str("".into());
        }
    }
}
pub fn setup_properties_advanced(prop_adp: &PropertiesAdapter, metadata: Option<&Metadata>) {
    if let Some(metadata) = metadata {
        prop_adp.set_inode(metadata.ino().to_string().into());
        prop_adp.set_device(metadata.dev().to_string().into());
        prop_adp.set_blocks(metadata.blocks().to_string().into());
        prop_adp.set_blksize(metadata.blksize().to_string().into());
    } else {
        //TODO: If they're all the same for multiple files, show them
        prop_adp.set_inode("".into());
        prop_adp.set_device("".into());
        prop_adp.set_blocks("".into());
        prop_adp.set_blksize("".into());
    }
}

/*
 *  Calculates the directory size by iterating with walkdir.
 *  Uses 2 threads, one to update the UI every 500ms and another to calculate
 *  the size and notify the first.
 * */

pub fn calculate_directory_size(
    files: &Vec<FileItem>,
    prop_adp: &PropertiesAdapter,
    window: Weak<PropertiesWindow>,
) {
    prop_adp.set_is_directory_calculated(false);
    prop_adp.set_directory_size(_i64 { a: 0, b: 0 });

    let (send_a, recv_a) = channel();
    let (send_b, recv_b) = channel();
    let window_clone = window.clone();
    std::thread::spawn(move || {
        let delay = Duration::from_millis(500); //UI update frequency
        while let Ok(data) = recv_a.recv() {
            if data != 0 {
                let _ = window_clone.upgrade_in_event_loop(move |w| {
                    let (a, b) = i64_to_i32(data);
                    w.global::<PropertiesAdapter>()
                        .set_directory_size(_i64 { a, b });
                });
            }
            sleep(delay);

            //Usually will be Err when it's done calculating
            if send_b.send(1).is_err() {
                break;
            }
        }
    });

    let files = files.clone();
    std::thread::spawn(move || {
        let mut total: i64 = 0;
        send_a.send(0).ok();

        for f in files {
            if f.is_dir {
                for entry_res in WalkDir::new(PathBuf::from(f.path.to_string())).follow_links(false)
                {
                    if let Ok(entry) = entry_res {
                        total += match entry.metadata() {
                            Ok(m) => m.size() as i64,
                            Err(_) => 0,
                        };
                    }
                    if recv_b.try_recv().is_ok() {
                        send_a.send(total).ok();
                    }
                }
            } else {
                total += i32_to_i64((f.size.a, f.size.b));
                if recv_b.try_recv().is_ok() {
                    send_a.send(total).ok();
                }
            }
        }

        //Final update so we get it instantly without waiting 500ms
        window
            .upgrade_in_event_loop(move |w| {
                let (a, b) = i64_to_i32(total);
                w.global::<PropertiesAdapter>()
                    .set_directory_size(_i64 { a, b });
                w.global::<PropertiesAdapter>()
                    .set_is_directory_calculated(true)
            })
            .ok();
    });
}

/*
 *  Save and close
 * */
pub fn save(prop_win: Rc<Weak<PropertiesWindow>>, mw: Rc<Weak<MainWindow>>) {
    let w = prop_win.unwrap();
    let prop_adp = w.global::<PropertiesAdapter>();

    let files = prop_adp.get_files();
    let single_file = files.row_count() == 1;
    for f in files.iter() {
        let path_str = f.path.to_string();
        let mut path = PathBuf::from(&path_str);

        if single_file {
            let new_filename = prop_adp.get_filename().to_string();
            let new_path = path.with_file_name(&new_filename);

            //Update file name
            if f.file_name != new_filename {
                let ret = rename_file(&path, &new_path);
                if ret.is_err() {
                    log_error_str("Could not rename file"); //TODO:
                } else {
                    path = new_path;
                }
            }
        }

        //Chown uid
        if prop_adp.get_uid_dirty() {
            let owner_str = prop_adp.get_owner_value().to_string();
            if let Ok(users) = get_all_users() {
                if let Some((k, _)) = users.iter().find(|(_, v)| **v == owner_str) {
                    let ret = lchown(path.clone(), Some(*k), None);
                    if ret.is_err() {
                        log_error(ret.err().unwrap());
                    }
                } else {
                    log_error_str("The target user does not exist.")
                }
            } else {
                log_error_str("Could not get users. Does /etc/passwd have the right permissions?");
            }
        }

        //Chown gid
        if prop_adp.get_gid_dirty() {
            let group_str = prop_adp.get_group_value().to_string();
            if let Ok(groups) = get_all_groups() {
                if let Some((k, _)) = groups.iter().find(|(_, v)| v.name == group_str) {
                    let ret = lchown(path.clone(), None, Some(*k));
                    if ret.is_err() {
                        log_error(ret.err().unwrap());
                    }
                } else {
                    log_error_str("The target group does not exist.")
                }
            } else {
                log_error_str("Could not get groups. Does /etc/group have the right permissions?");
            }
        }

        //Permissions
        if prop_adp.get_perm_bits_dirty() {
            if let Ok(new_mode) =
                u32::from_str_radix(&(prop_adp.get_perm_bits_str().to_string()), 8)
            {
                let ret = set_permissions(path, Permissions::from_mode(new_mode));
                if ret.is_err() {
                    log_error(ret.err().unwrap());
                }
            } else {
                log_error_str("Could not parse the permission mode.")
            }
        }
    }

    //Refresh UI
    set_current_tab_file(
        mw.unwrap().global::<TabsAdapter>().invoke_get_current_tab(),
        mw,
        false,
    );
    w.hide().unwrap();
}

pub fn rename_file(from: &Path, to: &Path) -> Result<(), Error> {
    //Make sure there's no file with that name already
    let already_exists = std::fs::read_dir(match to.parent() {
        Some(p) => p,
        None => Path::new("/"),
    })?
    .find(|entry| match entry {
        Ok(f) => {
            let b = to.file_name();
            if b.is_some() {
                return f.file_name() == b.unwrap();
            } else {
                return false;
            }
        }
        Err(_) => false,
    });

    if already_exists.is_some() {
        Err(Error::new(ErrorKind::AlreadyExists, "Already Exists"))
    } else {
        std::fs::rename(from, to)
    }
}
