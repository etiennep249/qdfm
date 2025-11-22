use std::{collections::HashSet, hash::Hash};

use i_slint_core::items::{KeyEvent, KeyboardModifiers};

use crate::{
    globals::config_read,
    utils::{capitalize_first, error_handling::log_error_str},
};

use super::{keybind_callbacks::call_keybind_callback, keys::get_key};

pub struct KeyBind {
    _key: char,
    _modifiers: KeyboardModifiers,
    original_string: String, //The config string
    internal_id: u32,        //For hashmap indexing
}

impl KeyBind {
    pub fn new(key: char, modifiers: KeyboardModifiers, original_string: String) -> KeyBind {
        Self {
            internal_id: ((Self::flatten_modifiers(&modifiers) as u32) << 16) | (key as u32),
            _key: key,
            _modifiers: modifiers,
            original_string,
        }
    }
    fn flatten_modifiers(mods: &KeyboardModifiers) -> u8 {
        (mods.alt as u8) << 0
            | (mods.control as u8) << 1
            | (mods.shift as u8) << 2
            | (mods.meta as u8) << 3
    }
}

impl Hash for KeyBind {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.internal_id.hash(state);
    }
}

impl PartialEq for KeyBind {
    fn eq(&self, other: &Self) -> bool {
        self.internal_id == other.internal_id
    }
}
impl Eq for KeyBind {}

static FORMAT_PRIORITY: [&str; 4] = ["Ctrl", "Shift", "Alt", "Meta"];
///This function formats the original string into a cleaner verison fit for
///display, eg. in the context menu. Not super fast, should not run often.
pub fn format_keybind(function: &str) -> String {
    //TODO: Don't loop every time...
    let mut kb = None;
    let conf = config_read();
    let keybinds = conf.keybinds.as_ref().unwrap();
    for (k, v) in keybinds {
        if v == function {
            kb = Some(k);
            break;
        }
    }
    let Some(kb) = kb else { return String::new() };
    let source = kb.original_string.to_lowercase();
    //Hashset of unique modifiers, in lowercase, unordered.
    let words: HashSet<&str> = source.split_whitespace().collect();
    let mut words_vec: Vec<String> = words.into_iter().map(|s| capitalize_first(s)).collect();
    words_vec.sort_by_key(|s| {
        FORMAT_PRIORITY
            .iter()
            .position(|p| p == s)
            .unwrap_or(usize::MAX)
    });
    words_vec.join("+")
}

///Looks up the keybind in the configuration.
///If it exists and has an associated callback, it is called.
///Otherwise nothing happens. An error will be logged in case
///a keybind does exist for the given key presses but the callback is invalid.
///
///Returns true if the key led to something, false if it did not have a keybind.
pub fn use_keybind(key: KeyEvent) -> bool {
    let conf = config_read();

    let keybind_function = conf.get_keybind_function(KeyBind::new(
        key.text.chars().next().unwrap(),
        key.modifiers,
        "".into(),
    ));
    if let Some(callback) = keybind_function {
        call_keybind_callback(&callback);
        return true;
    } else {
        //The given keypress has no callback, do nothing.
        return false;
    }
}

///Turns a string from the config into a keybind, if possible.
///The format is this: {Modifiers, separated by spaces, optional} {Key}
///The possible modifiers are: ctrl, alt, meta, shift
///The possible keys are those seen in keybinds/keys.rs plus standard letters
///
///[Examples]
///ctrl alt delete
///ctrl a
///f11
///shift home
///...
pub fn get_keybind(str: &str) -> Option<KeyBind> {
    let mut modifiers = KeyboardModifiers {
        alt: false,
        control: false,
        meta: false,
        shift: false,
    };
    let mut key = '\0';
    for s in str.split_whitespace() {
        match s {
            "ctrl" => {
                modifiers.control = true;
            }
            "alt" => {
                modifiers.alt = true;
            }
            "meta" => {
                modifiers.meta = true;
            }
            "shift" => {
                modifiers.shift = true;
            }
            _ => {
                if let Some(k) = get_key(s) {
                    key = k;
                } else {
                    log_error_str(&format!("Invalid key in keybinds: {}", s));
                    return None;
                }
            }
        }
    }

    if key != '\0' {
        Some(KeyBind::new(key, modifiers, str.to_string()))
    } else {
        None
    }
}
