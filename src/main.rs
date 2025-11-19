use i_slint_backend_winit::{EventResult, WinitWindowAccessor};
use qdfm::callbacks::utils::format_size_detailed;
use qdfm::core::generate_files_for_path;
use qdfm::globals::config_lock;
use qdfm::ui::*;
use qdfm::utils::drag_and_drop::{dnd_move, dnd_press, dnd_release, move_file, xdnd_init};
use qdfm::utils::error_handling::log_error_str;
use qdfm::{callbacks::*, enclose};
use qdfm::{drives, ui};
use slint::VecModel;
use std::rc::Rc;
use std::sync::Once;
use winit::event::WindowEvent;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

//TODO: Leak some globals
fn main() {
    //Use winit
    let backend = i_slint_backend_winit::Backend::new().unwrap();
    slint::platform::set_platform(Box::new(backend)).unwrap();

    //MainWindow
    let w: MainWindow = MainWindow::new().unwrap();

    //PropertiesWindow
    let prop_win = PropertiesWindow::new().unwrap();
    let prop_weak = Rc::new(prop_win.as_weak());

    start_ui_listener(w.as_weak());

    //TODO: Review what can be done in threads

    let weak = Rc::new(w.as_weak());
    //Initialization sequence
    //TODO: Optimize this
    {
        let mut conf = config_lock();
        let drives = drives::get_drives();
        w.global::<SidebarItems>().set_drive_list(drives.into());
        w.global::<FileManager>().set_files(
            Rc::new(VecModel::from(generate_files_for_path(
                conf.get::<String>("default_path").unwrap().as_str(),
            )))
            .into(),
        );
        w.global::<Theme>()
            .invoke_setup(conf.get::<String>("theme").unwrap().into(), 3840, 2160); //Change these
        w.global::<ColumnHeadersAdapter>()
            .set_headers(Rc::new(conf.get_headers()).into());

        //Temp, default breadcrumbs path
        /*w.global::<TabsAdapter>().set_breadcrumbs(
            [TabItem {
                internal_path: "/".into(),
                text: "".into(),
                selected: false,
                text_length: 1,
            }]
            .into(),
        );*/

        ui::send_message(UIMessage::SetCurrentTabFile(
            TabItem {
                internal_path: "/".into(),
                text: "".into(),
                selected: false,
                text_length: 1,
            },
            false,
        ));

        //Default sort
        //TODO use config
        qdfm::sort::sort_by_name(&w, true, true);

        conf.init_mappings();
    }

    // Listen to window events
    {
        let weak = weak.clone();
        static XDND_INIT: Once = Once::new();
        w.window()
            .on_winit_window_event(move |_, we: &WindowEvent| -> EventResult {
                match we {
                    WindowEvent::DroppedFile(buf) => {
                        let win = weak.unwrap();
                        let current_path = win
                            .global::<TabsAdapter>()
                            .invoke_get_current_tab()
                            .internal_path;
                        if let Some(buf_str) = buf.to_str() {
                            move_file(weak.clone(), buf_str, &current_path);
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        XDND_INIT.call_once(|| {
                            let win = weak.unwrap();
                            let window_id = win
                                .window()
                                .with_winit_window(|w| {
                                    if let Ok(handle) = w.window_handle() {
                                        match handle.as_raw() {
                                            RawWindowHandle::Xcb(h) => {
                                                return h.window.get();
                                            }
                                            RawWindowHandle::Xlib(h) => {
                                                return h.window as u32;
                                            }
                                            RawWindowHandle::Wayland(_) => {
                                                /*TODO*/
                                                return 0;
                                            }
                                            _ => (),
                                        }
                                    } else {
                                        log_error_str(
                                            "Could not get the window handle. Things may not work.",
                                        );
                                    }
                                    0
                                })
                                .expect("Could not find winit window??");
                            xdnd_init(window_id);
                        });
                    }
                    _ => {}
                }
                EventResult::Propagate
            });
    }

    //Callbacks
    {
        let sidebaritems = w.global::<SidebarItems>();
        sidebaritems.on_drive_clicked(|i| sidebar::sidebar_item_clicked(i));
        sidebaritems.on_left_arrow_clicked(|| sidebar::left_arrow_clicked());
        sidebaritems.on_right_arrow_clicked(|| sidebar::right_arrow_clicked());
        w.global::<FileManager>()
            .on_fileitem_doubleclicked(|file, i| filemanager::fileitem_doubleclicked(file, i));
        let tabs_adapter = w.global::<TabsAdapter>();
        tabs_adapter.on_breadcrumb_clicked(|i| tabs::breadcrumb_clicked(i));
        tabs_adapter.on_breadcrumb_accepted(|s| tabs::breadcrumb_accepted(s));
        w.global::<ColumnHeadersAdapter>()
            .on_header_clicked(|header| headers::on_header_click(header));
        w.global::<ColumnHeadersAdapter>()
            .on_adjust_size(|header, offset, original| {
                headers::on_header_resize(header, offset, original)
            });

        let file_manager = w.global::<FileManager>();
        file_manager.on_format_size(move |i| filemanager::format_size(i));
        file_manager.on_pressed(|| dnd_press());
        file_manager.on_released(|| dnd_release());
        file_manager.on_moved(|x, y| dnd_move(x, y));
        file_manager.on_format_date(move |i| filemanager::format_date(i));
        file_manager.on_add_to_selected(|i, f| filemanager::selection::add_to_selected(i, f));
        file_manager.on_is_index_selected(|i| filemanager::selection::is_index_selected(i));
        file_manager.on_remove_from_selected(|i| filemanager::selection::remove_from_selected(i));
        file_manager.on_reset_selected(|| filemanager::selection::clear_selection());
        file_manager
            .on_set_single_selected(|i, f| filemanager::selection::set_single_selected(i, f));
        file_manager.on_shift_select(|i| filemanager::selection::shift_select(i));
        file_manager.on_is_nothing_selected(move || filemanager::selection::is_nothing_selected());
        file_manager.on_clear_selection(|| filemanager::selection::clear_selection());
        prop_win
            .global::<PropertiesAdapter>()
            .on_format_size_detailed(move |i| format_size_detailed(i));
        prop_win
            .global::<FileManager>()
            .on_format_date(move |i| filemanager::format_date(i));
        prop_win
            .global::<PropertiesAdapter>()
            .on_ok(enclose! { (prop_weak) move || properties::ok(prop_weak.clone())});
        prop_win
            .global::<PropertiesAdapter>()
            .on_cancel(enclose! { (prop_weak) move || properties::cancel(prop_weak.clone())});
        prop_win
            .global::<PropertiesAdapter>()
            .on_recalculate_bitmask(
                enclose! { (prop_weak) move || properties::recalculate_bitmask(prop_weak.clone())},
            );
        let ctx_adp = w.global::<ContextAdapter>();
        ctx_adp.on_show_context_menu(|x, y| context_menu::show_context_menu(x, y));
        ctx_adp.on_menuitem_click(move |callback_item| {
            context_menu::menuitem_click(callback_item, prop_win.as_weak())
        });
        ctx_adp.on_menuitem_hover(|callback_item| context_menu::menuitem_hover(callback_item));
    }
    w.run().unwrap();
}
