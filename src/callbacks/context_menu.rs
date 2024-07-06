use crate::ui::*;
use slint::Weak;
use std::rc::Rc;

pub fn menuitem_click(item: FileItem, context_item: ContextItem, mw: Rc<Weak<MainWindow>>) {
    println!("File clicked: {}", item.path);
}
