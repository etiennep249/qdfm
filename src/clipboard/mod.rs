use copy::rename_or_copy_file;
use slint::{ComponentHandle, Weak};

use crate::{
    callbacks::filemanager::set_current_tab_file,
    ui::{FileItem, MainWindow, TabsAdapter},
    utils::{error_handling::log_error_str, types},
};
use std::{
    collections::VecDeque,
    path::Path,
    rc::Rc,
    sync::{Mutex, OnceLock},
};

pub mod copy;
pub mod cut;
pub mod delete;
pub mod paste;

///Contains the files to delete after the paste
pub static CUT_BUFFER: OnceLock<Mutex<Vec<FileItem>>> = OnceLock::new();

//1mb, TODO: config file
const PROGRESS_WINDOW_BYTE_THRESHOLD: i64 = 1048576;
const ESTIMATE_CAPACITY: usize = 10000;
const PER_FILE_OVERHEAD: f64 = 0.0001f64;

fn format_size_and_filecount_progress_status(
    current_size: i64,
    total_size: i64,
    current_files: i64,
    total_files: i64,
) -> String {
    if total_size <= 0 {
        "Calculating...".into()
    } else {
        types::format_size(current_size as u64, false).to_string()
            + " / "
            + &types::format_size(total_size as u64, false)
            + " | "
            + &current_files.to_string()
            + " / "
            + &total_files.to_string()
            + " files."
    }
}

fn format_size_progress_status(current: i64, total: i64) -> String {
    if total < 0 {
        "Calculating...".into()
    } else {
        types::format_size(current as u64, false).to_string()
            + " / "
            + &types::format_size(total as u64, false)
    }
}

fn average_speed(vec: &VecDeque<f64>) -> f64 {
    let mut sum = 0f64;
    for i in vec {
        sum += i;
    }
    sum / vec.len() as f64
}

///https://en.wikipedia.org/wiki/Moving_average
fn update_weighted_speed(vec: &mut VecDeque<f64>, speed: f64, capacity: usize) {
    if vec.len() < capacity {
        vec.push_back(speed);
    } else {
        let old_average = vec.iter().sum::<f64>() / vec.len() as f64;

        vec.pop_front();
        let weight = 0.6;
        let adjusted_speed = weight * speed + (1.0 - weight) * old_average;
        vec.push_back(adjusted_speed);
    }
}

#[inline]
fn estimate_time_left(speed: f64, bytes_left: i64) -> f64 {
    bytes_left as f64 / speed / 1000f64
}
pub fn file_exists_in_dir(dir_path: &str, filename: &str) -> Result<bool, ()> {
    let dir = std::fs::read_dir(dir_path);
    if dir.is_err() {
        log_error_str(&format!("Cannot access {}", dir_path));
        return Err(());
    }
    let already_exists = dir.unwrap().find(|entry| match entry {
        Ok(f) => {
            return f.file_name() == filename;
        }
        Err(_) => false,
    });
    if already_exists.is_some() {
        return Ok(true);
    } else {
        return Ok(false);
    }
}
///
///  Called when a file is dropped in the window
///  The file is moved from its original location to the current folder
///
pub fn move_file(mw: Rc<Weak<MainWindow>>, buf: &str, destination: &str) {
    if file_exists_in_dir(destination, buf) == Ok(false) {
        if let Some(filename) = buf.split("/").last() {
            if rename_or_copy_file(
                Path::new(buf),
                Path::new(&(destination.to_owned() + "/" + &filename)),
            )
            .is_err()
            {
                //File was copied rather than moved
                //TODO: Add default behavior config? Should we manually delete the original after
                //if they were on a different filesystem?
            }
        }
    }
    //Refresh UI
    set_current_tab_file(
        mw.unwrap().global::<TabsAdapter>().invoke_get_current_tab(),
        mw,
        false,
    );
}
