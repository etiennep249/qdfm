use once_cell::sync::Lazy;
use std::sync::{Mutex, MutexGuard};

use sysinfo::{System, SystemExt};

static SYSINFO: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new()));
pub fn sysinfo_lock() -> MutexGuard<'static, System> {
    match SYSINFO.lock() {
        Ok(e) => e,
        Err(_) => {
            panic!("Could not get SYSINFO lock.");
        }
    }
}
