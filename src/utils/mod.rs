use std::{fs::File, io::Read};

pub mod drag_and_drop;
pub mod error_handling;
pub mod file_picker;
pub mod types;

//Returns true if s is a valid directory
pub fn is_directory_valid(s: &str) -> bool {
    let metadata = std::fs::metadata(s);
    metadata.is_ok() && metadata.unwrap().is_dir()
}

pub fn rand() -> u64 {
    let file = File::open("/dev/urandom");
    if file.is_err() {
        return 0;
    }
    let mut buf = [0; 8];
    let numbers = file.unwrap().read_exact(&mut buf);
    if numbers.is_err() {
        0
    } else {
        u64::from_be_bytes(buf)
    }
}

pub fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
