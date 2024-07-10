use slint::{ComponentHandle, SharedString, VecModel, Weak};
use std::{
    cmp::Ordering,
    fs::Metadata,
    io::{Error, ErrorKind},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
    rc::Rc,
    sync::mpsc::channel,
    thread::sleep,
    time::{Duration, UNIX_EPOCH},
};
use walkdir::WalkDir;

use crate::{
    core::{
        get_all_groups, get_all_users, get_file_encoding, get_file_magic_type, get_file_metadata,
        get_gid, get_uid, Group,
    },
    ui::*,
    utils::types::{format_date, i64_to_i32},
};

pub fn setup_properties(
    item: FileItem,
    prop_adp: PropertiesAdapter,
    prop_win: Weak<PropertiesWindow>,
) {
    /*
     *      Setup the data to show
     * */

    //TODO: Show some sort of loading animation while we wait for these operations on slow pcs.
    //TODO: specify changing owner only works as root

    //Get metadata to pass around
    let metadata = get_file_metadata(&item.path);
    if metadata.is_err() {
        return;
    }
    let metadata = metadata.unwrap();

    setup_properties_info(&item, &prop_adp, &metadata);
    setup_properties_permissions(&prop_adp, &metadata);
    setup_properties_advanced(&prop_adp, &metadata);
    if metadata.is_dir() {
        calculate_directory_size(&item, &prop_adp, prop_win);
    }

    /*Reset State*/
    prop_adp.set_selected_tab_idx(0);
    prop_adp.set_perm_bits_dirty(false);
    prop_adp.set_uid_dirty(false);
    prop_adp.set_gid_dirty(false);
    //TODO: reset focus too, unsure how to do it from rust code, has-focus is out only
    prop_adp.set_file(item);
}

pub fn setup_properties_info(item: &FileItem, prop_adp: &PropertiesAdapter, metadata: &Metadata) {
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
}
pub fn setup_properties_permissions(prop_adp: &PropertiesAdapter, metadata: &Metadata) {
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
    prop_adp.set_perm_bits(perm as i32);
    prop_adp.set_perm_bits_str(format!("{:o}", perm).into());
}
pub fn setup_properties_advanced(prop_adp: &PropertiesAdapter, metadata: &Metadata) {
    prop_adp.set_inode(metadata.ino().to_string().into());
    prop_adp.set_device(metadata.dev().to_string().into());
    prop_adp.set_blocks(metadata.blocks().to_string().into());
    prop_adp.set_blksize(metadata.blksize().to_string().into());
}

/*
 *  Calculates the directory size by iterating with walkdir.
 *  Uses 2 threads, one to update the UI every 500ms and another to calculate
 *  the size and notify the first.
 * */

pub fn calculate_directory_size(
    item: &FileItem,
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

    let path = item.path.clone().to_string();
    std::thread::spawn(move || {
        let mut total: i64 = 0;
        let _ = send_a.send(0);
        for entry_res in WalkDir::new(&path).follow_links(false) {
            if let Ok(entry) = entry_res {
                total += match entry.metadata() {
                    Ok(m) => m.size() as i64,
                    Err(_) => 0,
                };
            }
            if let Ok(_) = recv_b.try_recv() {
                let _ = send_a.send(total);
            }
        }

        //Final update so we get it instantly without waiting 500ms
        let _ = window.upgrade_in_event_loop(move |w| {
            let (a, b) = i64_to_i32(total);
            w.global::<PropertiesAdapter>()
                .set_directory_size(_i64 { a, b });
            w.global::<PropertiesAdapter>()
                .set_is_directory_calculated(true)
        });
    });
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
