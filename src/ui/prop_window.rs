use slint::invoke_from_event_loop;

use crate::{
    callbacks::{filemanager, properties, utils::format_size_detailed},
    ui::{FileManager, PropertiesAdapter, PropertiesWindow},
};
use slint::ComponentHandle;
use std::sync::OnceLock;

///Properties Window
static PROPWINDOW: OnceLock<PropertiesWindow> = OnceLock::new();

unsafe impl Send for PropertiesWindow {}
unsafe impl Sync for PropertiesWindow {}

fn get_or_init_prop_window() -> &'static PropertiesWindow {
    PROPWINDOW.get_or_init(|| {
        let prop_win = PropertiesWindow::new().unwrap();
        prop_win
            .global::<PropertiesAdapter>()
            .on_format_size_detailed(move |i| format_size_detailed(i));
        prop_win
            .global::<FileManager>()
            .on_format_date(move |i| filemanager::format_date(i));
        prop_win
            .global::<PropertiesAdapter>()
            .on_ok(|| properties::ok());
        prop_win
            .global::<PropertiesAdapter>()
            .on_cancel(|| properties::cancel());
        prop_win
            .global::<PropertiesAdapter>()
            .on_recalculate_bitmask(|| properties::recalculate_bitmask());
        prop_win
    })
}

///Runs the given closure in the event loop with the PropertiesWindow instance.
pub fn run_with_prop_window(func: impl FnOnce(&PropertiesWindow) + Send + 'static) {
    invoke_from_event_loop(|| {
        func(get_or_init_prop_window());
    })
    .ok();
}
///Gets the a reference to the properties window.
///MUST BE CALLED FROM THE MAIN THREAD. Ideally used in combination with a run_with_x_window()
//TODO: panic if not in main thread
pub fn unwrap_prop_window() -> &'static PropertiesWindow {
    get_or_init_prop_window()
}
