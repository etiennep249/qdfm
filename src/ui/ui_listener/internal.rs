use std::{rc::Rc, sync::RwLock};

use main_window::SELECTED_TABITEM;
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
    if item.internal_path == "/" {
        item.text = "/".into();
    }

    let tabs = mw.global::<TabsAdapter>();
    tabs.set_path_shown(false);

    //History
    if remember {
        add_to_history(tabs.invoke_get_current_tab());
    }

    //Breadcrumbs
    tabs.set_breadcrumbs(Rc::new(VecModel::from(get_breadcrumbs_for(&item))).into());

    //Set items both in the UI and in rust
    tabs.invoke_set_current_tab(item.clone());
    let mut rust_tabitem = SELECTED_TABITEM
        .get_or_init(|| RwLock::new(None))
        .write()
        .unwrap();
    *rust_tabitem = Some(item.clone());

    //Set files
    let filemanager = mw.global::<FileManager>();
    let files = crate::core::generate_files_for_path(item.internal_path.as_str());
    filemanager.set_files_len(files.len() as i32);
    filemanager.set_files(Rc::new(VecModel::from(files)).into());

    //Sort
    call_current_sort(&mw);

    //Selection
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

///Refreshes the UI (immediately)
pub fn refresh_ui(mw: &MainWindow) {
    set_current_tab_file(None, false, mw);
}
