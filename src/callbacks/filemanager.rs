use crate::context_menus::cm_file::open_with_default;
use crate::core;
use crate::globals::config_lock;
use crate::globals::selected_files_lock;
use crate::globals::selected_files_try_lock;
use crate::sort::call_current_sort;
use crate::ui::*;
use crate::utils::types;
use crate::utils::types::i32_to_i64;
use slint::ComponentHandle;
use slint::Image;
use slint::Model;
use slint::SharedPixelBuffer;
use slint::SharedString;
use slint::VecModel;
use slint::Weak;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::OnceLock;
use std::thread::sleep;
use std::time::Duration;

use super::context_menu::ContextCallback;
use super::tabs::get_breadcrumbs_for;

pub fn fileitem_doubleclicked(item: FileItem, _i: i32, mw: Rc<Weak<MainWindow>>) {
    if item.is_dir {
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
    } else {
        //Care with config lock, needs to be dropped before call to open_with_default
        let is_some = {
            let conf = config_lock();
            let default_mapping = conf.get_mapping_default(&item.extension);
            default_mapping.is_some()
        };
        if is_some {
            open_with_default(vec![item]);
        }
    }
}
// Selected files functions
pub fn is_index_selected(i: i32) -> bool {
    selected_files_lock().contains_key(&i)
}

pub fn clear_selection(mw: Rc<Weak<MainWindow>>) {
    reset_selected(mw);
}

pub fn add_to_selected(mw: Rc<Weak<MainWindow>>, i: i32, file: FileItem) {
    let mut sel_files = selected_files_lock();
    sel_files.insert(i, file);
    set_selected_visual(mw.clone(), i, true);
    let mw = mw.unwrap();
    let fm = mw.global::<FileManager>();
    if sel_files.len() == 2 {
        fm.set_is_single_selected(false);
    } else if sel_files.len() == 1 {
        fm.set_is_single_selected(true);
    }
    fm.set_single_selected_index(i);
}

pub fn shift_select(mww: Rc<Weak<MainWindow>>, i: i32) {
    let mut sel_files = selected_files_lock();
    let mw = mww.unwrap();
    let fm = mw.global::<FileManager>();
    let last_selected_index = fm.get_single_selected_index();
    let visual_selected = fm.get_visual_selected();

    let was_clicked_selected = sel_files.contains_key(&i);

    //To decide whether we go reverse or not
    let range = if i < last_selected_index && !was_clicked_selected {
        i..=last_selected_index
    } else if i > last_selected_index && !was_clicked_selected {
        last_selected_index..=i
    } else if i < last_selected_index && was_clicked_selected {
        (i + 1)..=last_selected_index
    } else {
        last_selected_index..=(i + 1)
    };
    for i in range {
        if !was_clicked_selected {
            sel_files.insert(i, fm.get_files().row_data(i as usize).unwrap());
            visual_selected.set_row_data(i as usize, true);
        } else {
            sel_files.remove(&i);
            visual_selected.set_row_data(i as usize, false);
        }
    }
    fm.set_is_single_selected(false);
    fm.set_single_selected_index(i);
}

pub fn remove_from_selected(mw: Rc<Weak<MainWindow>>, i: i32) {
    let mut sel_files = selected_files_lock();
    if sel_files.len() > 1 {
        mw.unwrap()
            .global::<FileManager>()
            .set_is_single_selected(false);
    }
    sel_files.remove(&i);
    set_selected_visual(mw, i, false);
}

//Will give it a couple tries, but not guaranteed to work.
//CANNOT be blocking with a regular lock here since this is likely called
//from a scope that already has the lock.
pub fn reset_selected(mw: Rc<Weak<MainWindow>>) {
    let mut attempts = 0;
    while attempts < 15 {
        if let Ok(mut sel_files) = selected_files_try_lock() {
            sel_files.drain();
            let mw = mw.unwrap();
            let fm = mw.global::<FileManager>();
            fm.set_is_single_selected(false);
            let visual_selected = fm.get_visual_selected();
            for i in 0..visual_selected.row_count() {
                visual_selected.set_row_data(i, false);
            }
        }
        attempts += 1;
        sleep(Duration::from_millis(5));
    }
}

pub fn set_single_selected(mw: Rc<Weak<MainWindow>>, i: i32, file: FileItem) {
    let mut sel_files = selected_files_lock();
    sel_files.drain();
    sel_files.insert(i, file);

    reset_selected_visual_list(mw.clone());
    set_selected_visual(mw.clone(), i, true);
    let mw = mw.unwrap();
    let fm = mw.global::<FileManager>();

    fm.set_is_single_selected(true);
    fm.set_single_selected_index(i);
}

pub fn is_nothing_selected() -> bool {
    selected_files_lock().is_empty()
}

pub fn set_selected_visual(mw: Rc<Weak<MainWindow>>, i: i32, val: bool) {
    mw.unwrap()
        .global::<FileManager>()
        .get_visual_selected()
        .set_row_data(i as usize, val);
}
pub fn reset_selected_visual_list(mw: Rc<Weak<MainWindow>>) {
    let mw = mw.unwrap();
    let fm = mw.global::<FileManager>();

    fm.set_visual_selected(
        Rc::new(VecModel::from(vec![
            false;
            fm.get_visual_selected().row_count()
        ]))
        .into(),
    );
}

pub fn init_selected_visual(mw: Rc<Weak<MainWindow>>, row_count: usize) {
    mw.unwrap()
        .global::<FileManager>()
        .set_visual_selected(Rc::new(VecModel::from(vec![false; row_count])).into());
}

//TODO: Better refresh. Perhaps a queue? Don't want UI to abruptly refresh when background
//operations finish. That or make this function non-distruptive, maintain selected files.
pub fn set_current_tab_file(mut item: TabItem, mw: Rc<Weak<MainWindow>>, remember: bool) {
    let files = core::generate_files_for_path(item.internal_path.as_str());
    if item.internal_path == "/" {
        item.text = "/".into();
    }
    let files_len = files.len();

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
    call_current_sort(mw.clone());
    reset_selected(mw.clone());
    init_selected_visual(mw, files_len);
}

pub fn format_size(i: _i64) -> SharedString {
    types::format_size(i32_to_i64((i.a, i.b)) as u64, false)
}
pub fn format_date(i: _i64) -> SharedString {
    types::format_date(i32_to_i64((i.a, i.b)))
}
pub fn show_context_menu(x: f32, y: f32, mw: Rc<Weak<MainWindow>>) {
    //To verify if everything is a file and their extension
    //If they are, we can offer to open, and with the extension's mappings
    let (file, file_count, all_files, same_mapping) = {
        let files = selected_files_lock();
        let mut iter = files.iter();
        let first = iter.next().unwrap().1;
        let mut same_file_type = true;
        let mut same_extension = true;
        for f in iter {
            if f.1.extension != first.extension {
                same_extension = false;
            }
            if f.1.is_dir != first.is_dir {
                same_file_type = false;
            }
        }

        (
            first.clone(),
            files.len(),
            same_file_type && !first.is_dir,
            same_extension,
        )
    };

    let w = mw.unwrap();
    let ctx_adapter = w.global::<ContextAdapter>();

    //TODO: have all of these items stored somewhere so we dont genereate everytime

    let mut menu: Vec<ContextItem> = Vec::new();

    let conf = config_lock();

    let default_mapping = if file_count > 1 && !same_mapping {
        None
    } else {
        conf.get_mapping_default(&file.extension)
    };

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
    if all_files {
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
    if file.is_dir && file_count == 1 {
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
