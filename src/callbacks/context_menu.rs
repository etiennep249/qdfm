use crate::context_menus::cm_file;
use crate::ui::*;
use slint::Weak;
use std::rc::Rc;

pub enum ContextCallback {
    OpenWithDefault,
    ShowProperties,
}

pub fn menuitem_click(
    item: FileItem,
    context_item: ContextItem,
    mw: Rc<Weak<MainWindow>>,
    prop_win: Weak<PropertiesWindow>,
) {
    let mw_clone = mw.clone();
    match context_item.callback_id {
        c if c == ContextCallback::ShowProperties as i32 => {
            cm_file::show_properties(item, mw, prop_win)
        }
        c if c == ContextCallback::OpenWithDefault as i32 => cm_file::open_with_default(item, mw),
        _ => (),
    }
    mw_clone.unwrap().invoke_hide_context_menu();
}
