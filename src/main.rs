use qdfm::ui::*;
use qdfm::ui;

fn main() {
    //Use winit
    let backend = i_slint_backend_winit::Backend::new().unwrap();
    slint::platform::set_platform(Box::new(backend)).unwrap();

    start_ui_listener();

    //TODO: Review what can be done in threads

    //Initialization sequence
    ui::send_message(UIMessage::SetCurrentTabFile(
        TabItem {
            internal_path: "/".into(),
            text: "".into(),
            selected: false,
            text_length: 1,
        },
        false,
    ));

    main_window::run_main_window();
}
