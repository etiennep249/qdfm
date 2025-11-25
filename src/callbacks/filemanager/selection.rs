use crate::ui::*;
use main_window::{get_selected_tab_file, run_with_main_window};
use slint::Model;
use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

///Selected files are kept under the global SELECTED_FILES, hidden behind a RwLock.
///
///(index, file)
static SELECTED_FILES: OnceLock<RwLock<HashMap<i32, FileItem>>> = OnceLock::new();
pub fn selected_files_read() -> RwLockReadGuard<'static, HashMap<i32, FileItem>> {
    match SELECTED_FILES
        .get_or_init(|| RwLock::new(HashMap::new()))
        .read()
    {
        Ok(e) => e,
        Err(_) => {
            panic!("Could not get SELECTED_FILES lock.");
        }
    }
}
pub fn selected_files_clone() -> Vec<FileItem> {
    selected_files_read().values().cloned().collect()
}

///Do not make this public
fn selected_files_write() -> RwLockWriteGuard<'static, HashMap<i32, FileItem>> {
    match SELECTED_FILES
        .get_or_init(|| RwLock::new(HashMap::new()))
        .write()
    {
        Ok(e) => e,
        Err(_) => {
            panic!("Could not get SELECTED_FILES lock.");
        }
    }
}
#[cfg(test)]
///Public version for testing purposes
pub fn selected_files_write_tests() -> RwLockWriteGuard<'static, HashMap<i32, FileItem>> {
    match SELECTED_FILES
        .get_or_init(|| RwLock::new(HashMap::new()))
        .write()
    {
        Ok(e) => e,
        Err(_) => {
            panic!("Could not get SELECTED_FILES lock.");
        }
    }
}
///Returns the selected file if there is exactly one selected. Otherwise returns None
pub fn get_selected_file() -> Option<FileItem> {
    let selected_files = selected_files_read();
    if selected_files.len() != 1 {
        None
    } else {
        Some(selected_files.iter().next().unwrap().1.clone())
    }
}

///Returns whether or not that particular index is selected
pub fn is_index_selected(i: i32) -> bool {
    selected_files_read().contains_key(&i)
}

///Clears selection
///Requires a handle to the MainWindow to update the UI accordingly
pub fn clear_selection() {
    let mut sel_files = selected_files_write();
    sel_files.clear();
    run_with_main_window(|mw| {
        let fm = mw.global::<FileManager>();
        fm.set_is_single_selected(false);
        clear_selection_visual(&mw);
    });
}

///This function resets the visual selection state
fn clear_selection_visual(mw: &MainWindow) {
    let fm = mw.global::<FileManager>();
    let files = fm.get_files();
    for i in 0..files.row_count() {
        fm.invoke_set_selected(i as i32, false);
    }
}

///Adds a file and its index to the selection while keeping what was previously selected
pub fn add_to_selected(i: i32, file: FileItem) {
    run_with_main_window(move |mw| {
        let mut sel_files = selected_files_write();
        sel_files.insert(i, file.clone());
        set_selected_visual(&mw, i, true);
        let fm = mw.global::<FileManager>();
        if sel_files.len() == 2 {
            fm.set_is_single_selected(false);
        } else if sel_files.len() == 1 {
            fm.set_is_single_selected(true);
        }
        fm.set_single_selected_index(i);
    });
}

///Selects all files in the current directory.
pub fn select_all() {
    run_with_main_window(move |mw| {
        let mut sel_files = selected_files_write();
        let fm = mw.global::<FileManager>();
        for (i, file) in fm.get_files().iter().enumerate() {
            sel_files.insert(i as i32, file.clone());
            set_selected_visual(&mw, i as i32, true);
        }
        if sel_files.len() > 1 {
            fm.set_is_single_selected(false);
            fm.set_single_selected_index(sel_files.len() as i32 - 1);
        } else if sel_files.len() == 1 {
            fm.set_is_single_selected(true);
            fm.set_single_selected_index(0);
        }
    });
}

pub fn shift_select(i: i32) {
    run_with_main_window(move |mw| {
        let mut sel_files = selected_files_write();
        let fm = mw.global::<FileManager>();
        let last_selected_index = fm.get_single_selected_index();

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
                fm.invoke_set_selected(i, true);
            } else {
                sel_files.remove(&i);
                fm.invoke_set_selected(i, false);
            }
        }
        fm.set_is_single_selected(false);
        fm.set_single_selected_index(i);
    });
}

