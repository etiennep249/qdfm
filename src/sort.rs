use std::rc::Rc;

use crate::ui::*;
use slint::{SortModel, Weak};

pub fn sort_by_name(mw: Rc<Weak<MainWindow>>, ascending: bool) {
    let mw_upgraded = mw.unwrap();
    let fm = mw_upgraded.global::<FileManager>();
    if ascending {
        fm.set_files(
            Rc::new(SortModel::new(fm.get_files(), |lhs, rhs| {
                lhs.file_name
                    .to_lowercase()
                    .cmp(&rhs.file_name.to_lowercase())
            }))
            .into(),
        );
    } else {
        fm.set_files(
            Rc::new(SortModel::new(fm.get_files(), |lhs, rhs| {
                rhs.file_name
                    .to_lowercase()
                    .cmp(&lhs.file_name.to_lowercase())
            }))
            .into(),
        );
    }
}
