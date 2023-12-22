use crate::core;
use crate::globals::config_lock;
use crate::ui::*;
use crate::utils::doubleclicks::check_for_dclick;
use once_cell::sync::OnceCell;
use slint::VecModel;
use slint::Weak;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use super::tabs::get_breadcrumbs_for;

//For now, there is no double click handler, CHANGE TO DOUBLE CLICK WHEN/IF IT'S IMPLEMENTED
pub fn fileitem_clicked(item: FileItem, mw: Rc<Weak<MainWindow>>) {
    if check_for_dclick() {
        fileitem_doubleclicked(item, mw);
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

pub fn set_current_tab_file(item: TabItem, mw: Rc<Weak<MainWindow>>, remember: bool) {
    let files = core::generate_files_for_path(item.internal_path.as_str());
    mw.upgrade_in_event_loop(move |w| {
        let tabs = w.global::<TabsAdapter>();
        if remember {
            add_to_history(tabs.invoke_get_current_tab());
        }
        tabs.set_breadcrumbs(Rc::new(VecModel::from(get_breadcrumbs_for(&item))).into());
        tabs.invoke_set_current_tab(item);
        w.global::<FileManager>()
            .set_files(Rc::new(VecModel::from(files)).into());
    })
    .unwrap();
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
