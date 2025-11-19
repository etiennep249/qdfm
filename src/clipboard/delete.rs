use crate::{
    callbacks::filemanager::selection::selected_files_read,
    progress_window::show_progress_window,
    ui::{self},
    utils::error_handling::log_error_str,
};
use std::{fs::metadata, path::Path, sync::mpsc::channel, thread, time::Duration};
use walkdir::WalkDir;

use super::{format_size_and_filecount_progress_status, PROGRESS_WINDOW_BYTE_THRESHOLD};

pub fn delete() {
    let thread = thread::spawn(move || {
        let (progress, recv) = channel();
        let selected_files_lock = selected_files_read();
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
            show_progress_window(recv, Duration::from_millis(100));
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
                    if entry.path().is_dir() && !entry.path_is_symlink() {
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
            ui::send_message(ui::UIMessage::Refresh);
        }
    });
    #[cfg(test)]
    thread.join().unwrap();
}
