use std::rc::Rc;

use crate::ui::*;
use slint::{SortModel, Weak};

enum SortBy {
    Name,
    Size,
    Date,
}

static mut CURRENT_SORT: SortBy = SortBy::Name;
static mut CURRENT_ASC: bool = true;

//These are mostly safe since we only ever touch them in the main thread
fn set_current_sort(new_sort: SortBy, asc: bool) {
    unsafe {
        CURRENT_SORT = new_sort;
        CURRENT_ASC = asc;
    }
}
pub fn call_current_sort(mw: Rc<Weak<MainWindow>>) {
    unsafe {
        match CURRENT_SORT {
            SortBy::Name => sort_by_name(mw, CURRENT_ASC, false),
            _ => (),
        }
    }
}

//save: whether we need to modify the statics or not
//It's a small optimization, we don't need do it if calling from call_current_sort
pub fn sort_by_name(mw: Rc<Weak<MainWindow>>, ascending: bool, save: bool) {
    if save {
        set_current_sort(SortBy::Name, ascending);
    }
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
