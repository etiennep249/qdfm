use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::SystemTime;

static DOUBLECLICK_TIMESTAMP: Lazy<Mutex<SystemTime>> =
    Lazy::new(|| Mutex::new(SystemTime::UNIX_EPOCH));
static DELAY: u64 = 500; /*ms*/

pub fn check_for_dclick() -> bool {
    let mut ts = DOUBLECLICK_TIMESTAMP.lock().unwrap();
    let now = SystemTime::now();
    if let Ok(since) = now.duration_since(*ts) {
        if (since.as_millis() as u64) < DELAY {
            *ts = SystemTime::now();
            return true;
        }
    }
    *ts = SystemTime::now();
    false
}
