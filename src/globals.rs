use std::sync::{Mutex, MutexGuard, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::config::Config;
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
static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();
pub fn config_read() -> RwLockReadGuard<'static, Config> {
    match CONFIG.get_or_init(|| RwLock::new(Config::new())).read() {
        Ok(e) => e,
        Err(_) => {
            panic!("Could not get SYSINFO lock.");
        }
    }
}
pub fn config_write() -> RwLockWriteGuard<'static, Config> {
    match CONFIG.get_or_init(|| RwLock::new(Config::new())).write() {
        Ok(e) => e,
        Err(_) => {
            panic!("Could not get SYSINFO lock.");
        }
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
