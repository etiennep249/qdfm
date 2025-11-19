use arboard::Clipboard;
use std::{
    collections::VecDeque,
    fs::metadata,
    path::PathBuf,
    str::FromStr,
    sync::{
        mpsc::{channel, Sender},
        Mutex,
    },
    thread,
    time::Duration,
};
use walkdir::WalkDir;

use crate::{
    progress_window::show_progress_window,
    rename_window::{self, setup_rename_window, RenameOption},
    ui::{self},
    utils::error_handling::log_error_str,
};

use super::{
    copy::copy_single_file_operation, format_size_progress_status, CUT_BUFFER, ESTIMATE_CAPACITY,
    PROGRESS_WINDOW_BYTE_THRESHOLD,
};

///Pastes the selected file(s) in to_path.
///Done in another thread.
//TODO: Implement copy ourselves so we can have progress info for large files
pub fn paste_file(to_path: PathBuf) {
    let _thread = thread::spawn(move || {
        let mut is_cut = false;
        let mut paths: Vec<PathBuf> = Vec::new();
        //Check if we are doing cut/paste
        {
            if let Ok(mut buf) = CUT_BUFFER.get_or_init(|| Mutex::new(Vec::new())).lock() {
                if buf.len() > 0 {
                    paths = buf
                        .iter()
                        .map(|file| PathBuf::from_str(&file.path).unwrap())
                        .collect();
                    *buf = Vec::new();
                    is_cut = true;
                    //It's a CUT-PASTE operation.
                    //Set the paths and clear the buffer
                }
            } else {
                log_error_str("Cut buffer could not be accessed.");
                return;
            }
        }

        //If it's a COPY-PASTE operation, get the content from the clipboard

        if !is_cut {
            if let Ok(mut clipboard) = Clipboard::new() {
                if let Ok(text) = clipboard.get_text() {
                    if !is_cut && !text.starts_with("file:///") {
                        //Content is not a file, move on
                        return;
                    }

                    paths = text
                        .split("\n")
                        .map(|s| PathBuf::from_str(&s.replace("file://", "")).unwrap())
                        .collect();

                    //Since the last path is going to contain a newline but not a separating one,
                    //Remove it from the list of paths
                    paths.remove(paths.len() - 1);
                } else {
                    log_error_str("Couldn't get clipboard text");
                    return;
                }
            } else {
                log_error_str("Could not find a clipboard.");
                return;
            }
        }

        //---------------------------- Setup Progress Window -------------------------------------
        //Start a progress window
        let (progress, recv) = channel();

        //We have to check if the receiver is dead (if the transfer was canceled) everytime we send
        if progress
            .send((0.0, "Calculating...".into(), -1f64, true))
            .is_err()
        {
            return;
        }

        let mut all_success = true;
        if progress
            .send((0.0, "Calculating status.".into(), -1f64, true))
            .is_err()
        {
            return;
        }

        let rename_win = setup_rename_window();

        //--------------------------------Calculate paste size ------------------------------------
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
            } else {
                log_error_str("No metadata");
            }
        }
        if total == 0 {
            log_error_str("Error calculating the total size to paste. You will not know the status, but the operation should proceed.");
            return;
        }

        //Don't bother showing a progress window if the file is too small
        //Messages will just end up being sent nowhere/never read
        if total > PROGRESS_WINDOW_BYTE_THRESHOLD as u64 {
            show_progress_window(recv, Duration::from_millis(100));
        }
        let total = total as i64;

        let mut current: i64 = 0;
        let mut remaining_time: f64 = -1f64;

        let mut speed_vec: VecDeque<f64> = VecDeque::with_capacity(ESTIMATE_CAPACITY);
        let mut avg_speed: f64 = -1f64;

        if progress
            .send((0.0, format_size_progress_status(0, total), -1f64, true))
            .is_err()
        {
            return;
        }

        //------------------------------------Paste Operation --------------------------------------
        let mut apply_to_all = false;
        let mut apply_to_all_option = RenameOption::Rename;

        //Loop over every directory/file/simlink in the paste and copy them over.
        //It mainly just calls paste_<folder/symlink/file>_with_checks on everything
        //While handling some overwrite information
        for path in paths.iter() {
            let base_dir_path = path.parent().unwrap().to_string_lossy().to_string();
            let new_path = &to_path.join(
                path.to_string_lossy()
                    .strip_prefix(&(base_dir_path.clone() + "/"))
                    .unwrap()
                    .to_string(),
            );
            if is_cut {
                //TODO: RENAME WHOLE FOLDER INSTEAD
                return;
            }
            if path.is_dir() && !path.is_symlink() {
                //These paths get changed throughout the iteration to preserve new names
                let mut walkdir_path = to_path.clone();
                let mut walkdir_depth = 0;

                let mut walkdir_ignore_prefix = String::from("IMPOSSIBLEPREFIX");
                let mut walkdir_overwrite_prefix = String::from("IMPOSSIBLEPREFIX");

                for entry_res in WalkDir::new(&path) {
                    if entry_res.is_err() {
                        log_error_str(&format!(
                                    "File cannot be accessed. Skipping. Perhaps a permission issue? Error Text: {}",
                                    entry_res.err().unwrap().to_string()));
                        all_success = false;
                        continue;
                    }
                    let entry = entry_res.unwrap();

                    //If this went back some levels, pop walkdir_path
                    let went_up_by: i32 = walkdir_depth - entry.depth() as i32;
                    if went_up_by > 0 {
                        for _ in 0..went_up_by {
                            walkdir_path.pop();
                        }
                        walkdir_depth = entry.depth() as i32;
                    }

                    if entry.path().starts_with(&walkdir_ignore_prefix) {
                        continue;
                    }

                    let mut new_path = &mut walkdir_path.join(entry.file_name());
                    if entry.path().is_dir() && !entry.path_is_symlink() {
                        if let Ok(overwrite) = paste_folder_with_checks(
                            &entry.path().to_owned(),
                            &mut new_path,
                            total,
                            &mut current,
                            &mut speed_vec,
                            &mut avg_speed,
                            &mut remaining_time,
                            &mut all_success,
                            &progress,
                            &rename_win,
                            &mut apply_to_all,
                            &mut apply_to_all_option,
                            entry.path().starts_with(&walkdir_overwrite_prefix),
                        ) {
                            if overwrite {
                                //Overwrite all children
                                walkdir_overwrite_prefix = entry.path().to_str().unwrap().into();
                                log_error_str(&format!(
                                    "Set overwrite to: {}",
                                    walkdir_overwrite_prefix,
                                ));
                            }
                        } else {
                            //Ignore all children
                            walkdir_ignore_prefix = entry.path().to_str().unwrap().into();
                        }

                        //Update names
                        walkdir_path = new_path.clone();
                        walkdir_depth = entry.depth() as i32 + 1;

                        //Update names
                    } else if entry.path().is_file() || entry.path_is_symlink() {
                        paste_file_with_checks(
                            &entry.path().to_owned(),
                            &new_path,
                            total,
                            &mut current,
                            &mut speed_vec,
                            &mut avg_speed,
                            &mut remaining_time,
                            &mut all_success,
                            &progress,
                            &rename_win,
                            &mut apply_to_all,
                            &mut apply_to_all_option,
                            entry.path().starts_with(&walkdir_overwrite_prefix),
                            is_cut,
                        );
                    }
                }
            } else if path.is_file() || path.is_symlink() {
                paste_file_with_checks(
                    path,
                    &new_path,
                    total,
                    &mut current,
                    &mut speed_vec,
                    &mut avg_speed,
                    &mut remaining_time,
                    &mut all_success,
                    &progress,
                    &rename_win,
                    &mut apply_to_all,
                    &mut apply_to_all_option,
                    false,
                    is_cut,
                );
            }
        }

        //------------------------------------Paste Operation END---------------------------------

        //If this was a Cut/Paste operation, delete the original (unless this was a partial success)
        //TODO: if all_success == false, still delete the original for everything except the failed
        //TODO: BIG TODO IF A FILE IS IGNORED IT SHOULD NOT BE DELETED

        //If we didn't show the progress window, we need to refresh manually
        if total <= PROGRESS_WINDOW_BYTE_THRESHOLD {
            ui::send_message(ui::UIMessage::Refresh);
        }
    });
    #[cfg(test)]
    _thread.join().unwrap();
}
//Returns Err if the file could not be copied and should not be deleted in a cut/paste
pub fn paste_file_with_checks(
    path: &PathBuf,
    to_path: &PathBuf,
    total: i64,
    current: &mut i64,
    speed_vec: &mut VecDeque<f64>,
    avg_speed: &mut f64,
    remaining_time: &mut f64,
    all_success: &mut bool,
    progress: &Sender<(f32, String, f64, bool)>,
    rename_win: &rename_window::RenameWindow,
    apply_to_all: &mut bool,
    apply_to_all_option: &mut RenameOption,
    overwrite: bool, //Will still overwrite if apply_to_all
    is_rename: bool, //If the file should be moved instead of copied
) {
    let mut to_path = to_path.clone();
    if !overwrite && to_path.exists() {
        let mut option = RenameOption::Rename;
        if *apply_to_all && *apply_to_all_option != RenameOption::Rename {
            option = apply_to_all_option.clone();
        } else {
            if let Ok(ret) =
                rename_win.show_rename_window(path.file_name().unwrap().to_str().unwrap().into())
            {
                if ret.apply_to_all {
                    *apply_to_all = true;
                    *apply_to_all_option = ret.option.clone();
                } else {
                    *apply_to_all = false;
                }
                option = ret.option;
                if option == RenameOption::Rename {
                    to_path.set_file_name(ret.filename.unwrap());
                }
            } else {
                return;
            }
        }
        match option {
            RenameOption::Ignore => {
                //Ignore, so don't copy
                //TODO: IF A FILE IS IGNORED IT SHOULD NOT GET DELETED (CUT PASTE)
                return;
            }
            RenameOption::Rename => {
                //Will never happen in an apply_to_all, so is already taken care of
            }
            RenameOption::Overwrite => {
                //Continue with the operation
            }
        }
    }
    if copy_single_file_operation(
        to_path,
        &path,
        current,
        speed_vec,
        avg_speed,
        remaining_time,
        total,
        all_success,
        progress,
        is_rename,
    )
    .is_err()
    {
        *all_success = false;
        return;
    }
}
//Returns Err if the subfolder items should NOT BE PASTED
//The returned value is true if the folder content should be overwritten
pub fn paste_folder_with_checks(
    path: &PathBuf,
    to_path: &mut PathBuf,
    _total: i64,
    _current: &mut i64,
    _speed_vec: &mut VecDeque<f64>,
    _avg_speed: &mut f64,
    _remaining_time: &mut f64,
    all_success: &mut bool,
    _progress: &Sender<(f32, String, f64, bool)>,
    rename_win: &rename_window::RenameWindow,
    apply_to_all: &mut bool,
    apply_to_all_option: &mut RenameOption,
    overwrite: bool,
) -> Result<bool, ()> {
    if overwrite {
        //Folder already exists so just do nothing
        return Ok(true);
    }
    if to_path.exists() {
        let mut option = RenameOption::Rename;
        if *apply_to_all && *apply_to_all_option != RenameOption::Rename {
            option = apply_to_all_option.clone();
        } else {
            if let Ok(ret) =
                rename_win.show_rename_window(path.file_name().unwrap().to_str().unwrap().into())
            {
                if ret.apply_to_all {
                    *apply_to_all = true;
                    *apply_to_all_option = ret.option.clone();
                } else {
                    *apply_to_all = false;
                }
                option = ret.option;
                if option == RenameOption::Rename {
                    to_path.set_file_name(ret.filename.unwrap());
                }
            } else {
                return Err(());
            }
        }
        match option {
            RenameOption::Ignore => {
                //Ignore, so don't copy
                return Err(());
            }
            RenameOption::Rename => {
                //Will never happen in an apply_to_all, so is already taken care of
            }
            RenameOption::Overwrite => {
                return Ok(true);
                //Continue with the operation
            }
        }
    }
    if std::fs::create_dir(to_path).is_err() {
        *all_success = false;
        return Err(());
    }

    Ok(false)
}
