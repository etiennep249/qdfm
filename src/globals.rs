use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::config::Config;
use sysinfo::{System, SystemExt};

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
