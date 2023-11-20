use crate::ui::*;
use slint::Weak;
use std::rc::Rc;

use super::filemanager;

pub fn sidebar_item_clicked(item: SidebarItem, mw: Rc<Weak<MainWindow>>) {
    filemanager::set_current_tab_file(
        TabItem {
            internal_path: item.internal_path,
            text: item.text.clone(),
            text_length: item.text.len() as i32,
        },
        mw,
        true,
    )
}

pub fn left_arrow_clicked(mw: Rc<Weak<MainWindow>>) {
    match filemanager::get_prev_history(mw.clone()) {
        Some(e) => filemanager::set_current_tab_file(e, mw, false),
        None => (),
    }
}
pub fn right_arrow_clicked(mw: Rc<Weak<MainWindow>>) {
    match filemanager::get_next_history(mw.clone()) {
        Some(e) => filemanager::set_current_tab_file(e, mw, false),
        None => (),
    }
}
