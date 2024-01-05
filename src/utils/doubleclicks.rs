use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::Instant;

//(Time, index)
static DOUBLECLICK_TIMESTAMP: Lazy<Mutex<(Instant, i32)>> =
    Lazy::new(|| Mutex::new((Instant::now(), -1)));
static DELAY: u64 = 500; /*ms*/

//TODO: Latest version of slint supposedly supports double clicks, possibly use theirs when merged
pub fn check_for_dclick(index: i32) -> bool {
    let mut guard = DOUBLECLICK_TIMESTAMP.lock().unwrap();
    let ts = guard.0;
    let idx = guard.1;
    if index != idx {
        //Make sure the double click happened on the same file
        *guard = (Instant::now(), index);
        false
    } else if (ts.elapsed().as_millis() as u64).le(&DELAY) {
        *guard = (Instant::now(), idx);
        true
    } else {
        *guard = (Instant::now(), index);
        false
    }
}
