use crate::{
    callbacks::filemanager::set_current_tab_file,
    globals::selected_files_lock,
    progress_window::show_progress_window,
    ui::*,
    utils::{error_handling::log_error_str, types},
};
use arboard::{Clipboard, SetExtLinux};
use slint::{format, ComponentHandle, Weak};
use std::{
    collections::VecDeque,
    fs::{metadata, read_link},
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
    sync::{
        mpsc::{channel, Sender},
        Mutex, OnceLock,
    },
    thread,
    time::{Duration, Instant},
};
use walkdir::WalkDir;

///Contains the files to delete after the paste
static CUT_BUFFER: OnceLock<Mutex<Vec<FileItem>>> = OnceLock::new();

//1mb, TODO: config file
const PROGRESS_WINDOW_BYTE_THRESHOLD: i64 = 1048576;

///Copies a file to the clipboard
pub fn copy_file(files: Vec<FileItem>) {
    thread::spawn(move || {
        if let Ok(mut clip) = Clipboard::new() {
            let text = files
                .iter()
                .fold(String::new(), |lhs, rhs| lhs + "file://" + &rhs.path + "\n");
            if clip.set().wait().text(text).is_err() {
                log_error_str("Could not set the clipboard text");
            }
        } else {
            log_error_str("Could not find a clipboard.");
        }
    });
}

