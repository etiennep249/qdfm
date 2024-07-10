use crate::{file_properties::setup_properties, ui::*};
use slint::{ComponentHandle, LogicalPosition, Weak};
use std::rc::Rc;

pub fn open_with_default(item: FileItem, mw: Rc<Weak<MainWindow>>) {
    println!("File clicked: {}", item.path);
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
