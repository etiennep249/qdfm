use crate::context_menus::cm_file;
use crate::ui::*;
use slint::Weak;
use std::path::Path;
use std::rc::Rc;

pub enum ContextCallback {
    OpenWithDefault,
    ShowProperties,
    Cut,
    Copy,
    PasteIntoSelected,
    PasteHere,
    Delete,
}

pub fn menuitem_click(
    item: FileItem,
    context_item: ContextItem,
    mw: Rc<Weak<MainWindow>>,
    prop_win: Weak<PropertiesWindow>,
) {
    let mw_clone_do_not_pass = mw.clone();
    match context_item.callback_id {
        c if c == ContextCallback::ShowProperties as i32 => {
            cm_file::show_properties(item, mw, prop_win)
        }
        c if c == ContextCallback::OpenWithDefault as i32 => cm_file::open_with_default(item, mw),
        c if c == ContextCallback::Copy as i32 => cm_file::copy(item),
        c if c == ContextCallback::Cut as i32 => cm_file::cut(item),
        c if c == ContextCallback::PasteIntoSelected as i32 => {
            cm_file::paste(Path::new(&(item.path.to_string())), mw)
        }
        c if c == ContextCallback::PasteHere as i32 => {
            cm_file::paste(Path::new(&(item.path.to_string())).parent().unwrap(), mw)
        }
        c if c == ContextCallback::Delete as i32 => cm_file::delete(item, mw),
        _ => (),
    }
    mw_clone_do_not_pass.unwrap().invoke_hide_context_menu();
}
