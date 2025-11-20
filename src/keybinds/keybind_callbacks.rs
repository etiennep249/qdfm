use crate::{
    callbacks::filemanager::selection, context_menus::files::open_with_default, ui,
    utils::error_handling::log_error_str,
};

///Calls the callback for a given function/feature.
///This 'name' string will have come directly from the user config
///There are no checks to make sure it's valid until now, where it will log and error if that is
///the case. Similar to the get_key function, this is very effective as a jump table.
pub fn call_keybind_callback(name: &str) {
    match name {
        "select_all" => selection::select_all(),
        "select_down" => selection::select_down(true),
        "select_up" => selection::select_up(true),
        "shift_select_down" => selection::select_down(false),
        "shift_select_up" => selection::select_up(false),
        "enter" => {
            //If multiple files are selected, make sure they are all files, then run them
            if selection::get_common_extension().is_some() {
                let files = selection::selected_files_read().values().cloned().collect();
                open_with_default(files);
            } else if selection::is_single_selected_directory() {
                let file = selection::get_selected_file().unwrap();
                ui::send_message(ui::UIMessage::SetCurrentTabFile(
                    ui::TabItem {
                        internal_path: file.path,
                        selected: true,
                        text: file.file_name.clone(),
                        text_length: file.file_name.len() as i32,
                    },
                    true,
                ));
            }
        }
        _ => {
            log_error_str(&format!(
                "Invalid function for keybind! You may want to verify that you typed it correctly. '{}'",name
            ));
        }
    }
}
