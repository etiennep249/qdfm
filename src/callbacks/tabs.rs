use crate::ui::*;

use slint::Weak;
use std::rc::Rc;

pub fn breadcrumb_clicked(item: TabItem, mw: Rc<Weak<MainWindow>>) {
    println!("clicked!: {}", item.text);
}
