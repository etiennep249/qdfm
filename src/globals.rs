use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard, OnceLock, TryLockError},
};

use crate::config::Config;
use crate::ui::*;
use sysinfo::{System, SystemExt};
use x11rb::{
    connect,
    rust_connection::{DefaultStream, RustConnection},
};

static SYSINFO: OnceLock<Mutex<System>> = OnceLock::new();
pub fn sysinfo_lock() -> MutexGuard<'static, System> {
    match SYSINFO.get_or_init(|| Mutex::new(System::new())).lock() {
        Ok(e) => e,
        Err(_) => {
            panic!("Could not get SYSINFO lock.");
        }
    }
}
static CONFIG: OnceLock<Mutex<Config>> = OnceLock::new();
pub fn config_lock() -> MutexGuard<'static, Config> {
    match CONFIG.get_or_init(|| Mutex::new(Config::new())).lock() {
        Ok(e) => e,
        Err(_) => {
            panic!("Could not get SYSINFO lock.");
        }
    }
}

//(index, file)
static SELECTED_FILES: OnceLock<Mutex<HashMap<i32, FileItem>>> = OnceLock::new();
pub fn selected_files_lock() -> MutexGuard<'static, HashMap<i32, FileItem>> {
    match SELECTED_FILES
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    {
        Ok(e) => e,
        Err(_) => {
            panic!("Could not get SELECTED_FILES lock.");
        }
    }
}
pub fn selected_files_try_lock() -> Result<
    MutexGuard<'static, HashMap<i32, FileItem>>,
    TryLockError<MutexGuard<'static, HashMap<i32, FileItem>>>,
> {
    SELECTED_FILES
        .get_or_init(|| Mutex::new(HashMap::new()))
        .try_lock()
}
///Returns the selected file if there is exactly one selected. Otherwise returns None
pub fn get_selected_file(
    selected_files: &MutexGuard<'static, HashMap<i32, FileItem>>,
) -> Option<FileItem> {
    if selected_files.len() != 1 {
        None
    } else {
        Some(selected_files.iter().next().unwrap().1.clone())
    }
}

//Only written to once during init sequence
static mut QDFM_WIN_ID: u32 = 0;
pub fn qdfm_win_id() -> u32 {
    unsafe { QDFM_WIN_ID }
}
pub fn set_qdfm_win_id(i: u32) {
    unsafe {
        QDFM_WIN_ID = i;
    }
}

static X_CONNECTION: OnceLock<Mutex<RustConnection<DefaultStream>>> = OnceLock::new();
pub fn x_conn_lock() -> MutexGuard<'static, RustConnection> {
    match X_CONNECTION
        .get_or_init(|| {
            Mutex::new(
                connect(None)
                    .expect(
                        "Cannot connect to X11. (Because check for wayland is not implemented yet)",
                    )
                    .0,
            )
        })
        .lock()
    {
        Ok(e) => e,
        Err(_) => {
            panic!("Could not get X_CONNECTION lock.");
        }
    }
}
