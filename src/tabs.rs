use crate::globals::tabs_lock;
use crate::ui::*;
use slint::VecModel;
use std::rc::Rc;

#[derive(Clone)]
pub struct Tab {
    pub path: String,
}

pub fn get_readonly_tab(i: usize) -> Tab {
    let t = tabs_lock();
    t[i].clone()
}
pub fn get_current_tab_idx() -> usize {
    //TODO
    0
}