pub fn cut_file(files: Vec<FileItem>) {
    thread::spawn(|| {
        let buf = CUT_BUFFER.get_or_init(|| Mutex::new(Vec::new())).lock();

        if let Ok(mut buf_lock) = buf {
            *buf_lock = files.clone();
            copy_file(files);
        } else {
            log_error_str("Could not get the cut buffer. The operation has been canceled.");
        }
    });
}
const CAPACITY: usize = 10000;
const PER_FILE_OVERHEAD: f64 = 0.0001f64;
///Pastes the selected file(s) in to_path.
///Done in another thread.
//TODO: Implement copy ourselves so we can have progress info for large files
pub fn paste_file(to_path: PathBuf, mw: Rc<Weak<MainWindow>>) {
    //Get a weak ref to pass since we can't deref the Rc
    let mw = mw.unwrap().as_weak();
    thread::spawn(move || {
        let clipboard = Clipboard::new();
        if clipboard.is_err() {
            log_error_str("Could not find a clipboard.");
            return;
        }
        let text = clipboard.unwrap().get_text();
        if text.is_err() {
            //Content is not a file, move on
            return;
        }
        let text = text.unwrap();
        if !text.starts_with("file:///") {
            //Content is not a file, move on
            return;
        }

        //TODO: not needed if a tiny file

        //Start a progress window
        let (progress, recv) = channel();

        //We have to check if the receiver is dead (if the transfer was canceled) everytime we send
        if progress
            .send((0.0, "Calculating...".into(), -1f64, true))
            .is_err()
        {
            return;
        }

        //Unwrap is supposedly infallible
        let mut paths: Vec<PathBuf> = text
            .split("\n")
            .map(|s| PathBuf::from_str(&s.replace("file://", "")).unwrap())
            .collect();

        //Since the last path is going to contain a newline but not a separating one,
        //Remove it from the list of paths
        paths.remove(paths.len() - 1);

        let buf = CUT_BUFFER.get_or_init(|| Mutex::new(Vec::new())).lock();
        if buf.is_err() {
            log_error_str("Cut buffer could not be accessed.");
            return;
        }
        //If this gets set to false, we don't delete the original in case of a Cut/Paste
        let mut all_success = true;
        let mut buf_lock = buf.unwrap();

        if progress
            .send((0.0, "Calculating status.".into(), -1f64, true))
            .is_err()
        {
            return;
        }

        //Calculate paste size
        //Errors are ignored(since if it can't get metadata, good chance it can't read/copy either)
        //So the total adds up
        let mut total = 0;
        for path in paths.iter() {
            if let Ok(m) = metadata(path) {
                if m.is_dir() {
                    for entry_res in WalkDir::new(path).follow_links(false) {
                        if let Ok(entry) = entry_res {
                            total += entry.metadata().and_then(|m| Ok(m.len())).unwrap_or(0);
                        }
                    }
                } else if !m.is_symlink() {
                    total += m.len();
                }
            }
        }
        if total == 0 {
            log_error_str("Error calculating the total size to paste. You will not know the status, but the operation should proceed.");
            return;
        }

        //Don't bother showing a progress window if the file is too small
        //Messages will just end up being sent nowhere/never read
        if total > PROGRESS_WINDOW_BYTE_THRESHOLD as u64 {
            show_progress_window(mw.clone(), recv, Duration::from_millis(100));
        }

        let total = total as i64;

        let mut current: i64 = 0;
        let mut remaining_time: f64 = -1f64;

        let mut speed_vec: VecDeque<f64> = VecDeque::with_capacity(CAPACITY);
        let mut avg_speed: f64 = -1f64;

        if progress
            .send((0.0, format_size_progress_status(0, total), -1f64, true))
            .is_err()
        {
            return;
        }

        for path in paths.iter() {
            if to_path.to_str().unwrap().contains(path.to_str().unwrap()) {
                //TODO:
                log_error_str("Error: Recursive paste currently not enabled. Please copy to another destination first.");
                return;
            }

            // We are copying a directory
            if path.is_dir() {
                //Check if the folder already exists
                let exists = file_exists_in_dir(
                    to_path.to_str().unwrap(),
                    path.file_name().unwrap().to_str().unwrap(),
                );
                if exists.is_err() {
                    return;
                } else if exists == Ok(true) {
                    //TODO:
                    log_error_str("already exists TODO prompt for rename");
                    return;
                }

                let base_dir_path = path.parent().unwrap().to_string_lossy().to_string();

                //Loop over every file we have to copy
                for entry_res in WalkDir::new(&path) {
                    if entry_res.is_err() {
                        log_error_str(&format!(
                            "File cannot be accessed. Skipping. Perhaps a permission issue? Error Text: {}",
                            entry_res.err().unwrap().to_string()));
                        all_success = false;
                        continue;
                    }
                    let entry = entry_res.unwrap();
                    //Sub-item is a directory
                    if entry.path().is_dir() {
                        let dir = entry
                            .path()
                            .to_string_lossy()
                            .strip_prefix(&(base_dir_path.clone() + "/"))
                            .unwrap()
                            .to_string();
                        let is_err = if entry.path_is_symlink() {
                            std::os::unix::fs::symlink(
                                read_link(entry.path()).unwrap(),
                                to_path.join(&dir),
                            )
                            .is_err()
                        } else {
                            std::fs::create_dir_all(to_path.join(&dir)).is_err()
                        };
                        if is_err {
                            log_error_str("Could not create the directory. Canceling operations. Some files may already have been copied.");
                            return;
                        }
                    //Sub-item is a file
                    } else {
                        if copy_single_file(
                            to_path
                                .join(path.file_name().unwrap())
                                .join(entry.path().strip_prefix(path).unwrap()),
                            entry.path(),
                            &mut current,
                            &mut speed_vec,
                            &mut avg_speed,
                            &mut remaining_time,
                            total,
                            &mut all_success,
                            &progress,
                        )
                        .is_err()
                        {
                            return;
                        }
                    }
                }
            //We are copying a file
            } else if path.is_file() {
                if copy_single_file(
                    to_path.join(path.file_name().unwrap()),
                    path,
                    &mut current,
                    &mut speed_vec,
                    &mut avg_speed,
                    &mut remaining_time,
                    total,
                    &mut all_success,
                    &progress,
                )
                .is_err()
                {
                    return;
                }
            }
        }

        //If this was a Cut/Paste operation, delete the original (unless this was a partial success)
        if buf_lock.len() > 0 {
            if !all_success {
                log_error_str("Not all operations succeeded, the original file(s) and/or folder(s) have not been deleted during the Cut/Paste operation.");
            } else {
                for path in paths.iter() {
                    if path.is_dir() {
                        if std::fs::remove_dir_all(path).is_err() {
                            log_error_str(
                            &format!("Source directory could not be removed during Cut/Paste operation. Directory: {}", path.to_str().unwrap_or("None"))
                        );
                        }
                    } else {
                        if std::fs::remove_file(path).is_err() {
                            log_error_str(&format!(
                            "Source file could not be removed during Cut/Paste operation. File: {}",
                            path.to_str().unwrap_or("None")
                        ));
                        }
                    }
                }
            }
            *buf_lock = Vec::new();
        }
        //If we didn't show the progress window, we need to refresh manually
        if total <= PROGRESS_WINDOW_BYTE_THRESHOLD {
            mw.upgrade_in_event_loop(|w| {
                set_current_tab_file(
                    w.global::<TabsAdapter>().invoke_get_current_tab(),
                    Rc::new(w.as_weak()),
                    false,
                );
            })
            .ok();
        }
    });
}

