use crate::{
    ui::*,
    utils::{error_handling::log_error, types::i64_to_i32},
};
use std::fs;

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
                    let (size_a, size_b) = i64_to_i32(meta.len() as i64);
                    FileItem {
                        path: f.path().to_str().unwrap().into(),
                        file_name: f.file_name().to_str().unwrap().into(),
                        is_dir: meta.is_dir(),
                        size: _i64 {
                            a: size_a,
                            b: size_b,
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
    }
}
