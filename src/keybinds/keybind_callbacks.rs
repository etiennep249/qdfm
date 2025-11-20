use crate::utils::error_handling::log_error_str;

///Calls the callback for a given function/feature.
///This 'name' string will have come directly from the user config
///There are no checks to make sure it's valid until now, where it will log and error if that is
///the case. Similar to the get_key function, this is very effective as a jump table.
pub fn call_keybind_callback(name: &str) {
    match name {
        "select_all" => {
            println!("selecting all");
        }
        _ => {
            log_error_str(
                "Invalid function for keybind! You may want to verify that you typed it correctly.",
            );
        }
    }
}