///Usually when pressing arrow down on the keyboard.
///Selection moves down by one from the last.
pub fn select_down(discard_previous: bool) {
    run_with_main_window(move |mw| {
        let mut sel_files = selected_files_write();
        let fm = mw.global::<FileManager>();

        let current_i = fm.get_single_selected_index();
        let i = if sel_files.len() == 0 {
            0
        } else {
            if current_i == fm.get_files_len() - 1 {
                //Last index, do nothing
                return;
            } else {
                current_i + 1
            }
        };

        if discard_previous {
            //Single select
            sel_files.clear();
            clear_selection_visual(&mw);
            fm.set_is_single_selected(true);
            sel_files.insert(i, fm.invoke_get_file(i));
            set_selected_visual(&mw, i, true);
            fm.set_single_selected_index(i);
        } else {
            //Logic for shift_select
            if sel_files.contains_key(&i) {
                sel_files.remove(&current_i);
                set_selected_visual(&mw, current_i, false);
            } else {
                sel_files.insert(i, fm.invoke_get_file(i));
                set_selected_visual(&mw, i, true);
            }
            fm.set_single_selected_index(i);
            if sel_files.len() == 1 {
                fm.set_is_single_selected(true);
            } else {
                fm.set_is_single_selected(false);
            }
        }
    });
}

///Usually when pressing arrow up on the keyboard.
///Selection moves up by one from the last.
pub fn select_up(discard_previous: bool) {
    run_with_main_window(move |mw| {
        let mut sel_files = selected_files_write();
        let fm = mw.global::<FileManager>();

        let current_i = fm.get_single_selected_index();
        let i = if sel_files.len() == 0 {
            fm.get_files_len() - 1
        } else {
            if current_i == 0 {
                //Last index, do nothing
                return;
            } else {
                current_i - 1
            }
        };

        if discard_previous {
            //Single select
            sel_files.clear();
            clear_selection_visual(&mw);
            fm.set_is_single_selected(true);
            sel_files.insert(i, fm.invoke_get_file(i));
            set_selected_visual(&mw, i, true);
            fm.set_single_selected_index(i);
        } else {
            //Logic for shift_select
            if sel_files.contains_key(&i) {
                sel_files.remove(&current_i);
                set_selected_visual(&mw, current_i, false);
            } else {
                sel_files.insert(i, fm.invoke_get_file(i));
                set_selected_visual(&mw, i, true);
            }
            fm.set_single_selected_index(i);
            if sel_files.len() == 1 {
                fm.set_is_single_selected(true);
            } else {
                fm.set_is_single_selected(false);
            }
        }
    });
}

///Removes the file at this index from the selection
pub fn remove_from_selected(i: i32) {
    run_with_main_window(move |mw| {
        let mut sel_files = selected_files_write();
        if sel_files.len() > 1 {
            mw.global::<FileManager>().set_is_single_selected(false);
        }
        sel_files.remove(&i);
        set_selected_visual(&mw, i, false);
    });
}

///Sets the file as the only selected file. Will clear the selection first.
pub fn set_single_selected(i: i32, file: FileItem) {
    clear_selection();
    add_to_selected(i, file);
}

///Returns true if nothing is selected
pub fn is_nothing_selected() -> bool {
    selected_files_read().is_empty()
}

///Returns true if only one file/folder is selected
pub fn only_one_selected() -> bool {
    selected_files_read().len() == 1
}

///Makes the file visually selected
pub fn set_selected_visual(mw: &MainWindow, i: i32, val: bool) {
    mw.global::<FileManager>().invoke_set_selected(i, val);
}

///If all selected files are indeed files AND they share an extension, return that extension.
///This is mostly used to allow opening multiple files at the same time with the same program.
pub fn get_common_extension() -> Option<String> {
    let files = selected_files_read();
    if files.is_empty() {
        return None;
    }
    let mut iter = files.iter();
    let first = iter.next().unwrap().1;
    if first.is_dir {
        return None;
    }
    let mut same_extension = true;
    for f in iter {
        if f.1.extension != first.extension {
            same_extension = false;
        }
        if f.1.is_dir {
            return None;
        }
    }
    if same_extension {
        return Some(first.extension.clone().into());
    } else {
        return None;
    }
}

///Returns true if only one file is selected and it's a directory.
pub fn is_single_selected_directory() -> bool {
    let files = selected_files_read();
    if files.len() == 1 && files.iter().next().unwrap().1.is_dir {
        return true;
    } else {
        return false;
    }
}

///TODO: Name might not be representative
///
///This function returns the base path of a selected item
///If nothing is selected, this corresponds to the current directory
///If a single directory is selected, this corresponds to the directory selected
///If more than one file/directory is selected, this returns None.
///
///This should be used for actions that can be done in a directory, eg creating a file
pub fn get_selected_path() -> Option<String> {
    let files = selected_files_read();
    let selected_tabitem = get_selected_tab_file();

    //Nothing selected -> Current dir
    if files.len() == 0 {
        return selected_tabitem.map(|tab| tab.internal_path.to_string());
    } else if files.len() == 1 {
        if let Some(selected_file) = get_selected_file() {
            //One dir selected -> that dir's path
            if selected_file.is_dir {
                return Some(selected_file.path.to_string());
            }
        }
    }
    None
}
