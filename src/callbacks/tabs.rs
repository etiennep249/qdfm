use crate::ui::*;

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
