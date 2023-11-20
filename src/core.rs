use crate::{ui::*, utils::error_handling::log_error};
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
                FileItem {
                    path: f.path().to_str().unwrap().into(),
                    file_name: f.file_name().to_str().unwrap().into(),
                }
            } else {
                FileItem {
                    path: "?".into(),
                    file_name: "?".into(),
                }
            }
        })
        .collect::<Vec<FileItem>>()
}
