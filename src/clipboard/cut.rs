use super::{copy::copy_file, CUT_BUFFER};
use crate::{ui::FileItem, utils::error_handling::log_error_str};
use std::{sync::Mutex, thread};

//TODO: Add a couple more spots where the CUT buffer gets cleared so we don't accidentally cut
//something we cut an hour ago
pub fn cut_file(files: Vec<FileItem>) {
    let thread = thread::spawn(|| {
        let buf = CUT_BUFFER.get_or_init(|| Mutex::new(Vec::new())).lock();

        if let Ok(mut buf_lock) = buf {
            *buf_lock = files.clone();
            copy_file(files, true);
        } else {
            log_error_str("Could not get the cut buffer. The operation has been canceled.");
        }
    });
    #[cfg(test)]
    thread.join().unwrap();
}
