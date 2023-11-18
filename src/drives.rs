use slint::VecModel;
use std::rc::Rc;
use sysinfo::Disk;
use sysinfo::DiskExt;
use sysinfo::SystemExt;

use crate::globals::sysinfo_lock;
use crate::ui::*;

static HIDDEN_MOUNTS: [&str; 1] = ["/boot"];

pub fn get_drives() -> Rc<VecModel<SidebarItem>> {
    let mut system = sysinfo_lock();
    system.refresh_disks_list();

    let drives: Rc<VecModel<SidebarItem>> = Rc::new(VecModel::default());

    for d in system.disks() {
        let drive_name = d.mount_point().to_str().unwrap();
        if is_hidden(drive_name) {
            continue;
        }
        let final_format = "  ".to_owned() + &format_drive_name(drive_name);
        drives.push(SidebarItem {
            text: final_format.into(),
        });
    }
    drives
}

fn is_hidden(s: &str) -> bool {
    for mp in HIDDEN_MOUNTS {
        if s == mp {
            return true;
        }
    }
    false
}

fn drive_pct(d: &Disk) -> u64 {
    100 - (100 * d.available_space() / d.total_space())
}

fn drive_pct_formatted(d: &Disk) -> String {
    drive_pct(d).to_string() + "%"
}

// Takes the name of the last folder and capitalizes it
// "/mnt/storage" -> "Storage"
fn format_drive_name(s: &str) -> String {
    if s == "/" {
        return String::from(s);
    }
    match s.rsplit_once("/") {
        None => String::from(s),
        Some(e) => {
            if let Some(c) = e.1.get(0..1) {
                let cc = String::from(c).to_uppercase();
                if let Some(s2) = e.1.get(1..) {
                    cc + s2
                } else {
                    cc
                }
            } else {
                String::from(e.1)
            }
        }
    }
}
