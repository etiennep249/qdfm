use slint::platform::Key;

///Returns the internal representation (in unicode string, as of Slint 1.14) of a given key
///Used to translate from a human readable config format into an actual key Slint recognizes
///
///Basically just a static Map (compiler-generated jump table)
///Returns None if there is seemingly no valid key for the given string.
///Might be a good idea to alert the user that their config won't work.
pub fn get_key(key: &str) -> Option<char> {
    Some(
        match key {
            "up" => Key::UpArrow,
            "down" => Key::DownArrow,
            "left" => Key::LeftArrow,
            "right" => Key::RightArrow,
            "backspace" => Key::Backspace,
            "tab" => Key::Tab,
            "enter" => Key::Return,
            "escape" => Key::Escape,
            "backtab" => Key::Backtab,
            "delete" => Key::Delete,
            "capslock" => Key::CapsLock,
            "space" => Key::Space,
            "f1" => Key::F1,
            "f2" => Key::F2,
            "f3" => Key::F3,
            "f4" => Key::F4,
            "f5" => Key::F5,
            "f6" => Key::F6,
            "f7" => Key::F7,
            "f8" => Key::F8,
            "f9" => Key::F9,
            "f10" => Key::F10,
            "f11" => Key::F11,
            "f12" => Key::F12,
            "insert" => Key::Insert,
            "home" => Key::Home,
            "end" => Key::End,
            "pageup" => Key::PageUp,
            "pagedown" => Key::PageDown,
            "scrolllock" => Key::ScrollLock,
            "pause" => Key::Pause,
            "sysreq" => Key::SysReq,
            "stop" => Key::Stop,
            "menu" => Key::Menu,
            _ => {
                //If the key is one character long, then we assume it's one of the letters
                if key.len() == 1 {
                    return key.chars().next();
                } else {
                    return None;
                }
            }
        }
        .into(),
    )
}
