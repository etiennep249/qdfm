use crate::core;
use crate::globals::config_lock;
use crate::sort::call_current_sort;
use crate::ui::*;
use crate::utils::types;
use crate::utils::types::i32_to_i64;
use slint::Image;
use slint::SharedPixelBuffer;
use slint::SharedString;
use slint::VecModel;
use slint::Weak;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::OnceLock;

use super::context_menu::ContextCallback;
use super::tabs::get_breadcrumbs_for;

pub fn fileitem_doubleclicked(item: FileItem, _i: i32, mw: Rc<Weak<MainWindow>>) {
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
    let filemanager = w.global::<FileManager>();

    filemanager.set_files(Rc::new(VecModel::from(files)).into());
    call_current_sort(mw);
    filemanager.set_selected_index(-1);
}

pub fn format_size(i: _i64) -> SharedString {
    types::format_size(i32_to_i64((i.a, i.b)), false)
}
pub fn format_date(i: _i64) -> SharedString {
    types::format_date(i32_to_i64((i.a, i.b)))
}
pub fn show_context_menu(x: f32, y: f32, file: FileItem, mw: Rc<Weak<MainWindow>>) {
    let w = mw.unwrap();
    let ctx_adapter = w.global::<ContextAdapter>();

    //TODO: have all of these items stored somewhere so we dont genereate everytime

    let mut menu: Vec<ContextItem> = Vec::new();

    //TODO: Get shortcuts from config file
    let conf = config_lock();

    let default_mapping = conf.get_mapping_default(&file.extension);
    let quick_mapping = conf.get_mappings_quick(&file.extension);

    if default_mapping.is_some() {
        menu.push(ContextItem {
            display: ("Open With ".to_owned() + &default_mapping.unwrap()).into(),
            callback_id: ContextCallback::OpenWithDefault as i32,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: true,
            click_on_hover: false,
            internal_id: 0,
        });
    }
    if !quick_mapping.is_empty() {
        menu.push(ContextItem {
            display: ("Open With").into(),
            callback_id: ContextCallback::OpenWith as i32,
            shortcut: "▶".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: true,
            click_on_hover: true,
            internal_id: 0,
        });
    }
    menu.push(ContextItem {
        display: "Cut".into(),
        callback_id: ContextCallback::Cut as i32,
        shortcut: "".into(),
        icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
        has_separator: false,
        click_on_hover: false,
        internal_id: 0,
    });
    menu.push(ContextItem {
        display: "Copy".into(),
        callback_id: ContextCallback::Copy as i32,
        shortcut: "".into(),
        icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
        has_separator: false,
        click_on_hover: false,
        internal_id: 0,
    });
    if file.is_dir {
        menu.push(ContextItem {
            display: "Paste Into".into(),
            callback_id: ContextCallback::PasteIntoSelected as i32,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: true,
            click_on_hover: false,
            internal_id: 0,
        });
    } else {
        menu.push(ContextItem {
            display: "Paste Here".into(),
            callback_id: ContextCallback::PasteHere as i32,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: true,
            click_on_hover: false,
            internal_id: 0,
        });
    }
    menu.push(ContextItem {
        display: "Delete".into(),
        callback_id: ContextCallback::Delete as i32,
        shortcut: "".into(),
        icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
        has_separator: true,
        click_on_hover: false,
        internal_id: 0,
    });
    menu.push(ContextItem {
        display: "Properties".into(),
        callback_id: ContextCallback::ShowProperties as i32,
        shortcut: "".into(),
        icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
        has_separator: false,
        click_on_hover: false,
        internal_id: 0,
    });

    ctx_adapter.set_items(Rc::new(VecModel::from(menu)).into());
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
static NAV_HISTORY: OnceLock<Mutex<(VecDeque<TabItem>, VecDeque<TabItem>)>> = OnceLock::new();

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
