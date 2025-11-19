use crate::ui::{self, *};
use slint::{Model, VecModel};
use std::{
    collections::HashMap,
    rc::Rc,
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
    sel_files.drain();
    ui::run_with_main_window(|mw| {
        let fm = mw.global::<FileManager>();
        fm.set_is_single_selected(false);
        let visual_selected = fm.get_visual_selected();
        for i in 0..visual_selected.row_count() {
            visual_selected.set_row_data(i, false);
        }
    });
}

///Adds a file and its index to the selection while keeping what was previously selected
pub fn add_to_selected(i: i32, file: FileItem) {
    ui::run_with_main_window(move |mw| {
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

pub fn shift_select(i: i32) {
    ui::run_with_main_window(move |mw| {
        let mut sel_files = selected_files_write();
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
    });
}

///Removes the file at this index from the selection
pub fn remove_from_selected(i: i32) {
    ui::run_with_main_window(move |mw| {
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
    mw.global::<FileManager>()
        .get_visual_selected()
        .set_row_data(i as usize, val);
}

///Initialises the selected visual array to a new one (based on the row count)
///Usually called when the current directory changes
///
///TODO: Necessary?
pub fn init_selected_visual(mw: &MainWindow, row_count: usize) {
    mw.global::<FileManager>()
        .set_visual_selected(Rc::new(VecModel::from(vec![false; row_count])).into());
}

///If all selected files are indeed files AND they share an extension, return that extension.
///This is mostly used to allow opening multiple files at the same time with the same program.
pub fn get_common_extension() -> Option<String> {
    let files = selected_files_read();
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
    if same_file_type && same_extension {
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
