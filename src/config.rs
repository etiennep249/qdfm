use serde::{Deserialize, Serialize};
use serde_json::Error;
use slint::VecModel;

use crate::{
    keybinds::keybind::{get_keybind, KeyBind},
    ui::*,
};
use std::{collections::HashMap, str::FromStr};

use crate::utils::error_handling::log_error_str;

pub struct Config {
    map: HashMap<&'static str, String>,
    extension_mappings_default: Option<HashMap<String, String>>,
    extension_mappings_quick: Option<HashMap<String, Vec<Mapping>>>,
    pub keybinds: Option<HashMap<KeyBind, String>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Mapping {
    pub display_name: String,
    //icon
    pub command: String,
}

impl Config {
    pub fn new() -> Self {
        let mut ret = Self {
            map: Config::default_config(),
            extension_mappings_default: None,
            extension_mappings_quick: None,
            keybinds: None,
        };
        ret.init();
        ret
    }
    pub fn init(&mut self) {
        self.init_mappings();
        self.init_keybinds();
    }
    //TODO: use json everywhere
    fn default_config() -> HashMap<&'static str, String> {
        HashMap::from([
            ("max_nav_history", "6".into()),
            ("default_path", "/".into()),
            ("terminal", "st".into()),
            ("theme", "dark".into()),
            (
                //<name>:<width_percent>:<0/1/2 not_selected/ascending/descending>
                "headers",
                "name:70:1,size:15:0,date:20:0".into(),
            ),
            ("default_sort", "name".into()),
            (
                "extension_mappings_default",
                r#"{
                        "sh": "Bash",
                        "txt": "Neovim"
                    }"#
                .into(),
            ),
            (
                "extension_mappings_quick",
                r#"{
                        "sh":[
                            {"display_name": "Bash", "command": "/usr/local/bin/st /bin/bash"},
                            {"display_name": "Sh", "command": "/usr/local/bin/st /bin/sh"}
                        ], 
                        "txt":[
                            {"display_name": "Neovim", "command": "/usr/local/bin/st /bin/nvim"},
                            {"display_name": "Nano", "command": "/usr/local/bin/st /bin/nano"}
                        ]
                    }"#
                .into(),
            ),
            //Multiple keybinds for the same feature is allowed
            (
                "keybinds",
                r#"{
                        "ctrl a": "select_all",
                        "up": "select_up",
                        "down": "select_down",
                        "shift down": "shift_select_down",
                        "shift up": "shift_select_up",
                        "enter": "enter",
                        "ctrl c": "copy",
                        "ctrl v": "paste",
                        "ctrl x": "cut",
                        "alt enter": "properties",
                        "delete": "delete"
                    }"#
                .into(),
            ),
        ])
    }
    //"Safe" to unwrap, error will have been logged.
    pub fn get<T: FromStr>(&self, k: &str) -> Option<T> {
        let res = self.map.get(k).unwrap().parse::<T>();
        if res.is_err() {
            log_error_str(&("Invalid configuration for key ".to_owned() + k));
            None
        } else {
            Some(res.ok().unwrap())
        }
    }
    pub fn get_headers(&self) -> VecModel<Header> {
        let headers_string: String = self.get("headers").unwrap();
        let headers_vec: Vec<Header> = headers_string
            .split(",")
            .map(|s| {
                let mut iter = s.split(":");
                let name = iter.next().unwrap();
                let pct = iter.next().unwrap().parse::<f32>().unwrap();
                let sort = iter.next().unwrap().parse::<i32>().unwrap();

                match name {
                    "name" => Header {
                        inner_value: 0,
                        display: "Name".into(),
                        width_pct: pct,
                        alignment: 0,
                        sort,
                    },
                    "size" => Header {
                        inner_value: 1,
                        display: "Size".into(),
                        width_pct: pct,
                        alignment: 2,
                        sort,
                    },
                    "date" => Header {
                        inner_value: 2,
                        display: "Date".into(),
                        width_pct: pct,
                        alignment: 0,
                        sort,
                    },
                    _ => panic!("Could not parse headers from configuration"),
                }
            })
            .collect();
        let headers_vecmodel = VecModel::default();
        headers_vecmodel.set_vec(headers_vec);
        headers_vecmodel
    }

    /*
     *  These two functions retrieve the extension mappings from the configuration
     * */
    pub fn get_mapping_default(&self, extension: &str) -> Option<&String> {
        self.extension_mappings_default
            .as_ref()
            .unwrap()
            .get(extension)
    }
    pub fn get_mappings_quick(&self, extension: &str) -> Vec<Mapping> {
        self.extension_mappings_quick
            .as_ref()
            .unwrap()
            .get(extension)
            .unwrap_or(&Vec::new())
            .to_vec()
    }
    fn init_mappings(&mut self) -> bool {
        let mut ret = true;
        if let Err(e) = self.init_mappings_default() {
            log_error_str(&format!("Could not parse 'extension_mappings_default' from configuration. Please fix it and restart. Error: {}", e.to_string()));
            ret = false;
        }
        if let Err(e) = self.init_mappings_quick() {
            log_error_str(&format!("Could not parse 'extension_mappings_quick' from configuration. Please fix it and restart. Error: {}", e.to_string()));
            ret = false;
        }
        ret
    }

    fn init_mappings_default(&mut self) -> Result<(), Error> {
        let config_string: String = self.get("extension_mappings_default").unwrap();
        let json: HashMap<String, String> = serde_json::from_str(&config_string)?;

        self.extension_mappings_default = Some(json);
        Ok(())
    }
    fn init_mappings_quick(&mut self) -> Result<(), Error> {
        let config_string: String = self.get("extension_mappings_quick").unwrap();
        let json: HashMap<String, Vec<Mapping>> = serde_json::from_str(&config_string)?;

        self.extension_mappings_quick = Some(json);
        Ok(())
    }

    pub fn set_default_for(&mut self, ext: &str, name: &str) {
        if let Some(ref mut mappings) = self.extension_mappings_default {
            if let Some(value) = mappings.get_mut(ext) {
                *value = name.into();
            } else {
                mappings.insert(ext.to_string(), name.into());
            }
        }
    }

    pub fn set_mappings_quick(&mut self, ext: &str, in_vec: Vec<Mapping>) {
        if let Some(ref mut mappings) = self.extension_mappings_quick {
            if let Some(vec) = mappings.get_mut(ext) {
                *vec = in_vec;
            } else {
                mappings.insert(ext.to_string(), in_vec);
            }
        }
    }
    ///Returns a string representation of what to do when a given keybind is pressed
    ///This is used to check if a particular key combination being pressed has a keybind,
    ///and if so, what to do.
    pub fn get_keybind_function(&self, keybind: KeyBind) -> Option<&String> {
        if let Some(ref keybinds) = self.keybinds {
            keybinds.get(&keybind)
        } else {
            None
        }
    }

    ///This function parses the json keybinds into a hashmap
    ///that can easily and quickly be indexed into
    pub fn init_keybinds(&mut self) {
        let config_string: String = self.get("keybinds").unwrap();
        let Ok(json) = serde_json::from_str::<HashMap<String, String>>(&config_string) else {
            log_error_str("Failed to parse keybinds JSON from config. 
                Try an online JSON parser to verify your syntax and watch for trailing commas, which are not allowed.");
            return;
        };

        let parsed_keybinds = json
            .into_iter()
            .filter_map(|(k, v)| get_keybind(&k).map(|key| (key, v)))
            .collect();

        self.keybinds = Some(parsed_keybinds);
    }
}
