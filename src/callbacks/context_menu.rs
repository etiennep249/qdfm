use crate::globals::config_lock;
use crate::ui::*;
use crate::{context_menus as cm, ui};
use slint::{ComponentHandle, Image, SharedPixelBuffer, VecModel, Weak};
use std::rc::Rc;

use super::filemanager::selection;

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

pub fn show_context_menu(x: f32, y: f32) {
    //TODO: have all of these items stored somewhere so we dont genereate everytime
    //Also don't do all that in the main thread. Or maybe it doesn't matter anyway since we don't
    //need the UI to update in the split second before the context menu shows
    ui::run_with_main_window(move |mw| {
        let mut menu: Vec<ContextItem> = Vec::new();

        let conf = config_lock();

        let default_mapping =
            selection::get_common_extension().and_then(|f| conf.get_mapping_default(&f));

        if default_mapping.is_some() {
            menu.push(ContextItem {
                display: ("Open With ".to_owned() + &default_mapping.unwrap()).into(),
                callback_id: ContextCallback::OpenWithDefault as i32,
                shortcut: "".into(),
                icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
                has_separator: true,
                click_on_hover: false,
                internal_id: 0,
            });
            menu.push(ContextItem {
                display: ("Open With").into(),
                callback_id: ContextCallback::OpenWith as i32,
                shortcut: "▶".into(),
                icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
                has_separator: true,
                click_on_hover: true,
                internal_id: 0,
            });
        }

        menu.push(ContextItem {
            display: "Cut".into(),
            callback_id: ContextCallback::Cut as i32,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: false,
            click_on_hover: false,
            internal_id: 0,
        });
        menu.push(ContextItem {
            display: "Copy".into(),
            callback_id: ContextCallback::Copy as i32,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: false,
            click_on_hover: false,
            internal_id: 0,
        });
        if selection::is_single_selected_directory() {
            menu.push(ContextItem {
                display: "Paste Into".into(),
                callback_id: ContextCallback::PasteIntoSelected as i32,
                shortcut: "".into(),
                icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
                has_separator: true,
                click_on_hover: false,
                internal_id: 0,
            });
        } else {
            menu.push(ContextItem {
                display: "Paste Here".into(),
                callback_id: ContextCallback::PasteHere as i32,
                shortcut: "".into(),
                icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
                has_separator: true,
                click_on_hover: false,
                internal_id: 0,
            });
        }
        menu.push(ContextItem {
            display: "Delete".into(),
            callback_id: ContextCallback::Delete as i32,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: true,
            click_on_hover: false,
            internal_id: 0,
        });
        menu.push(ContextItem {
            display: "Properties".into(),
            callback_id: ContextCallback::ShowProperties as i32,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: false,
            click_on_hover: false,
            internal_id: 0,
        });

        let ctx_adapter = mw.global::<ContextAdapter>();
        ctx_adapter.set_items(Rc::new(VecModel::from(menu)).into());
        ctx_adapter.set_x_pos(x + 1f32);
        ctx_adapter.set_y_pos(y + 1f32);
    });
}
