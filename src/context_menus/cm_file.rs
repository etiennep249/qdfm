use crate::{
    callbacks::{context_menu::ContextCallback, filemanager::set_current_tab_file},
    clipboard,
    core::run_command,
    enclose,
    file_properties::setup_properties,
    globals::config_lock,
    manage_open_with,
    ui::*,
    utils::error_handling::log_error_str,
};
use slint::{ComponentHandle, Image, LogicalPosition, Model, SharedPixelBuffer, VecModel, Weak};
use std::{path::Path, rc::Rc};

pub fn open_with_default(item: FileItem) {
    let conf = config_lock();
    if let Some(default) = conf.get_mapping_default(&item.extension) {
        if let Some(cmd) = conf
            .get_mappings_quick(&item.extension)
            .iter()
            .find(|m| m.display_name == *default)
        {
            run_command(&(cmd.command.to_string() + " " + &item.path));
        }
    }
}

/*
 *  Shows a secondary context menu on the right
 * */
pub fn open_with(file: FileItem, mw: Rc<Weak<MainWindow>>) {
    let w = mw.unwrap();
    let ctx_adapter = w.global::<ContextAdapter>();

    let mut menu: Vec<ContextItem> = Vec::new();

    //TODO: Get shortcuts from config file
    let conf = config_lock();

    let quick_mapping = conf.get_mappings_quick(&file.extension);

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
    menu.push(ContextItem {
        display: ("Manage").into(),
        callback_id: ContextCallback::ManageQuick as i32,
        shortcut: "".into(),
        icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
        has_separator: false,
        click_on_hover: false,
        internal_id: 0,
    });

    //Show the secondary menu where it needs to be
    let theme = w.global::<Theme>().get_current();
    ctx_adapter.set_secondary_items(Rc::new(VecModel::from(menu)).into());
    ctx_adapter.set_secondary_x_pos(ctx_adapter.get_x_pos() + 200.0);
    ctx_adapter.set_secondary_y_pos(
        ctx_adapter.get_y_pos()
            + (get_index(&ctx_adapter) as f32 * theme.context_menu_entry_height)
            + 1.0,
    );
    ctx_adapter.set_current_hover_callback_id(ContextCallback::OpenWith as i32);
    ctx_adapter.set_is_secondary_visible(true);
}

pub fn open_with_quick(context_item: &ContextItem, file: FileItem) {
    let conf = config_lock();
    let vec = conf.get_mappings_quick(&file.extension);
    run_command(&(vec[context_item.internal_id as usize].command.to_string() + " " + &file.path));
}

fn get_index(ctx_adapter: &ContextAdapter) -> i32 {
    ctx_adapter
        .get_items()
        .iter()
        .position(|f| f.callback_id == ContextCallback::OpenWith as i32)
        .unwrap() as i32
}

pub fn copy(item: FileItem) {
    clipboard::copy_file(item);
}
pub fn cut(item: FileItem) {
    clipboard::cut_file(item);
}
pub fn paste(path: &Path, mw: Rc<Weak<MainWindow>>) {
    clipboard::paste_file(path, mw);
}
pub fn delete(file: FileItem, mw: Rc<Weak<MainWindow>>) {
    let ret = if file.is_dir {
        std::fs::remove_dir_all(Path::new(&file.path.to_string()))
    } else {
        std::fs::remove_file(Path::new(&file.path.to_string()))
    };
    if ret.is_err() {
        log_error_str(&format!(
            "Could not delete \"{}\". Error Text: {}",
            file.path,
            ret.err().unwrap().to_string()
        ))
    }
    //Refresh UI
    set_current_tab_file(
        mw.unwrap().global::<TabsAdapter>().invoke_get_current_tab(),
        mw,
        false,
    );
}
pub fn show_properties(
    item: FileItem,
    mw: Rc<Weak<MainWindow>>,
    prop_win_rc: Weak<PropertiesWindow>,
) {
    /*
     *      Create the properties window centered on top of the other window.
     *      With a fixed position, this will be a floating window even on tiling WMs without hints.
     * */

    let main_win = mw.unwrap();
    let prop_win = prop_win_rc.unwrap();
    let pos = main_win.window().position();
    let x = pos.x as f32 + (main_win.get_win_width() / 2.0) - (prop_win.get_win_width() / 2.0);
    let y = pos.y as f32 + (main_win.get_win_height() / 2.0) - (prop_win.get_win_height() / 2.0);
    prop_win.window().set_position(LogicalPosition { x, y });
    setup_properties(item, prop_win.global::<PropertiesAdapter>(), prop_win_rc);
    prop_win.show().unwrap();
}

pub fn manage_quick(file: FileItem, mw: Rc<Weak<MainWindow>>) {
    let win = ManageOpenWithWindow::new().unwrap();

    let main_win = mw.unwrap();
    let pos = main_win.window().position();
    let x = pos.x as f32 + (main_win.get_win_width() / 2.0) - (win.get_win_width() / 2.0);
    let y = pos.y as f32 + (main_win.get_win_height() / 2.0) - (win.get_win_height() / 2.0);

    win.window().set_position(LogicalPosition { x, y });

    let adp = win.global::<ManageOpenWithAdapter>();
    let rc = Rc::new(win.as_weak());

    adp.set_extension(file.extension.clone().into());

    adp.on_ok(enclose! { (rc) move |ext| manage_open_with::ok(rc.clone(), ext)});
    adp.on_cancel(enclose! { (rc) move || manage_open_with::cancel(rc.clone())});
    let filename = file.path.clone();
    adp.on_open_with(
        enclose! { (rc) move |term| manage_open_with::open_with(rc.clone(), term, filename.clone())},
    );
    adp.on_set_default(move |ext, s| manage_open_with::set_default(ext, s));
    adp.on_remove_mapping(
        enclose! { (rc) move |i| manage_open_with::remove_mapping(rc.clone(), i as usize)},
    );
    adp.on_add_mapping(
        enclose! { (rc) move |mapping| manage_open_with::add_mapping(rc.clone(), mapping)},
    );

    manage_open_with::setup_manage_open_with(adp, file);

    //setup_properties(item, prop_win.global::<PropertiesAdapter>(), prop_win_rc);
    win.show().unwrap();
}
