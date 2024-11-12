use std::io::Error;
pub fn log_error(err: Error) { /*TODO*/
}
pub fn log_error_str(msg: &str) {
    println!("{}", msg);
}
pub fn user_notice(msg: &str) {
    println!("{}", msg);
}
pub fn log_debug(msg: &str) {}
