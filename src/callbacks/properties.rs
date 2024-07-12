use crate::{
    core::{get_all_groups, get_all_users},
    file_properties::rename_file,
    ui::*,
    utils::error_handling::{log_error, log_error_str},
};
use slint::{ComponentHandle, Weak};
use std::{
    fs::{set_permissions, Permissions},
    os::unix::fs::{lchown, PermissionsExt},
    path::Path,
    rc::Rc,
};

use super::filemanager::set_current_tab_file;

/*
 *  Save and close
 * */
pub fn ok(prop_win: Rc<Weak<PropertiesWindow>>, mw: Rc<Weak<MainWindow>>) {
    let w = prop_win.unwrap();
    let prop_adp = w.global::<PropertiesAdapter>();
    let original_file = prop_adp.get_file();

    let path_str = original_file.path.to_string();
    let mut path = Path::new(&path_str);

    let new_filename = prop_adp.get_filename().to_string();
    let new_path = path.with_file_name(&new_filename);

    //Update file name
    if original_file.file_name != new_filename {
        let ret = rename_file(path, &new_path);
        if ret.is_err() {
            log_error_str("Could not rename file");
        } else {
            path = &new_path;
        }
    }

    //Refresh UI
    set_current_tab_file(
        mw.unwrap().global::<TabsAdapter>().invoke_get_current_tab(),
        mw,
        false,
    );

    //Chown uid
    if prop_adp.get_uid_dirty() {
        let owner_str = prop_adp.get_owner_value().to_string();
        if let Ok(users) = get_all_users() {
            if let Some((k, _)) = users.iter().find(|(_, v)| **v == owner_str) {
                let ret = lchown(path, Some(*k), None);
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
                let ret = lchown(path, None, Some(*k));
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
        if let Ok(new_mode) = u32::from_str_radix(&(prop_adp.get_perm_bits_str().to_string()), 8) {
            let ret = set_permissions(path, Permissions::from_mode(new_mode));
            if ret.is_err() {
                log_error(ret.err().unwrap());
            }
        } else {
            log_error_str("Could not parse the permission mode.")
        }
    }

    w.hide().unwrap();
}
pub fn cancel(prop_win: Rc<Weak<PropertiesWindow>>) {
    let w = prop_win.unwrap();
    w.hide().unwrap();
}
pub fn bitmask(i: i32, prop_bits: i32) -> bool {
    (prop_bits & i) > 0
}

pub fn recalculate_bitmask(mask: i32, prop_win: Rc<Weak<PropertiesWindow>>) {
    let prop_win = prop_win.unwrap();
    let prop_adp = prop_win.global::<PropertiesAdapter>();
    let new_bits = prop_adp.get_perm_bits() ^ mask;
    prop_adp.set_perm_bits(new_bits);
    prop_adp.set_perm_bits_str(format!("{:o}", new_bits).into());
}
