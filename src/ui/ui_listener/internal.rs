use std::rc::Rc;

use slint::VecModel;

use crate::{
    callbacks::{
        filemanager::{add_to_history, selection},
        tabs::get_breadcrumbs_for,
    },
    sort::call_current_sort,
    ui::*,
};

//TODO: Better refresh. Perhaps a queue? Don't want UI to abruptly refresh when background
//operations finish. That or make this function non-distruptive, maintain selected files.
pub fn set_current_tab_file(mut item: Option<TabItem>, remember: bool, mw: MainWindow) {
    //If no tab item is provided, assume a refresh
    if item.is_none() {
        item = Some(mw.global::<TabsAdapter>().invoke_get_current_tab());
    }
    let mut item = item.unwrap();
    let files = crate::core::generate_files_for_path(item.internal_path.as_str());
    if item.internal_path == "/" {
        item.text = "/".into();
    }
    let files_len = files.len();

    let tabs = mw.global::<TabsAdapter>();
    tabs.set_path_shown(false);
    if remember {
        add_to_history(tabs.invoke_get_current_tab());
    }

    tabs.set_breadcrumbs(Rc::new(VecModel::from(get_breadcrumbs_for(&item))).into());
    tabs.invoke_set_current_tab(item);
    let filemanager = mw.global::<FileManager>();
    filemanager.set_files(Rc::new(VecModel::from(files)).into());
    call_current_sort(&mw);
    selection::clear_selection();
    selection::init_selected_visual(&mw, files_len);
}

pub fn refresh_ui(mw: MainWindow) {
    set_current_tab_file(None, false, mw);
}
