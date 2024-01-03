use std::rc::Rc;

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
//TODO: Can optimize by not converting to i64 if it's already smaller FOR THIS AND ESP SIZE
//Slight perf improvement, but given how many files there are to sort, might be worth it
pub fn sort_by_date(mw: Rc<Weak<MainWindow>>, ascending: bool, save: bool) {
    if save {
        set_current_sort(SortBy::Date, ascending);
    }
    let mw_upgraded = mw.unwrap();
    let fm = mw_upgraded.global::<FileManager>();
    if ascending {
        fm.set_files(
            Rc::new(SortModel::new(fm.get_files(), |lhs, rhs| {
                i32_to_i64((lhs.date.a, lhs.date.b)).cmp(&i32_to_i64((rhs.date.a, rhs.date.b)))
            }))
            .into(),
        );
    } else {
        fm.set_files(
            Rc::new(SortModel::new(fm.get_files(), |lhs, rhs| {
                i32_to_i64((rhs.date.a, rhs.date.b)).cmp(&i32_to_i64((lhs.date.a, lhs.date.b)))
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
                i32_to_i64((lhs.size.a, lhs.size.b)).cmp(&i32_to_i64((rhs.size.a, rhs.size.b)))
            }))
            .into(),
        );
    } else {
        fm.set_files(
            Rc::new(SortModel::new(fm.get_files(), |lhs, rhs| {
                i32_to_i64((rhs.size.a, rhs.size.b)).cmp(&i32_to_i64((lhs.size.a, lhs.size.b)))
            }))
            .into(),
        );
    }
}
