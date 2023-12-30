use crate::{
    ui::*,
    utils::{error_handling::user_notice, is_directory_valid},
};

use slint::{SharedString, Weak};
use std::rc::Rc;

use super::filemanager::set_current_tab_file;

pub fn breadcrumb_clicked(item: TabItem, mw: Rc<Weak<MainWindow>>) {
    set_current_tab_file(item, mw, true);
}

pub fn get_breadcrumbs_for(item: &TabItem) -> Vec<TabItem> {
    let mut s = String::from("/");
    item.internal_path
        .strip_prefix("/")
        .unwrap()
        .split("/")
        .map(|x| {
            let r = TabItem {
                internal_path: SharedString::from(s.clone() + x),
                text: x.into(),
                selected: true,
                text_length: -1,
            };
            s += x;
            s += "/";
            r
        })
        .collect()
}

pub fn breadcrumb_accepted(mut s: SharedString, mw: Rc<Weak<MainWindow>>) {
    if !is_directory_valid(&s) {
        user_notice("Invalid path!");
        return;
    }
    s = match s.strip_suffix("/") {
        Some(s) => s.into(),
        None => s,
    };
    let item = TabItem {
        text: s.rsplit("/").next().unwrap().into(),
        internal_path: s,
        selected: true,
        text_length: -1,
    };
    set_current_tab_file(item, mw, true);
}
