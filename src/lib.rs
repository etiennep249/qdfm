pub mod ui {
    slint::include_modules!();
}
pub mod callbacks;
pub mod clipboard;
pub mod config;
pub mod context_menus;
pub mod core;
pub mod drives;
pub mod file_properties;
pub mod globals;
pub mod key_events;
pub mod sort;
pub mod utils;

#[cfg(test)]
pub mod tests;
