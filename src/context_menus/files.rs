use crate::{
    callbacks::{
        context_menu::ContextCallback,
        filemanager::selection::{self, selected_files_read},
    },
    clipboard,
    core::run_command,
    enclose,
    file_properties::setup_properties,
    globals::config_read,
    manage_open_with,
    ui::{self, *},
};
use slint::{ComponentHandle, Image, LogicalPosition, Model, SharedPixelBuffer, VecModel, Weak};
use std::{path::PathBuf, rc::Rc};

pub fn open_with_default(files: Vec<FileItem>) {
    let conf = config_read();

    if let Some(default) = conf.get_mapping_default(&files[0].extension) {
        if let Some(cmd) = conf
            .get_mappings_quick(&files[0].extension)
            .iter()
            .find(|m| m.display_name == *default)
        {
            for f in files {
                run_command(&(cmd.command.to_string() + " " + &f.path));
            }
        }
    }
}

///Shows a secondary context menu on the right
pub fn open_with() {
    ui::run_with_main_window(|mw| {
        let ctx_adapter = mw.global::<ContextAdapter>();

        let mut menu: Vec<ContextItem> = Vec::new();

        //TODO: Get shortcuts from config file

        /* If all files in the selection have the same extension, then show the mappings for that
         * extension. Otherwise mappings are not displayed but the user can still choose a one-time
         * mapping to open all the selected files with.*/
        let conf = config_read();
        let extension = {
            let files = selected_files_read();
            let mut iter = files.iter();
            if files.len() == 1 {
                Some(iter.next().unwrap().1.extension.clone())
            } else if files.len() > 1 {
                let ext = iter.next().unwrap().1.extension.clone();
                if iter.all(|v| v.1.extension == ext) {
                    Some(ext)
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some(ext) = extension {
            let quick_mapping = conf.get_mappings_quick(&ext);
            for (i, mapping) in quick_mapping.iter().enumerate() {
                menu.push(ContextItem {
                    display: (&mapping.display_name).into(),
                    callback_id: ContextCallback::OpenWithQuick as i32,
                    shortcut: "".into(),
                    icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
                    has_separator: if i == quick_mapping.len() - 1 {
                        true
                    } else {
                        false
                    },
                    click_on_hover: false,
                    internal_id: i as i32,
                });
            }
        }
        menu.push(ContextItem {
            display: ("More").into(),
            callback_id: ContextCallback::ManageQuick as i32,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: false,
            click_on_hover: false,
            internal_id: 0,
        });

        //Show the secondary menu where it needs to be
        let theme = mw.global::<Theme>().get_current();
        ctx_adapter.set_secondary_items(Rc::new(VecModel::from(menu)).into());
        ctx_adapter.set_secondary_x_pos(ctx_adapter.get_x_pos() + theme.context_menu_width);
        ctx_adapter.set_secondary_y_pos(
            ctx_adapter.get_y_pos()
                + (get_index(&ctx_adapter) as f32 * theme.context_menu_entry_height)
                + 1.0,
        );
        ctx_adapter.set_current_hover_callback_id(ContextCallback::OpenWith as i32);
        ctx_adapter.set_is_secondary_visible(true);
    });
}

///Runs the command associated with the selected files extension
///Only call this if you are certain that every selected files have the same mapping
pub fn open_with_quick(context_item: &ContextItem) {
    let files = selected_files_read();
    let mut cmd = String::from("");
    let mut cmd_set = false;
    for (_, file) in files.iter() {
        if !cmd_set {
            cmd = config_read().get_mappings_quick(&file.extension)
                [context_item.internal_id as usize]
                .command
                .clone();
            cmd_set = true;
        }
        run_command(&(cmd.to_owned() + " " + &file.path));
    }
}
fn get_index(ctx_adapter: &ContextAdapter) -> i32 {
    ctx_adapter
        .get_items()
        .iter()
        .position(|f| f.callback_id == ContextCallback::OpenWith as i32)
        .unwrap() as i32
}

///See clipboard::copy
///Copied files are the selected ones
pub fn copy() {
    clipboard::copy::copy_file(selection::selected_files_clone(), false);
}
///See clipboard::cut
///Cut files are the selected ones
pub fn cut() {
    clipboard::cut::cut_file(selection::selected_files_clone());
}
///See clipboard::paste
pub fn paste(here: bool) {
    if !here {
        if let Some(f) = selection::get_selected_file() {
            clipboard::paste::paste_file(PathBuf::from(&(f.path.to_string())));
        }
    } else {
        //TODO:
    }
}
pub fn delete() {
    clipboard::delete::delete();
}
pub fn show_properties(prop_win_rc: Weak<PropertiesWindow>) {
    /*
     *      Create the properties window centered on top of the other window.
     *      With a fixed position, this will be a floating window even on tiling WMs without hints.
     * */

    ui::run_with_main_window(|main_win| {
        let prop_win = prop_win_rc.unwrap();
        let pos = main_win.window().position();
        let x = pos.x as f32 + (main_win.get_win_width() / 2.0) - (prop_win.get_win_width() / 2.0);
        let y =
            pos.y as f32 + (main_win.get_win_height() / 2.0) - (prop_win.get_win_height() / 2.0);
        prop_win.window().set_position(LogicalPosition { x, y });
        setup_properties(
            selection::selected_files_clone(),
            prop_win.global::<PropertiesAdapter>(),
            prop_win_rc,
        );
        prop_win.show().unwrap();
    });
}

pub fn manage_quick() {
    ui::run_with_main_window(|main_win| {
        let win = ManageOpenWithWindow::new().unwrap();
        let pos = main_win.window().position();
        let x = pos.x as f32 + (main_win.get_win_width() / 2.0) - (win.get_win_width() / 2.0);
        let y = pos.y as f32 + (main_win.get_win_height() / 2.0) - (win.get_win_height() / 2.0);

        win.window().set_position(LogicalPosition { x, y });

        let adp = win.global::<ManageOpenWithAdapter>();
        let rc = Rc::new(win.as_weak());

        let files: Vec<FileItem> = selection::selected_files_clone();

        //If they all have the same extension, then we can use that extension's mappings
        let extension = {
            let mut iter = files.iter();
            let ext = iter.next().unwrap().extension.clone();
            if iter.all(|f| f.extension == ext) {
                ext
            } else {
                "NOEXT".into()
            }
        };

        let files_rc = Rc::new(files);

        //Callbacks
        adp.set_extension(extension);
        adp.on_ok(enclose! { (rc) move |ext| manage_open_with::ok(rc.clone(), ext)});
        adp.on_cancel(enclose! { (rc) move || manage_open_with::cancel(rc.clone())});
        adp.on_open_with(
        enclose! { (rc, files_rc) move |term| manage_open_with::open_with(rc.clone(), term, files_rc.clone())},
    );
        adp.on_set_default(move |ext, s| manage_open_with::set_default(ext, s));
        adp.on_remove_mapping(
            enclose! { (rc) move |i| manage_open_with::remove_mapping(rc.clone(), i as usize)},
        );
        adp.on_add_mapping(
            enclose! { (rc) move |mapping| manage_open_with::add_mapping(rc.clone(), mapping)},
        );

        manage_open_with::setup_manage_open_with(adp, files_rc);

        win.show().unwrap();
    });
}
