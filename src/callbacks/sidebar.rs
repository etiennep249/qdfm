use crate::ui::{self, *};

use super::filemanager;

pub fn sidebar_item_clicked(item: SidebarItem) {
    ui::send_message(UIMessage::SetCurrentTabFile(
        TabItem {
            internal_path: item.internal_path,
            text: item.text.clone(),
            text_length: item.text.len() as i32,
            selected: true,
        },
        true,
    ));
}

pub fn left_arrow_clicked() {
    match filemanager::get_prev_history() {
        Some(e) => ui::send_message(UIMessage::SetCurrentTabFile(e, false)),
        None => (),
    }
}
pub fn right_arrow_clicked() {
    match filemanager::get_next_history() {
        Some(e) => ui::send_message(UIMessage::SetCurrentTabFile(e, false)),
        None => (),
    }
}
