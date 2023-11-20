use std::{collections::HashMap, str::FromStr};

use crate::utils::error_handling::log_error_str;

pub struct Config {
    map: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            map: Config::default_config(),
        }
    }
    fn default_config() -> HashMap<String, String> {
        HashMap::from([
            (String::from("max_nav_history"), String::from("6")),
            (String::from("default_path"), String::from("/")),
            (String::from("theme"), String::from("dark")),
        ])
    }
    pub fn get<T: FromStr>(&self, k: &str) -> Option<T> {
        let res = self.map.get(k).unwrap().parse::<T>();
        if res.is_err() {
            log_error_str("Invalid configuration for key ".to_owned() + k);
            None
        } else {
            Some(res.ok().unwrap())
        }
    }
}
