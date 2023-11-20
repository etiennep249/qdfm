use once_cell::sync::Lazy;
use std::sync::{Mutex, MutexGuard};

use sysinfo::{System, SystemExt};

use crate::config::Config;

static SYSINFO: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new()));
pub fn sysinfo_lock() -> MutexGuard<'static, System> {
    match SYSINFO.lock() {
        Ok(e) => e,
        Err(_) => {
            panic!("Could not get SYSINFO lock.");
        }
    }
}
static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| Mutex::new(Config::new()));
pub fn config_lock() -> MutexGuard<'static, Config> {
    match CONFIG.lock() {
        Ok(e) => e,
        Err(_) => {
            panic!("Could not get SYSINFO lock.");
        }
    }
}
