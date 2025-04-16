use super::{
    average_speed, estimate_time_left, format_size_progress_status, update_weighted_speed,
    ESTIMATE_CAPACITY, PER_FILE_OVERHEAD,
};
use crate::{clipboard::CUT_BUFFER, ui::FileItem, utils::error_handling::log_error_str};
use arboard::{Clipboard, SetExtLinux};
use std::{
    collections::VecDeque,
    fs::read_link,
    path::{Path, PathBuf},
    sync::{mpsc::Sender, Mutex},
    thread::{self},
    time::Instant,
};

///Copies a file to the clipboard
pub fn copy_file(files: Vec<FileItem>, is_cut: bool) {
    thread::spawn(move || {
        if let Ok(mut clip) = Clipboard::new() {
            let text = files
                .iter()
                .fold(String::new(), |lhs, rhs| lhs + "file://" + &rhs.path + "\n");

            if !is_cut {
                //Reset the cut buffer
                if let Ok(mut buf) = CUT_BUFFER.get_or_init(|| Mutex::new(Vec::new())).lock() {
                    *buf = Vec::new();
                }
            }

            if clip.set().wait().text(text).is_err() {
                log_error_str("Could not set the clipboard text");
            }
        } else {
            log_error_str("Could not find a clipboard.");
        }
    });

    //Since the process is responsible for providing the clipboard, we cannot join this thread,
    //ever.
    /*#[cfg(test)]
    thread.join().unwrap();*/
    #[cfg(test)]
    std::thread::sleep(std::time::Duration::from_millis(10)); //Make sure clipboard gets set
}

///Copies a single file while maintaining speed and progress information
///Will return Err if the receiver was disconnected (and we should cancel the operation)
///Caller is responsible to verify whether or not this will be overwritten
pub fn copy_single_file_operation(
    to: PathBuf,
    entry: &Path,
    current: &mut i64,
    speed_vec: &mut VecDeque<f64>,
    avg_speed: &mut f64,
    remaining_time: &mut f64,
    total: i64,
    all_success: &mut bool,
    progress: &Sender<(f32, String, f64, bool)>,
    is_rename: bool, //If the file should be moved instead of copied
) -> Result<(), ()> {
    let is_err = if entry.is_symlink() {
        std::os::unix::fs::symlink(&read_link(entry).unwrap(), &to).is_err()
    } else {
        let before = Instant::now();
        let res = if is_rename {
            rename_or_copy_file(entry, &to)
        } else {
            std::fs::copy(entry, to)
        };
        let mut elapsed = before.elapsed().as_nanos() as f64 / 1000000f64;
        if elapsed <= 0f64 {
            elapsed = PER_FILE_OVERHEAD;
        }
        if let Ok(bytecount) = res {
            if bytecount > 0 {
                *current += bytecount as i64;
                update_weighted_speed(speed_vec, bytecount as f64 / elapsed, ESTIMATE_CAPACITY);
                *avg_speed = average_speed(speed_vec);
                *remaining_time = estimate_time_left(*avg_speed, total - *current);
            }
            false
        } else {
            true
        }
    };
    if is_err {
        log_error_str(&format!(
            "File could not be pasted: {}",
            entry.to_string_lossy()
        ));
        *all_success = false;
    } else {
        if progress
            .send((
                *current as f32 / total as f32,
                format_size_progress_status(*current, total),
                *remaining_time,
                false,
            ))
            .is_err()
        {
            return Err(());
        }
    }
    Ok(())
}

///Will rename/move the file if it's on the same filesystem, otherwise just copy
///Original file will be overwritten
pub fn rename_or_copy_file(from: &Path, to: &Path) -> Result<u64, std::io::Error> {
    if std::fs::rename(from, to).is_err() {
        return std::fs::copy(from, to);
    }
    Ok(0)
}
