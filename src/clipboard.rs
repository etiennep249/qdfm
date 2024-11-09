use crate::{
    callbacks::filemanager::set_current_tab_file, ui::*, utils::error_handling::log_error_str,
};
use arboard::{Clipboard, SetExtLinux};
use slint::Weak;
use std::{
    fs::read_link,
    path::Path,
    rc::Rc,
    sync::{Mutex, OnceLock},
};
use walkdir::WalkDir;

static CUT_BUFFER: OnceLock<Mutex<String>> = OnceLock::new();

pub fn copy_file(file: FileItem) {
    std::thread::spawn(move || {
        if let Ok(mut clip) = Clipboard::new() {
            let ret = clip.set().wait().text(format!("file://{}", file.path));
            if ret.is_err() {
                log_error_str("Could not set the clipboard text");
            }
        } else {
            log_error_str("Could not find a clipboard.");
        }
    });
}

pub fn cut_file(file: FileItem) {
    let buf = CUT_BUFFER.get_or_init(|| Mutex::new(String::new())).lock();

    if let Ok(mut buf_lock) = buf {
        *buf_lock = String::from(&file.path);
        drop(buf_lock);
    }

    copy_file(file);
}

//TODO: progress bar
pub fn paste_file(path: &Path, mw: Rc<Weak<MainWindow>>) {
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
    let text = text.replace("file://", "");
    let from = Path::new(&text);
    if from.file_name().is_none() {
        log_error_str("Invalid filename");
        return;
    }

    let file_exists = file_exists_in_dir(&text, from.file_name().unwrap().to_str().unwrap());

    if file_exists.is_err() {
        return;
    } else if file_exists.unwrap() == true {
        //TODO
        log_error_str("already exists TODO prompt for rename");
        return;
    }

    let buf = CUT_BUFFER.get_or_init(|| Mutex::new(String::new())).lock();
    if buf.is_err() {
        log_error_str("Cut buffer could not be accessed.");
        return;
    }
    //If this gets set to false, we don't delete the original in case of a Cut/Paste
    let mut all_success = true;
    let buf_lock = buf.unwrap();

    // We are copying a directory
    if from.is_dir() {
        let base_dir_path = from.parent().unwrap().to_string_lossy().to_string();
        let mut dir = String::new();
        //Loop over every file we have to copy
        for entry_res in WalkDir::new(&from) {
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
                    std::os::unix::fs::symlink(read_link(entry.path()).unwrap(), path.join(&dir))
                        .is_err()
                } else {
                    std::fs::create_dir_all(path.join(&dir)).is_err()
                };
                if is_err {
                    log_error_str("Could not create the directory. Canceling operations. Some files may already have been copied.");
                    return;
                }
            //Sub-item is a file
            } else {
                let to = path.join(&dir).join(entry.file_name());
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
    } else if from.is_file() {
        //Copy the file to the new destination, or create a simlink if needed
        let to = path.join(from.file_name().unwrap());
        let is_err = if from.is_symlink() {
            std::os::unix::fs::symlink(read_link(from).unwrap(), &to).is_err()
        } else {
            std::fs::copy(from, to).is_err()
        };
        if is_err {
            log_error_str(&format!(
                "File could not be copied: {}",
                from.to_string_lossy()
            ));
            all_success = false;
        }
    }

    //If this was a Cut/Paste operation, delete the original (unless this was a partial success)
    if *buf_lock == from.to_string_lossy() {
        if !all_success {
            log_error_str(
                "Not all operations succeeded, the original file/folder has not been deleted during the Cut/Paste operation.",
            );
        } else {
            if from.is_dir() {
                if std::fs::remove_dir_all(from).is_err() {
                    log_error_str(
                        "Source directory could not be removed during Cut/Paste operation.",
                    );
                }
            } else {
                if std::fs::remove_file(from).is_err() {
                    log_error_str("Source file could not be removed during Cut/Paste operation.");
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

//TODO BIG TODO implement drag and drop from qdfm to some other window ON WAYLAND
