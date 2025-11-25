use std::path::PathBuf;

use crate::{
    callbacks::{context_menu::ContextCallback, filemanager::selection::get_selected_path},
    core::{create_file, verify_file},
    ui::{self, main_window::run_with_main_window, ContextItem, CreateNewAdapter, CreateNewFile},
    utils::center_window_on_another,
};
use slint::{ComponentHandle, Image, SharedPixelBuffer};

use super::secondary_context_menu::show_secondary_context_menu;

///This opens up a window to create a new file
///The window is created every time and not cached.
pub fn create_new_file() {
    run_with_main_window(|mw| {
        if let Some(path) = get_selected_path() {
            let win = CreateNewFile::new().unwrap();
            win.window().set_position(center_window_on_another(
                mw.window().position(),
                mw.get_win_width(),
                mw.get_win_height(),
                win.get_win_width(),
                win.get_win_height(),
            ));
            let adp = win.global::<CreateNewAdapter>();
            prepare_create_new(&adp, &path);

            let cancel_weak = win.as_weak();
            let ok_weak = win.as_weak();
            let edited_weak = win.as_weak();

            adp.on_cancel(move || {
                if let Some(win) = cancel_weak.upgrade() {
                    win.hide().ok();
                }
            });

            //On Ok, do not show errors, warnings should be shown in on_edited.
            //This is also triggered by accepting the lineedit (pressing enter)
            adp.on_ok(move || {
                if let Some(win) = ok_weak.upgrade() {
                    let adp = win.global::<CreateNewAdapter>();
                    if verify_file(&adp.get_path_to_directory(), &adp.get_name()).is_none() {
                        let mut path = PathBuf::from(&adp.get_path_to_directory());
                        path.push(&adp.get_name());
                        create_file(path);
                        ui::send_message(ui::UIMessage::Refresh);
                        win.hide().ok();
                    }
                }
            });
            //On edited, issue a warning if the file already exists
            adp.on_edited(move |s| {
                if let Some(win) = edited_weak.upgrade() {
                    let adp = win.global::<CreateNewAdapter>();
                    if let Some(warning) = verify_file(&adp.get_path_to_directory(), &s) {
                        adp.set_warning(warning.into());
                    } else {
                        adp.set_warning("".into());
                    }
                }
            });

            win.show().unwrap();
        }
    });
}

///This opens up a window to create a new directory
pub fn create_new_dir() {
    if let Some(path) = get_selected_path() {}
}

///This opens up a window to create a new symlink
pub fn create_new_link() {
    if let Some(path) = get_selected_path() {}
}

fn prepare_create_new(adp: &CreateNewAdapter, path: &String) {
    adp.set_path_to_directory(path.into());
    adp.set_warning("".into());
    adp.set_link_target("".into());
    adp.set_name("".into());
}

///Triggered when hovering over "create new" in the context menu
pub fn create_new_hover(parent_index: i32) {
    let mut menu: Vec<ContextItem> = Vec::new();

    menu.push(ContextItem {
        display: ("File").into(),
        callback_id: ContextCallback::CreateNewFile as i32,
        shortcut: "".into(),
        icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
        has_separator: false,
        click_on_hover: false,
        internal_id: 0,
    });
    menu.push(ContextItem {
        display: ("Directory").into(),
        callback_id: ContextCallback::CreateNewDirectory as i32,
        shortcut: "".into(),
        icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
        has_separator: false,
        click_on_hover: false,
        internal_id: 0,
    });
    menu.push(ContextItem {
        display: ("Symlink").into(),
        callback_id: ContextCallback::CreateNewLink as i32,
        shortcut: "".into(),
        icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
        has_separator: false,
        click_on_hover: false,
        internal_id: 0,
    });

    show_secondary_context_menu(menu, parent_index);
}
