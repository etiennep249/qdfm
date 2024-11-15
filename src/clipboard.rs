use crate::{
    callbacks::filemanager::set_current_tab_file, ui::*, utils::error_handling::log_error_str,
};
use arboard::{Clipboard, SetExtLinux};
use slint::Weak;
use std::{
    fs::read_link,
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
    sync::{Mutex, OnceLock},
};
use walkdir::WalkDir;

///Contains the files to delete after the paste
static CUT_BUFFER: OnceLock<Mutex<Vec<FileItem>>> = OnceLock::new();

pub fn copy_file(files: Vec<FileItem>) {
    std::thread::spawn(move || {
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
    let buf = CUT_BUFFER.get_or_init(|| Mutex::new(Vec::new())).lock();

    if let Ok(mut buf_lock) = buf {
        *buf_lock = files.clone();
        copy_file(files);
    } else {
        log_error_str("Could not get the cut buffer. The operation has been canceled.");
    }
}

//TODO: progress bar and thread
pub fn paste_file(to_path: &Path, mw: Rc<Weak<MainWindow>>) {
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

    for path in paths.iter() {
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
            let mut dir = String::new();
            //Loop over every file we have to copy
            for entry_res in WalkDir::new(&path) {
                if entry_res.is_err() {
                    //Not sure what could cause this, but do not interrupt everything for one bad file.
                    //Do not copy the original if this happens however.
                    log_error_str(&format!(
                    "File cannot be accessed. Skipping. Perhaps a permission issue? Error Text: {}",
                    entry_res.err().unwrap().to_string(),
                ));
                    all_success = false;
                    continue;
                }
                let entry = entry_res.unwrap();

                //Sub-item is a directory
                if entry.path().is_dir() {
                    dir = entry
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
                    let to = to_path.join(&dir).join(entry.file_name());
                    let is_err = if entry.file_type().is_symlink() {
                        std::os::unix::fs::symlink(&read_link(entry.path()).unwrap(), &to).is_err()
                    } else {
                        std::fs::copy(entry.path(), to).is_err()
                    };
                    if is_err {
                        log_error_str(&format!(
                            "File could not be pasted: {}",
                            entry.path().to_string_lossy()
                        ));
                        all_success = false;
                    }
                }
            }
        //We are copying a file
        } else if path.is_file() {
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

            //Copy the file to the new destination, or create a simlink if needed
            let to = to_path.join(path.file_name().unwrap());
            let is_err = if path.is_symlink() {
                std::os::unix::fs::symlink(read_link(path).unwrap(), &to).is_err()
            } else {
                std::fs::copy(path, to).is_err()
            };
            if is_err {
                log_error_str(&format!(
                    "File could not be copied: {}",
                    path.to_string_lossy()
                ));
                all_success = false;
            }
        }
    }

    //If this was a Cut/Paste operation, delete the original (unless this was a partial success)
    if buf_lock.len() > 0 {
        if !all_success {
            log_error_str(
                "Not all operations succeeded, the original file(s) and/or folder(s) have not been deleted during the Cut/Paste operation.",
            );
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
    //Refresh UI
    set_current_tab_file(
        mw.unwrap().global::<TabsAdapter>().invoke_get_current_tab(),
        mw,
        false,
    );
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