///Copies a single file while maintaining speed and progress
///Will return Err if the receiver was disconnected (and we should cancel the operation)
fn copy_single_file(
    to: PathBuf,
    entry: &Path,
    current: &mut i64,
    speed_vec: &mut VecDeque<f64>,
    avg_speed: &mut f64,
    remaining_time: &mut f64,
    total: i64,
    all_success: &mut bool,
    progress: &Sender<(f32, String, f64, bool)>,
) -> Result<(), ()> {
    /*let exists = file_exists_in_dir(
        to_path.to_str().unwrap(),
        path.file_name().unwrap().to_str().unwrap(),
    );
    if exists.is_err() {
        return;
    } else if exists == Ok(true) {
        //TODO:
        log_error_str("already exists TODO prompt for rename");
        return;
    }*/
    let is_err = if entry.is_symlink() {
        std::os::unix::fs::symlink(&read_link(entry).unwrap(), &to).is_err()
    } else {
        let before = Instant::now();
        let res = std::fs::copy(entry, to);
        let mut elapsed = before.elapsed().as_nanos() as f64 / 1000000f64;
        if elapsed <= 0f64 {
            elapsed = PER_FILE_OVERHEAD;
        }
        if let Ok(bytecount) = res {
            if bytecount > 0 {
                *current += bytecount as i64;
                update_weighted_speed(speed_vec, bytecount as f64 / elapsed, CAPACITY);
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

fn format_size_progress_status(current: i64, total: i64) -> String {
    if total < 0 {
        "Calculating...".into()
    } else {
        types::format_size(current as u64, false).to_string()
            + " / "
            + &types::format_size(total as u64, false)
    }
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

pub fn delete(mww: Rc<Weak<MainWindow>>) {
    let mw = mww.clone().unwrap().as_weak();
    thread::spawn(move || {
        let (progress, recv) = channel();
        let selected_files_lock = selected_files_lock();
        let selected_files = selected_files_lock.values();

        if progress
            .send((0.0, "Calculating status.".into(), -1f64, true))
            .is_err()
        {
            return;
        }
        //Calculate size and total file count
        let mut total_size: i64 = 0;
        let mut total_files: i64 = 0;
        let mut current_size: i64 = 0;
        let mut current_files: i64 = 0;
        for file in selected_files.clone() {
            if let Ok(m) = metadata(file.path.to_string()) {
                if m.is_dir() {
                    for entry_res in WalkDir::new(file.path.to_string()).follow_links(false) {
                        if let Ok(entry) = entry_res {
                            total_size +=
                                entry.metadata().and_then(|m| Ok(m.len())).unwrap_or(0) as i64;
                            total_files += 1;
                        }
                    }
                } else if !m.is_symlink() {
                    total_size += m.len() as i64;
                    total_files += 1;
                }
            }
        }
        if total_size == 0 || total_files == 0 {
            log_error_str("Error calculating the total size to delete. You will not know the status, but the operation should proceed.");
            return;
        }
        //Don't bother showing a progress window if the file is too small
        //Messages will just end up being sent nowhere/never read
        if total_size > PROGRESS_WINDOW_BYTE_THRESHOLD {
            show_progress_window(mw.clone(), recv, Duration::from_millis(100));
        }

        if progress
            .send((
                current_size as f32 / total_size as f32,
                format_size_and_filecount_progress_status(
                    current_size,
                    total_size,
                    current_files,
                    total_files,
                ),
                -1f64,
                false,
            ))
            .is_err()
        {
            return;
        }

        for file in selected_files {
            if file.is_dir {
                for entry_res in WalkDir::new(&*file.path).contents_first(true) {
                    if entry_res.is_err() {
                        log_error_str(&format!(
                            "File cannot be accessed. Skipping. Perhaps a permission issue? Error Text: {}",
                            entry_res.err().unwrap().to_string()));
                        continue;
                    }
                    let entry = entry_res.unwrap();

                    //Sub-item is a directory
                    if entry.path().is_dir() {
                        if let Err(e) = std::fs::remove_dir(entry.path()) {
                            log_error_str(&format!(
                                "{} could not be accessed. Error Text: {}",
                                entry.path().to_str().unwrap(),
                                e.to_string()
                            ));
                        }
                    } else {
                        if let Err(e) = std::fs::remove_file(entry.path()) {
                            log_error_str(&format!(
                                "{} could not be accessed. Error Text: {}",
                                entry.path().to_str().unwrap(),
                                e.to_string()
                            ));
                        } else {
                            if let Ok(meta) = entry.metadata() {
                                current_size += meta.len() as i64;
                            }
                            current_files += 1;
                            if progress
                                .send((
                                    current_size as f32 / total_size as f32,
                                    format_size_and_filecount_progress_status(
                                        current_size,
                                        total_size,
                                        current_files,
                                        total_files,
                                    ),
                                    -1f64,
                                    false,
                                ))
                                .is_err()
                            {
                                return;
                            }
                        }
                    }
                }
            } else {
                let metadata = metadata(&file.path.to_string());
                if let Err(e) = std::fs::remove_file(Path::new(&file.path.to_string())) {
                    log_error_str(&format!(
                        "{} could not be accessed. Error Text: {}",
                        &file.path,
                        e.to_string()
                    ));
                } else {
                    if let Ok(meta) = metadata {
                        current_size += meta.len() as i64;
                    }
                    current_files += 1;
                    if progress
                        .send((
                            current_size as f32 / total_size as f32,
                            format_size_and_filecount_progress_status(
                                current_size,
                                total_size,
                                current_files,
                                total_files,
                            ),
                            -1f64,
                            false,
                        ))
                        .is_err()
                    {
                        return;
                    }
                }
            };
        }
        //Those files should not be selected anymore
        //Since UI refreshes when this is done, we don't care
        //But when UI refresh gets overhauled, this needs to come back
        //selected_files_lock.drain();

        drop(selected_files_lock);

        //If we didn't show the progress window, we need to refresh manually
        if total_size <= PROGRESS_WINDOW_BYTE_THRESHOLD {
            mw.upgrade_in_event_loop(|w| {
                set_current_tab_file(
                    w.global::<TabsAdapter>().invoke_get_current_tab(),
                    Rc::new(w.as_weak()),
                    false,
                );
            })
            .ok();
        }
    });
}

/**
  Called when a file is dropped in the window
  The file is moved from its original location to the current folder
*/
pub fn move_file(mw: Rc<Weak<MainWindow>>, buf: &str, destination: &str) {
    if file_exists_in_dir(destination, buf) == Ok(false) {
        if let Some(filename) = buf.split("/").last() {
            if std::fs::copy(buf, destination.to_owned() + "/" + &filename).is_ok() {
                if std::fs::remove_file(buf).is_err() {
                    log_error_str(
                        "Could not remove the original file, so it was copied instead of moved.",
                    );
                }
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
