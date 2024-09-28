use serde::{Deserialize, Serialize};
use serde_json::Error;
use slint::VecModel;

use crate::ui::*;
use std::{collections::HashMap, str::FromStr};

use crate::utils::error_handling::log_error_str;

pub struct Config {
    map: HashMap<String, String>,
    extension_mappings_default: Option<HashMap<String, Mapping>>,
    extension_mappings_quick: Option<HashMap<String, Vec<Mapping>>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Mapping {
    pub display_name: String,
    //icon
    pub command: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            map: Config::default_config(),
            extension_mappings_default: None,
            extension_mappings_quick: None,
        }
    }
    //TODO: use json everywhere
    fn default_config() -> HashMap<String, String> {
        HashMap::from([
            (String::from("max_nav_history"), String::from("6")),
            (String::from("default_path"), String::from("/")),
            (String::from("theme"), String::from("dark")),
            (
                //<name>:<width_percent>:<0/1/2 not_selected/ascending/descending>
                String::from("headers"),
                String::from("name:70:1,size:15:0,date:20:0"),
            ),
            (String::from("default_sort"), String::from("name")),
            (
                String::from("extension_mappings_default"),
                String::from(
                    r#"{
                        "sh": {"display_name": "Bash", "command": "/usr/local/bin/st /bin/bash"},
                        "txt": { "display_name" : "Neovim", "command": "/usr/local/bin/st /usr/bin/nvim"}
                    }"#,
                ),
            ),
            (
                String::from("extension_mappings_quick"),
                String::from(
                    r#"{
                        "sh":[
                            {"display_name": "Bash", "command": "/usr/local/bin/st /bin/bash"},
                            {"display_name": "Sh", "command": "/usr/local/bin/st /bin/sh"}
                        ], 
                        "txt":[
                            {"display_name": "Neovim", "command": "/usr/local/bin/st /bin/nvim"},
                            {"display_name": "Nano", "command": "/usr/local/bin/st /bin/nano"}
                        ]
                    }"#,
                ),
            ),
        ])
    }
    //"Safe" to unwrap
    pub fn get<T: FromStr>(&self, k: &str) -> Option<T> {
        //TODO: REMOVE MUT
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
    pub fn get_mapping_default(&self, extension: &str) -> Option<&Mapping> {
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
    pub fn init_mappings(&mut self) -> bool {
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

    pub fn init_mappings_default(&mut self) -> Result<(), Error> {
        let config_string: String = self.get("extension_mappings_default").unwrap();
        let json: HashMap<String, Mapping> = serde_json::from_str(&config_string)?;

        self.extension_mappings_default = Some(json);
        Ok(())
    }
    pub fn init_mappings_quick(&mut self) -> Result<(), Error> {
        let config_string: String = self.get("extension_mappings_quick").unwrap();
        let json: HashMap<String, Vec<Mapping>> = serde_json::from_str(&config_string)?;

        self.extension_mappings_quick = Some(json);
        Ok(())
    }
}
