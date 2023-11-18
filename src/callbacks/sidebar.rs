use crate::{globals::tabs_lock, tabs::get_current_tab_idx, ui::*};

pub fn sidebar_item_clicked(item: SidebarItem) {
    let mut tabs = tabs_lock();
    let tab = &mut tabs[get_current_tab_idx()];
    tab.path = String::from("TEST");
}
