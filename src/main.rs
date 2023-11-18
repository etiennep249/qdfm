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
        w.global::<SidebarItems>()
            .on_drive_clicked(sidebar_item_clicked);
    }
    w.run().unwrap();
}
