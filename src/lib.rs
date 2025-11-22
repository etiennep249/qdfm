pub mod ui {
    pub mod main_window;
    pub mod prop_window;
    pub mod ui_listener;
    pub use ui_listener::*;
    slint::include_modules!();
}

//TODO: why pub
pub mod callbacks;
pub mod clipboard;
pub mod config;
pub mod context_menus;
pub mod core;
pub mod drives;
pub mod file_properties;
pub mod globals;
pub mod keybinds;
pub mod manage_open_with;
pub mod progress_window;
mod rename_window;
pub mod sort;
pub mod utils;

//https://github.com/rust-lang/rfcs/issues/2407#issuecomment-385291238
//Replace with https://github.com/rust-lang/rfcs/pull/3512
//When/if it gets merged
#[macro_export]
macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

#[cfg(test)]
pub mod tests;
