use std::{cmp::Ordering, rc::Rc};

use crate::{ui::*, utils::types::i32_to_i64};
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
            SortBy::Date => sort_by_date(mw, CURRENT_ASC, false),
            SortBy::Size => sort_by_size(mw, CURRENT_ASC, false),
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
                if lhs.is_dir && !rhs.is_dir {
                    Ordering::Less
                } else if rhs.is_dir && !lhs.is_dir {
                    Ordering::Greater
                } else {
                    lhs.file_name
                        .to_lowercase()
                        .cmp(&rhs.file_name.to_lowercase())
                }
            }))
            .into(),
        );
    } else {
        fm.set_files(
            Rc::new(SortModel::new(fm.get_files(), |lhs, rhs| {
                if lhs.is_dir && !rhs.is_dir {
                    Ordering::Less
                } else if rhs.is_dir && !lhs.is_dir {
                    Ordering::Greater
                } else {
                    rhs.file_name
                        .to_lowercase()
                        .cmp(&lhs.file_name.to_lowercase())
                }
            }))
            .into(),
        );
    }
}
pub fn sort_by_date(mw: Rc<Weak<MainWindow>>, ascending: bool, save: bool) {
    if save {
        set_current_sort(SortBy::Date, ascending);
    }
    let mw_upgraded = mw.unwrap();
    let fm = mw_upgraded.global::<FileManager>();
    if ascending {
        fm.set_files(
            Rc::new(SortModel::new(fm.get_files(), |lhs, rhs| {
                if lhs.is_dir && !rhs.is_dir {
                    Ordering::Less
                } else if rhs.is_dir && !lhs.is_dir {
                    Ordering::Greater
                } else {
                    if lhs.date.a == 0 && rhs.date.a == 0 {
                        // No need to convert to 64 bits if date is stored entirely in 32 bits.
                        // This should always be true until 2038
                        lhs.date.b.cmp(&rhs.date.b)
                    } else {
                        i32_to_i64((lhs.date.a, lhs.date.b))
                            .cmp(&i32_to_i64((rhs.date.a, rhs.date.b)))
                    }
                }
            }))
            .into(),
        );
    } else {
        fm.set_files(
            Rc::new(SortModel::new(fm.get_files(), |lhs, rhs| {
                if lhs.is_dir && !rhs.is_dir {
                    Ordering::Less
                } else if rhs.is_dir && !lhs.is_dir {
                    Ordering::Greater
                } else {
                    if lhs.date.a == 0 && rhs.date.a == 0 {
                        rhs.date.b.cmp(&lhs.date.b)
                    } else {
                        i32_to_i64((rhs.date.a, rhs.date.b))
                            .cmp(&i32_to_i64((lhs.date.a, lhs.date.b)))
                    }
                }
            }))
            .into(),
        );
    }
}
pub fn sort_by_size(mw: Rc<Weak<MainWindow>>, ascending: bool, save: bool) {
    if save {
        set_current_sort(SortBy::Size, ascending);
    }
    let mw_upgraded = mw.unwrap();
    let fm = mw_upgraded.global::<FileManager>();
    if ascending {
        fm.set_files(
            Rc::new(SortModel::new(fm.get_files(), |lhs, rhs| {
                if lhs.is_dir && !rhs.is_dir {
                    Ordering::Less
                } else if rhs.is_dir && !lhs.is_dir {
                    Ordering::Greater
                } else {
                    if lhs.size.a == 0 && rhs.size.a == 0 {
                        lhs.size.b.cmp(&rhs.size.b)
                    } else {
                        i32_to_i64((lhs.size.a, lhs.size.b))
                            .cmp(&i32_to_i64((rhs.size.a, rhs.size.b)))
                    }
                }
            }))
            .into(),
        );
    } else {
        fm.set_files(
            Rc::new(SortModel::new(fm.get_files(), |lhs, rhs| {
                if lhs.is_dir && !rhs.is_dir {
                    Ordering::Less
                } else if rhs.is_dir && !lhs.is_dir {
                    Ordering::Greater
                } else {
                    if lhs.size.a == 0 && rhs.size.a == 0 {
                        rhs.size.b.cmp(&lhs.size.b)
                    } else {
                        i32_to_i64((rhs.size.a, rhs.size.b))
                            .cmp(&i32_to_i64((lhs.size.a, lhs.size.b)))
                    }
                }
            }))
            .into(),
        );
    }
}
