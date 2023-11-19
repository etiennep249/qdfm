use once_cell::sync::Lazy;
use qdfm::callbacks::sidebar::sidebar_item_clicked;
use qdfm::drives;
use qdfm::ui::*;

fn main() {
    let w: MainWindow = MainWindow::new().unwrap();

    //Initialization sequence
    {
        let drives = drives::get_drives();
        w.global::<SidebarItems>().set_drive_list(drives.into());
    }
    //Callbacks
    {
        let weak = w.as_weak();
        let sidebaritems = w.global::<SidebarItems>();
        sidebaritems.on_drive_clicked(move |i| sidebar_item_clicked(i, weak.clone()));
    }
    w.run().unwrap();
}
