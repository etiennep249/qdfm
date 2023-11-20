use crate::ui::*;
use slint::SharedString;
use slint::Weak;

use super::filemanager::set_current_tab_file;

pub fn sidebar_item_clicked(item: SidebarItem, mw: Weak<MainWindow>) {
    set_current_tab_file(
        FileItem {
            file_name: SharedString::default(),
            path: item.internal_path,
        },
        mw,
    )
}
