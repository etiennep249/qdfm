use once_cell::sync::Lazy;
use std::sync::{Mutex, MutexGuard};

use sysinfo::{System, SystemExt};

use crate::tabs::Tab;

static SYSINFO: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new()));
pub fn sysinfo_lock() -> MutexGuard<'static, System> {
    match SYSINFO.lock() {
        Ok(e) => e,
        Err(_) => {
            panic!("Could not get SYSINFO lock.");
        }
    }
}

static TABS: Lazy<Mutex<Vec<Tab>>> = Lazy::new(|| {
    Mutex::new(vec![Tab {
        path: String::from("/"), // TODO: GET DEFAULT PATH OR LAST SAVed
    }])
});
pub fn tabs_lock() -> MutexGuard<'static, Vec<Tab>> {
    match TABS.lock() {
        Ok(e) => e,
        Err(_) => {
            panic!("Could not get SYSINFO lock.");
        }
    }
}
