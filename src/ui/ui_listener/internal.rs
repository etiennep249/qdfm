use std::rc::Rc;

use slint::{Model, VecModel};

use crate::{
    callbacks::{
        filemanager::{add_to_history, selection},
        tabs::get_breadcrumbs_for,
    },
    sort::call_current_sort,
    ui::*,
};

///This function is used to set what directory the current tab is showing.
///It can also be used to refresh it.
pub fn set_current_tab_file(mut item: Option<TabItem>, remember: bool, mw: &MainWindow) {
    //If no tab item is provided, assume a refresh
    if item.is_none() {
        item = Some(mw.global::<TabsAdapter>().invoke_get_current_tab());
    }
    let mut item = item.unwrap();
    let files = crate::core::generate_files_for_path(item.internal_path.as_str());
    if item.internal_path == "/" {
        item.text = "/".into();
    }

    let tabs = mw.global::<TabsAdapter>();
    tabs.set_path_shown(false);
    if remember {
        add_to_history(tabs.invoke_get_current_tab());
    }

    tabs.set_breadcrumbs(Rc::new(VecModel::from(get_breadcrumbs_for(&item))).into());
    tabs.invoke_set_current_tab(item.clone());
    let filemanager = mw.global::<FileManager>();
    filemanager.set_files_len(files.len() as i32);
    filemanager.set_files(Rc::new(VecModel::from(files)).into());
    call_current_sort(&mw);
    selection::clear_selection();

    //If the current path is also a drive in the sidebar, make it appear selected
    let sidebar_items = mw.global::<SidebarItems>();
    sidebar_items.set_selected_drive(-1);
    for (i, drive) in sidebar_items.get_drive_list().iter().enumerate() {
        if drive.internal_path == item.internal_path {
            sidebar_items.set_selected_drive(i as i32);
        }
    }
}

pub fn refresh_ui(mw: &MainWindow) {
    set_current_tab_file(None, false, mw);
}
