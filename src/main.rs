use qdfm::callbacks::*;
use qdfm::core::generate_files_for_path;
use qdfm::drives;
use qdfm::globals::config_lock;
use qdfm::ui::*;
use slint::{Model, VecModel};
use std::rc::Rc;
//https://github.com/rust-lang/rfcs/issues/2407#issuecomment-385291238
//Replace with https://github.com/rust-lang/rfcs/pull/3512
//When/if it gets merged
macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

fn main() {
    let w: MainWindow = MainWindow::new().unwrap();
    let weak = Rc::new(w.as_weak());
    //Initialization sequence
    {
        let conf = config_lock();
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
        w.global::<TabsAdapter>().set_breadcrumbs(
            [TabItem {
                internal_path: "/".into(),
                text: "".into(),
                selected: false,
                text_length: 1,
            }]
            .into(),
        );
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
        w.global::<FileManager>().on_fileitem_clicked(
            enclose! { (weak) move |i| filemanager::fileitem_clicked(i, weak.clone())},
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
    }
    w.run().unwrap();
}
