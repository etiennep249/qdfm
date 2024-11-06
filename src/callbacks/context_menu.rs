use crate::context_menus::cm_file;
use crate::ui::*;
use slint::{ComponentHandle, Weak};
use std::path::Path;
use std::rc::Rc;

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

pub fn menuitem_click(
    file: FileItem,
    context_item: ContextItem,
    mw: Rc<Weak<MainWindow>>,
    prop_win: Weak<PropertiesWindow>,
) {
    let mw_clone_do_not_pass = mw.clone();
    match context_item.callback_id {
        c if c == ContextCallback::ShowProperties as i32 => {
            cm_file::show_properties(file, mw, prop_win)
        }
        c if c == ContextCallback::OpenWithDefault as i32 => cm_file::open_with_default(file),
        c if c == ContextCallback::OpenWith as i32 => cm_file::open_with(file, mw),
        c if c == ContextCallback::Copy as i32 => cm_file::copy(file),
        c if c == ContextCallback::Cut as i32 => cm_file::cut(file),
        c if c == ContextCallback::PasteIntoSelected as i32 => {
            cm_file::paste(Path::new(&(file.path.to_string())), mw)
        }
        c if c == ContextCallback::PasteHere as i32 => {
            cm_file::paste(Path::new(&(file.path.to_string())).parent().unwrap(), mw)
        }
        c if c == ContextCallback::Delete as i32 => cm_file::delete(file, mw),
        c if c == ContextCallback::OpenWithQuick as i32 => {
            cm_file::open_with_quick(&context_item, file)
        }
        c if c == ContextCallback::ManageQuick as i32 => cm_file::manage_quick(file, mw),
        _ => (),
    }
    if !context_item.click_on_hover {
        mw_clone_do_not_pass.unwrap().invoke_hide_context_menu();
    }
}
pub fn menuitem_hover(context_item: ContextItem, mw: Rc<Weak<MainWindow>>) {
    let w = mw.unwrap();
    let ctx_adapter = w.global::<ContextAdapter>();
    if context_item.callback_id != ctx_adapter.get_current_hover_callback_id() {
        ctx_adapter.set_is_secondary_visible(false);
    }
}
