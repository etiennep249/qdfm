use crate::callbacks::{context_menu, filemanager, headers, sidebar, tabs};
use crate::clipboard::move_file;
use crate::keybinds::handle_key_press;
use crate::sort::sort_by_name;
use crate::utils::drag_and_drop::{dnd_move, dnd_press, dnd_release, xdnd_init};
use crate::utils::error_handling::log_error_str;
use crate::{core::generate_files_for_path, drives, globals::config_write, ui::*};
use i_slint_backend_winit::{EventResult, WinitWindowAccessor};
use slint::ComponentHandle;
use slint::{invoke_from_event_loop, VecModel};
use std::sync::Once;
use std::{rc::Rc, sync::OnceLock};
use winit::event::WindowEvent;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

static MAINWINDOW: OnceLock<MainWindow> = OnceLock::new();

///Runs the given closure with the main window.
pub fn run_with_main_window(func: impl FnOnce(&MainWindow) + Send + 'static) {
    invoke_from_event_loop(|| {
        func(get_or_init_main_window());
    })
    .ok();
}

static XDND_INIT: Once = Once::new();
fn get_or_init_main_window() -> &'static MainWindow {
    MAINWINDOW.get_or_init(|| {
        let w: MainWindow = MainWindow::new().unwrap();

        let conf = config_write();
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
        // Listen to window events

        w.window()
            .on_winit_window_event(move |_, we: &WindowEvent| -> EventResult {
                match we {
                    WindowEvent::DroppedFile(buf) => {
                        let buf = buf.clone();
                        run_with_main_window(move |win| {
                            let current_path = win
                                .global::<TabsAdapter>()
                                .invoke_get_current_tab()
                                .internal_path;
                            if let Some(buf_str) = buf.to_str() {
                                move_file(buf_str, &current_path);
                            }
                        });
                    }
                    WindowEvent::RedrawRequested => {
                        XDND_INIT.call_once(|| {
                            run_with_main_window(|win| {
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
                        });
                    }
                    _ => {}
                }
                EventResult::Propagate
            });

        //Callbacks
        {
            w.on_handle_key_press(|key| handle_key_press(key));
            let sidebaritems = w.global::<SidebarItems>();
            sidebaritems.on_drive_clicked(|i| sidebar::sidebar_item_clicked(i));
            sidebaritems.on_left_arrow_clicked(|| sidebar::left_arrow_clicked());
            sidebaritems.on_right_arrow_clicked(|| sidebar::right_arrow_clicked());
            w.global::<FileManager>()
                .on_fileitem_doubleclicked(|file, _| filemanager::fileitem_doubleclicked(file));
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
            file_manager
                .on_remove_from_selected(|i| filemanager::selection::remove_from_selected(i));
            file_manager.on_reset_selected(|| filemanager::selection::clear_selection());
            file_manager
                .on_set_single_selected(|i, f| filemanager::selection::set_single_selected(i, f));
            file_manager.on_shift_select(|i| filemanager::selection::shift_select(i));
            file_manager
                .on_is_nothing_selected(move || filemanager::selection::is_nothing_selected());
            file_manager.on_clear_selection(|| filemanager::selection::clear_selection());

            let ctx_adp = w.global::<ContextAdapter>();
            ctx_adp.on_show_context_menu(|x, y| context_menu::show_context_menu(x, y));
            ctx_adp.on_menuitem_click(move |callback_item| {
                context_menu::menuitem_click(callback_item)
            });
            ctx_adp.on_menuitem_hover(|callback_item| context_menu::menuitem_hover(callback_item));
        }

        //Default sort
        //TODO use config
        sort_by_name(&w, true, true);

        w
    })
}

///Runs the main window. Intended to run in the main thread.
pub fn run_main_window() {
    get_or_init_main_window().run().unwrap();
}
