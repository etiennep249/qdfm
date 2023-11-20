use qdfm::callbacks::filemanager::fileitem_clicked;
use qdfm::callbacks::sidebar::sidebar_item_clicked;
use qdfm::core::generate_files_for_path;
use qdfm::drives;
use qdfm::ui::*;
use slint::VecModel;
use std::rc::Rc;

fn main() {
    let w: MainWindow = MainWindow::new().unwrap();
    let mut weak = w.as_weak();
    //Initialization sequence
    {
        let drives = drives::get_drives();
        w.global::<SidebarItems>().set_drive_list(drives.into());
        w.global::<FileManager>()
            .set_files(Rc::new(VecModel::from(generate_files_for_path("/"))).into());
    }
    //Callbacks
    {
        let sidebaritems = w.global::<SidebarItems>();
        sidebaritems.on_drive_clicked(move |i| sidebar_item_clicked(i, weak.clone()));
        weak = w.as_weak();
        w.global::<FileManager>()
            .on_fileitem_clicked(move |i| fileitem_clicked(i, weak.clone()));
    }
    w.run().unwrap();
}
