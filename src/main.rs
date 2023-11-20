use qdfm::callbacks::filemanager::*;
use qdfm::callbacks::sidebar::*;
use qdfm::core::generate_files_for_path;
use qdfm::drives;
use qdfm::globals::config_lock;
use qdfm::ui::*;
use slint::VecModel;
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
            .invoke_setup(conf.get::<String>("theme").unwrap().into());
    }
    //Callbacks
    {
        let sidebaritems = w.global::<SidebarItems>();
        sidebaritems
            .on_drive_clicked(enclose! { (weak) move |i| sidebar_item_clicked(i, weak.clone())});
        sidebaritems
            .on_left_arrow_clicked(enclose! { (weak) move || left_arrow_clicked(weak.clone())});
        sidebaritems
            .on_right_arrow_clicked(enclose! { (weak) move || right_arrow_clicked(weak.clone())});
        w.global::<FileManager>()
            .on_fileitem_clicked(enclose! { (weak) move |i| fileitem_clicked(i, weak.clone())});
    }
    w.run().unwrap();
}
