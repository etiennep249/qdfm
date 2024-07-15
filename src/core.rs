use magic::cookie::{DatabasePaths, Flags};
use slint::Weak;
use syscalls::syscall0;

use crate::{
    ui::*,
    utils::{error_handling::log_error, types::i64_to_i32},
};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, Metadata},
    process::Command,
    rc::Rc,
    time::SystemTime,
};

pub fn generate_files_for_path(path: &str) -> Vec<FileItem> {
    let dir = fs::read_dir(path);
    if dir.is_err() {
        log_error(dir.err().unwrap());
        return Vec::new();
    }
    dir.unwrap()
        .map(|file| {
            if let Ok(f) = file {
                if let Ok(meta) = std::fs::metadata(f.path()) {
                    let (size_a, size_b) = if meta.is_dir() {
                        (0, 0) //So that directories don't get sorted by size
                    } else {
                        i64_to_i32(meta.len() as i64)
                    };
                    let (date_a, date_b);
                    if let Ok(modified) = meta.modified() {
                        if let Ok(modified_dr) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                            (date_a, date_b) = i64_to_i32(modified_dr.as_secs() as i64);
                        } else {
                            return bad_file();
                        }
                    } else {
                        return bad_file();
                    }
                    FileItem {
                        path: f.path().to_str().unwrap().into(),
                        file_name: f.file_name().to_str().unwrap().into(),
                        is_dir: meta.is_dir(),
                        size: _i64 {
                            a: size_a,
                            b: size_b,
                        },
                        date: _i64 {
                            a: date_a,
                            b: date_b,
                        },
                        file_type: f
                            .path()
                            .extension()
                            .and_then(OsStr::to_str)
                            .unwrap_or_else(|| "N/A")
                            .into(),
                        is_link: f.file_type().unwrap().is_symlink(),
                        extension: f
                            .path()
                            .extension()
                            .and_then(OsStr::to_str)
                            .unwrap_or("")
                            .into(),
                    }
                } else {
                    bad_file()
                }
            } else {
                bad_file()
            }
        })
        .collect::<Vec<FileItem>>()
}

/*
 *      Generates a map of <uid, name> from /etc/passwd
 * */
pub fn get_all_users() -> Result<HashMap<u32, String>, std::io::Error> {
    let mut map: HashMap<u32, String> = HashMap::new();
    std::fs::read_to_string("/etc/passwd")?
        .split("\n")
        .for_each(|line| {
            if !line.starts_with("#") && !line.trim().is_empty() {
                let tokens = line.split(":").collect::<Vec<&str>>();
                if let Ok(uid) = tokens[2].parse::<u32>() {
                    map.insert(uid, tokens[0].trim().into());
                }
            }
        });
    Ok(map)
}

/*
 *  Generates a map of <gid, Group> from /etc/group
 * */

pub struct Group {
    pub gid: u32,
    pub name: String,
    pub members: Vec<String>,
}
pub fn get_all_groups() -> Result<HashMap<u32, Group>, std::io::Error> {
    let mut map: HashMap<u32, Group> = HashMap::new();
    std::fs::read_to_string("/etc/group")?
        .split("\n")
        .for_each(|line| {
            if !line.starts_with("#") && !line.trim().is_empty() {
                let tokens = line.split(":").collect::<Vec<&str>>();
                if let Ok(gid) = tokens[2].parse::<u32>() {
                    map.insert(
                        gid,
                        Group {
                            name: String::from(tokens[0]),
                            gid,
                            members: tokens[3].split(",").map(|s| s.trim().to_string()).collect(),
                        },
                    );
                }
            }
        });
    Ok(map)
}

//Returns the effective user id via syscall
//"These functions are always successful and never modify errno."
pub fn get_uid() -> u32 {
    unsafe { syscall0(syscalls::Sysno::geteuid).unwrap() as u32 }
}

//Returns the effective group id via syscall
//"These functions are always successful and never modify errno."
pub fn get_gid() -> u32 {
    unsafe { syscall0(syscalls::Sysno::getegid).unwrap() as u32 }
}

pub fn get_file_magic_type(path: &str) -> String {
    let cookie =
        magic::Cookie::open(Flags::ERROR | Flags::NO_CHECK_ENCODING | Flags::PRESERVE_ATIME);
    if cookie.is_err() {
        return "Unknown / Magic Open Error".into();
    }
    let cookie = cookie.unwrap().load(&DatabasePaths::default());
    if cookie.is_err() {
        return "Unknown / Magic Database Error".into();
    }
    let result = cookie.unwrap().file(path);
    if result.is_err() {
        return "Unknown / Magic Analysis Error".into();
    }
    return result.unwrap();
}

pub fn get_file_encoding(path: &str) -> String {
    let cookie = magic::Cookie::open(Flags::ERROR | Flags::MIME_ENCODING | Flags::PRESERVE_ATIME);
    if cookie.is_err() {
        return "Unknown / Magic Open Error".into();
    }
    let cookie = cookie.unwrap().load(&DatabasePaths::default());
    if cookie.is_err() {
        return "Unknown / Magic Database Error".into();
    }
    let result = cookie.unwrap().file(path);
    if result.is_err() {
        return "Unknown / Magic Analysis Error".into();
    }
    return result.unwrap();
}

pub fn get_file_metadata(path: &str) -> Result<Metadata, std::io::Error> {
    std::fs::metadata(path)
}

pub fn bad_file() -> FileItem {
    FileItem {
        path: "???".into(),
        file_name: "???".into(),
        is_dir: false,
        size: _i64 { a: 0, b: 0 },
        date: _i64 { a: 0, b: -1 }, //-1 Used as error condition, faster than comparing strings
        file_type: "Unknown / Bad file".into(),
        is_link: false,
        extension: "".into(),
    }
}
pub fn run_command(command: &str, _mw: Rc<Weak<MainWindow>>) {
    Command::new("setsid")
        .args(command.split(" ").collect::<Vec<&str>>())
        .spawn()
        .expect("failed to execute process");
}
