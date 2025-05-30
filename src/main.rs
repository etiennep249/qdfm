use filemanager::set_current_tab_file;
use i_slint_backend_winit::{WinitWindowAccessor, WinitWindowEventResult};
use qdfm::callbacks::utils::format_size_detailed;
use qdfm::core::generate_files_for_path;
use qdfm::drives;
use qdfm::globals::config_lock;
use qdfm::ui::*;
use qdfm::utils::drag_and_drop::{dnd_move, dnd_press, dnd_release, move_file, xdnd_init};
use qdfm::utils::error_handling::log_error_str;
use qdfm::{callbacks::*, enclose};
use slint::VecModel;
use std::rc::Rc;
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
        set_current_tab_file(
            TabItem {
                internal_path: "/".into(),
                text: "".into(),
                selected: false,
                text_length: 1,
            },
            weak.clone(),
            false,
        );

        //Default sort
        //TODO use config
        qdfm::sort::sort_by_name(weak.clone(), true, true);

        conf.init_mappings();
        let window_id = w.window().with_winit_window(|w| {
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
                log_error_str("Could not get the window handle. Things may not work.");
            }
            0
        });

        if window_id.is_some() {
            xdnd_init(window_id.unwrap());
        }
    }

    // Listen to window events
    {
        let weak = weak.clone();
        w.window()
            .on_winit_window_event(move |_, we: &WindowEvent| -> WinitWindowEventResult {
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
                    _ => {}
                }
                WinitWindowEventResult::Propagate
            });
    }

    //Callbacks
    {
        let sidebaritems = w.global::<SidebarItems>();
        sidebaritems.on_drive_clicked(
            enclose! { (weak) move |i| sidebar::sidebar_item_clicked(i, weak.clone())},
        );
        sidebaritems.on_left_arrow_clicked(
            enclose! { (weak) move || sidebar::left_arrow_clicked(weak.clone())},
        );
        sidebaritems.on_right_arrow_clicked(
            enclose! { (weak) move || sidebar::right_arrow_clicked(weak.clone())},
        );
        w.global::<FileManager>().on_fileitem_doubleclicked(
            enclose! { (weak) move |file, i| filemanager::fileitem_doubleclicked(file, i, weak.clone())},
        );
        let tabs_adapter = w.global::<TabsAdapter>();
        tabs_adapter.on_breadcrumb_clicked(
            enclose! { (weak) move |i| tabs::breadcrumb_clicked(i, weak.clone())},
        );
        tabs_adapter.on_breadcrumb_accepted(
            enclose! { (weak) move |s| tabs::breadcrumb_accepted(s, weak.clone())},
        );
        w.global::<ColumnHeadersAdapter>().on_header_clicked(
            enclose! { (weak) move |header| headers::on_header_click(header, weak.clone())},
        );
        w.global::<ColumnHeadersAdapter>().on_adjust_size(
            enclose! { (weak) move |header, offset, original| headers::on_header_resize(header, offset, original, weak.clone())},
        );

        let file_manager = w.global::<FileManager>();
        file_manager.on_format_size(move |i| filemanager::format_size(i));
        file_manager.on_pressed(enclose! { (weak) move || dnd_press(weak.clone())});
        file_manager.on_released(enclose! { (weak) move || dnd_release(weak.clone())});
        file_manager.on_moved(enclose! { (weak) move |x, y| dnd_move(weak.clone(), x, y)});
        file_manager.on_format_date(move |i| filemanager::format_date(i));
        file_manager.on_add_to_selected(
            enclose! { (weak) move |i, f| filemanager::add_to_selected(weak.clone(), i, f)},
        );
        file_manager.on_is_index_selected(|i| filemanager::is_index_selected(i));
        file_manager.on_remove_from_selected(
            enclose! { (weak) move |i| filemanager::remove_from_selected(weak.clone(), i)},
        );
        file_manager.on_reset_selected(
            enclose! { (weak) move || filemanager::reset_selected(weak.clone())},
        );
        file_manager.on_set_single_selected(
            enclose! { (weak) move |i, f| filemanager::set_single_selected(weak.clone(),i, f)},
        );
        file_manager
            .on_shift_select(enclose! { (weak) move |i| filemanager::shift_select(weak.clone(),i)});
        file_manager.on_is_nothing_selected(move || filemanager::is_nothing_selected());

        prop_win
            .global::<PropertiesAdapter>()
            .on_format_size_detailed(move |i| format_size_detailed(i));
        prop_win
            .global::<FileManager>()
            .on_format_date(move |i| filemanager::format_date(i));
        prop_win.global::<PropertiesAdapter>().on_ok(
            enclose! { (weak, prop_weak) move || properties::ok(prop_weak.clone(), weak.clone())},
        );
        prop_win
            .global::<PropertiesAdapter>()
            .on_cancel(enclose! { (prop_weak) move || properties::cancel(prop_weak.clone())});
        prop_win
            .global::<PropertiesAdapter>()
            .on_recalculate_bitmask(
                enclose! { (prop_weak) move || properties::recalculate_bitmask(prop_weak.clone())},
            );
        let ctx_adp = w.global::<ContextAdapter>();
        ctx_adp.on_show_context_menu(
            enclose! { (weak) move |x,y| filemanager::show_context_menu(x,y,weak.clone())},
        );
        ctx_adp.on_menuitem_click(
            enclose! { (weak) move |callback_item| context_menu::menuitem_click(callback_item, weak.clone(), prop_win.as_weak())},
        );
        ctx_adp.on_menuitem_hover(
            enclose! { (weak) move |callback_item| context_menu::menuitem_hover(callback_item, weak.clone())},
        );
    }
    w.run().unwrap();
}
