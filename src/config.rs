use slint::VecModel;

use crate::ui::*;
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
            (
                //<name>:<width_percent>:<0/1/2 not_selected/ascending/descending>
                String::from("headers"),
                String::from("name:70:1,size:15:0,date:20:0"),
            ),
            (String::from("default_sort"), String::from("name")),
        ])
    }
    //"Safe" to unwrap
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
}
