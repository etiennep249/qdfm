use crate::context_menus::files::open_with_default;
use crate::globals::config_read;
use crate::ui;
use crate::ui::*;
use crate::utils::types;
use crate::utils::types::i32_to_i64;
use main_window::run_with_main_window;
use slint::ComponentHandle;
use slint::SharedString;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::OnceLock;

pub mod selection;

///When a file is double clicked, it is opened with the default mapping.
///When a directory is double clicked, it is "moved into" or set as the new current directory.
pub fn fileitem_doubleclicked(item: FileItem) {
    if item.is_dir {
        ui::send_message(UIMessage::SetCurrentTabFile(
            TabItem {
                internal_path: item.path,
                text: item.file_name.clone(),
                text_length: item.file_name.len() as i32,
                selected: true,
            },
            true,
        ));
    } else {
        let is_some = {
            let conf = config_read();
            let default_mapping = conf.get_mapping_default(&item.extension);
            default_mapping.is_some()
        };
        if is_some {
            open_with_default(vec![item]);
        }
    }
}

pub fn format_size(i: _i64) -> SharedString {
    types::format_size(i32_to_i64((i.a, i.b)) as u64, false)
}
pub fn format_date(i: _i64) -> SharedString {
    types::format_date(i32_to_i64((i.a, i.b)))
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
static NAV_HISTORY: OnceLock<Mutex<(VecDeque<TabItem>, VecDeque<TabItem>)>> = OnceLock::new();

pub fn get_history() -> MutexGuard<'static, (VecDeque<TabItem>, VecDeque<TabItem>)> {
    NAV_HISTORY
        .get_or_init(|| {
            let conf = config_read();
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
    if hist.0.len() >= config_read().get("max_nav_history").unwrap() {
        hist.0.pop_front();
    }
    hist.0.push_back(item);
}
pub fn get_prev_history() -> Option<TabItem> {
    let mut hist = get_history();
    let item = hist.0.pop_back();
    match item {
        Some(e) => {
            drop(hist);
            /*Push the path we were on to RHISTORY*/
            run_with_main_window(|mw| {
                get_history()
                    .1
                    .push_back(mw.global::<TabsAdapter>().invoke_get_current_tab());
            });
            Some(e)
        }
        None => None,
    }
}
pub fn get_next_history() -> Option<TabItem> {
    let mut hist = get_history();
    let item = hist.1.pop_back();
    match item {
        Some(e) => {
            drop(hist);
            /*Push the path we were on to LHISTORY*/
            run_with_main_window(|mw| {
                get_history()
                    .1
                    .push_back(mw.global::<TabsAdapter>().invoke_get_current_tab());
            });
            Some(e)
        }
        None => None,
    }
}
