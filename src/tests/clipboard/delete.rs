use slint::ComponentHandle;
use std::{collections::HashMap, rc::Rc};

use crate::{
    callbacks::filemanager::selection::selected_files_write_tests, clipboard::delete::delete,
    core::empty_file, tests::clipboard::get_mainwindow,
};

use super::create_test_directory;

#[test]
pub fn test_delete() {
    let rc = Rc::new(get_mainwindow().as_weak());

    let path = create_test_directory("delete", true);

    let mut file1 = empty_file();
    file1.path = path
        .join("subfolder1")
        .join("file1")
        .to_string_lossy()
        .to_string()
        .into();
    let mut file2_simlink = empty_file();
    file2_simlink.path = path
        .join("subfolder1")
        .join("simlink_to_a_file")
        .to_string_lossy()
        .to_string()
        .into();
    let mut folder = empty_file();
    folder.is_dir = true;
    folder.path = path.join("subfolder1").to_string_lossy().to_string().into();

    *(selected_files_write_tests()) = HashMap::from([(0, file1), (1, file2_simlink)]);

    delete(rc.clone());

    assert_eq!(path.join("subfolder1").join("file1").exists(), false);
    assert_eq!(
        path.join("subfolder1").join("simlink_to_a_file").exists(),
        false
    );
    assert_eq!(
        path.join("subfolder2")
            .join("nonempty_subfolder")
            .join("file2")
            .exists(),
        true
    );

    //Delete everything
    *(selected_files_write_tests()) = HashMap::from([(0, folder)]);

    delete(rc);

    assert_eq!(path.join("subfolder1").exists(), false);
}
