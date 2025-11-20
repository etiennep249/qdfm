use i_slint_core::items::KeyEvent;
use keybind::use_keybind;

pub mod keybind;
mod keybind_callbacks;
pub mod keys;

///Needs to return false when the key is not handled
pub fn handle_key_press(key: KeyEvent) -> bool {
    use_keybind(key)
}
