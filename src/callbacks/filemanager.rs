use crate::core;
use crate::globals::config_lock;
use crate::sort::call_current_sort;
use crate::ui::*;
use crate::utils::doubleclicks::check_for_dclick;
use crate::utils::types;
use crate::utils::types::i32_to_i64;
use once_cell::sync::OnceCell;
use slint::Image;
use slint::SharedPixelBuffer;
use slint::SharedString;
use slint::VecModel;
use slint::Weak;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use super::tabs::get_breadcrumbs_for;

//For now, there is no double click handler, CHANGE TO DOUBLE CLICK WHEN/IF IT'S IMPLEMENTED
pub fn fileitem_clicked(item: FileItem, index: i32, mw: Rc<Weak<MainWindow>>) -> bool {
    if check_for_dclick(index) {
        fileitem_doubleclicked(item, mw);
        return true;
    } else {
        return false;
    }
}

pub fn fileitem_doubleclicked(item: FileItem, mw: Rc<Weak<MainWindow>>) {
    set_current_tab_file(
        TabItem {
            internal_path: item.path,
            text: item.file_name.clone(),
            text_length: item.file_name.len() as i32,
            selected: true,
        },
        mw,
        true,
    );
}

pub fn set_current_tab_file(mut item: TabItem, mw: Rc<Weak<MainWindow>>, remember: bool) {
    let files = core::generate_files_for_path(item.internal_path.as_str());
    if item.internal_path == "/" {
        item.text = "/".into();
    }

    let w = mw.unwrap();
    let tabs = w.global::<TabsAdapter>();
    tabs.set_path_shown(false);
    if remember {
        add_to_history(tabs.invoke_get_current_tab());
    }
    tabs.set_breadcrumbs(Rc::new(VecModel::from(get_breadcrumbs_for(&item))).into());
    tabs.invoke_set_current_tab(item);
    w.global::<FileManager>()
        .set_files(Rc::new(VecModel::from(files)).into());
    call_current_sort(mw);
}

pub fn format_size(i: _i64) -> SharedString {
    types::format_size(i32_to_i64((i.a, i.b)))
}
pub fn format_date(i: _i64) -> SharedString {
    types::format_date(i32_to_i64((i.a, i.b)))
}
pub fn show_context_menu(x: f32, y: f32, file: FileItem, mw: Rc<Weak<MainWindow>>) {
    let w = mw.unwrap();
    let ctx_adapter = w.global::<ContextAdapter>();

    let menu = [
        ContextItem {
            display: "Open With <default>".into(),
            callback_id: 0,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: true,
        },
        ContextItem {
            display: "Properties".into(),
            callback_id: 1,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: false,
        },
    ];

    //TODO: Have these somewhere so we don't have to generate it everytime
    ctx_adapter.set_items(menu.into());

    ctx_adapter.set_x_pos(x + 1f32);
    ctx_adapter.set_y_pos(y + 1f32);
}

/*
*   ====HISTORY===
*
*   Two VecDeque, one for LHISTORY (which we query when using the left arrow)
*   and one for RHISTORY (which we query when using the right arrow).
*
*   get_history() is a simple getter for the static mutex lock
*
*   add_to_history appends to LHISTORY and is called whenever we call set_current_tab_file(_,_,true)
*   It rotates LHISTORY to only keep max_nav_history items in the history.
*
*   get_prev_history and get_next_history are called when clicking the left/right arrows. They
*   return the TabItem we then navigate to. They also update the other VecDeque accordingly.
*
* */

/*(LHistory, RHistory)*/
static NAV_HISTORY: OnceCell<Mutex<(VecDeque<TabItem>, VecDeque<TabItem>)>> = OnceCell::new();

pub fn get_history() -> MutexGuard<'static, (VecDeque<TabItem>, VecDeque<TabItem>)> {
    NAV_HISTORY
        .get_or_init(|| {
            let conf = config_lock();
            Mutex::new((
                VecDeque::with_capacity(conf.get("max_nav_history").unwrap()),
                VecDeque::with_capacity(conf.get("max_nav_history").unwrap()),
            ))
        })
        .lock()
        .unwrap()
}
pub fn add_to_history(item: TabItem) {
    let mut hist = get_history();
    if hist.0.len() >= config_lock().get("max_nav_history").unwrap() {
        hist.0.pop_front();
    }
    hist.0.push_back(item);
}
pub fn get_prev_history(mw: Rc<Weak<MainWindow>>) -> Option<TabItem> {
    let mut hist = get_history();
    let item = hist.0.pop_back();
    match item {
        Some(e) => {
            /*Push the path we were on to RHISTORY*/
            mw.upgrade_in_event_loop(move |w| {
                get_history()
                    .1
                    .push_back(w.global::<TabsAdapter>().invoke_get_current_tab());
            })
            .unwrap();
            Some(e)
        }
        None => None,
    }
}
pub fn get_next_history(mw: Rc<Weak<MainWindow>>) -> Option<TabItem> {
    let mut hist = get_history();
    let item = hist.1.pop_back();
    match item {
        Some(e) => {
            /*Push the path we were on to LHISTORY*/
            mw.upgrade_in_event_loop(move |w| {
                get_history()
                    .0
                    .push_back(w.global::<TabsAdapter>().invoke_get_current_tab());
            })
            .unwrap();
            Some(e)
        }
        None => None,
    }
}
