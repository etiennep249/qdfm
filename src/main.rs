use qdfm::drives;
slint::include_modules!();

fn main() {
    let main: MainWindow = MainWindow::new().unwrap();

    //Initialization sequence
    {
        let drives = drives::get_drives();
        main.global::<SidebarItems>().set_drive_list(drives.into());
    }
    main.run().unwrap();
}
