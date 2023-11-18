use qdfm::drives;
use qdfm::ui::*;
use slint::VecModel;
use std::rc::Rc;

fn main() {
    let main: MainWindow = MainWindow::new().unwrap();

    //Initialization sequence
    {
        let drives: Rc<VecModel<SidebarItem>> = drives::get_drives();
        main.global::<SidebarItems>().set_drive_list(drives.into());
    }
    main.run().unwrap();
}
