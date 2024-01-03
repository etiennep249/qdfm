use crate::{
    ui::*,
    utils::{error_handling::log_error, types::i64_to_i32},
};
use std::{fs, time::SystemTime};

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

pub fn bad_file() -> FileItem {
    FileItem {
        path: "?".into(),
        file_name: "?".into(),
        is_dir: false,
        size: _i64 { a: 0, b: 0 },
        date: _i64 { a: 0, b: -1 }, //-1 Used as error condition, faster than comparing strings
    }
}
