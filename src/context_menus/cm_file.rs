use crate::ui::*;
use slint::{Image, SharedPixelBuffer, SharedString, Weak};
use std::rc::Rc;

pub fn open_with_default(item: FileItem, mw: Rc<Weak<MainWindow>>) {
    println!("File clicked: {}", item.path);
}

pub fn show_properties(item: FileItem, mw: Rc<Weak<MainWindow>>) {
    println!("File clicked: {}", item.path);
}
