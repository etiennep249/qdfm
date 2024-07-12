pub mod error_handling;
pub mod types;

//Returns true if s is a valid directory
pub fn is_directory_valid(s: &str) -> bool {
    let metadata = std::fs::metadata(s);
    metadata.is_ok() && metadata.unwrap().is_dir()
}
