use crate::context_menus::context_items::{get_ci, get_ci_capacity};
use crate::globals::config_read;
use crate::ui::*;
use crate::{context_menus as cm, ui};
use slint::{ComponentHandle, VecModel, Weak};
use std::rc::Rc;

use super::filemanager::selection::{self};

pub enum ContextCallback {
    OpenWithDefault,
    OpenWith,
    OpenWithQuick,
    ManageQuick,
    ShowProperties,
    Cut,
    Copy,
    PasteIntoSelected,
    PasteHere,
    Delete,
}

///Triggered when a certain menu item is clicked.
///Redirects the call to the proper callback based on the callback_id of the context item.
///It also hides the context menu afterwards.
pub fn menuitem_click(context_item: ContextItem, prop_win: Weak<PropertiesWindow>) {
    match context_item.callback_id {
        c if c == ContextCallback::ShowProperties as i32 => cm::files::show_properties(prop_win),
        c if c == ContextCallback::OpenWithDefault as i32 => {
            cm::files::open_with_default(selection::selected_files_clone())
        }
        c if c == ContextCallback::OpenWith as i32 => cm::files::open_with(),
        c if c == ContextCallback::Copy as i32 => cm::files::copy(),
        c if c == ContextCallback::Cut as i32 => cm::files::cut(),
        c if c == ContextCallback::PasteIntoSelected as i32 => cm::files::paste(false),
        /*c if c == ContextCallback::PasteHere as i32 => {
            cm_file::paste(
                true, /*Path::new(&(file.path.to_string())).parent().unwrap(),*/ mw,
            )
        }*/
        c if c == ContextCallback::Delete as i32 => cm::files::delete(),
        c if c == ContextCallback::OpenWithQuick as i32 => {
            cm::files::open_with_quick(&context_item)
        }
        c if c == ContextCallback::ManageQuick as i32 => cm::files::manage_quick(),
        _ => (),
    }
    if !context_item.click_on_hover {
        ui::send_message(UIMessage::HideContextMenu);
    }
}

///Code to run when any context item in a context menu is hovered.
pub fn menuitem_hover(context_item: ContextItem) {
    ui::run_with_main_window(move |mw| {
        let ctx_adapter = mw.global::<ContextAdapter>();
        if context_item.callback_id != ctx_adapter.get_current_hover_callback_id() {
            ctx_adapter.set_is_secondary_visible(false);
        }
    });
}

///Builds the context menu for the selected files at (x,y) and shows it.
pub fn show_context_menu(x: f32, y: f32) {
    ui::run_with_main_window(move |mw| {
        let mut menu: Vec<ContextItem> = Vec::with_capacity(get_ci_capacity());

        let conf = config_read();

        let default_mapping =
            selection::get_common_extension().and_then(|f| conf.get_mapping_default(&f));

        //Only offer 'open with' if we have mappings for the file's extension
        if default_mapping.is_some() {
            let mut open_with_default = get_ci("open_with_default");
            open_with_default.display =
                ("Open With ".to_owned() + &default_mapping.unwrap()).into();
            menu.push(open_with_default);
            menu.push(get_ci("open_with"));
        }

        let is_nothing_selected = selection::is_nothing_selected();

        if !is_nothing_selected {
            menu.push(get_ci("cut"));
            menu.push(get_ci("copy"));
        }
        if selection::is_single_selected_directory() {
            menu.push(get_ci("paste_into"));
        } else {
            menu.push(get_ci("paste_here"));
        }
        if !is_nothing_selected {
            menu.push(get_ci("delete"));
        }
        menu.push(get_ci("properties"));

        let ctx_adapter = mw.global::<ContextAdapter>();
        ctx_adapter.set_items(Rc::new(VecModel::from(menu)).into());
        ctx_adapter.set_x_pos(x + 1f32);
        ctx_adapter.set_y_pos(y + 1f32);
    });
}
