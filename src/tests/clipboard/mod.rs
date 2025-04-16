use crate::ui::MainWindow;
use std::{
    fs::{create_dir_all, remove_dir_all, set_permissions, File, Permissions},
    io::{Read, Write},
    os::{
        linux::fs::MetadataExt,
        unix::fs::{symlink, PermissionsExt},
    },
    path::{Path, PathBuf},
    str::FromStr,
};

//TODO: Add tests accross drives
//And obviously add more edge cases

pub mod copy;
pub mod cut;
pub mod delete;
pub mod paste;

//------------------ MainWindow Singleton --------------------
//Since MainWindow can only ever be created once
//Tests MUST be run with 1 max thread or this will not go well

pub struct MainWindowWrapper {
    mw: *mut MainWindow,
}
unsafe impl Sync for MainWindowWrapper {}

pub static mut MAINWINDOW: Option<MainWindowWrapper> = None;

pub fn get_mainwindow() -> &'static mut MainWindow {
    unsafe {
        if MAINWINDOW.is_none() {
            MAINWINDOW = Some(MainWindowWrapper {
                mw: Box::leak(Box::new(MainWindow::new().unwrap())),
            });
        }
        let ptr = MAINWINDOW.as_mut().unwrap().mw;
        return ptr.as_mut().unwrap();
    }
}

///Creates a test directory with various potential cases
///to test other functions with.
pub fn create_test_directory(name: &str, clean: bool) -> PathBuf {
    let path = "/tmp/qdfm_tests/".to_owned() + name;
    if clean {
        std::fs::remove_dir_all("/tmp/qdfm_tests/").ok();
        std::fs::remove_dir_all("/tmp/qdfm_tests/target").ok();
    }
    std::fs::create_dir_all(&path).unwrap();

    //Main directory structure
    //Permissions used for folder comparison
    std::fs::create_dir(path.clone() + "/subfolder1").unwrap();
    std::fs::create_dir(path.clone() + "/subfolder1/another_subfolder").unwrap();
    std::fs::create_dir(path.clone() + "/subfolder2").unwrap();
    std::fs::create_dir(path.clone() + "/subfolder2/empty_subfolder").unwrap();
    std::fs::create_dir(path.clone() + "/subfolder2/nonempty_subfolder").unwrap();

    create_random_file(path.clone() + "/file0");
    create_random_file(path.clone() + "/subfolder1/file1");
    create_random_file(path.clone() + "/subfolder2/nonempty_subfolder/file2");

    symlink(
        path.clone() + "/subfolder2/nonempty_subfolder",
        path.clone() + "/subfolder1/simlink_to_nonempty_subfolder",
    )
    .unwrap();
    symlink(
        path.clone() + "/subfolder2/empty_subfolder",
        path.clone() + "/subfolder1/simlink_to_empty_subfolder",
    )
    .unwrap();
    symlink(
        path.clone() + "/subfolder2/nonempty_subfolder/file2",
        path.clone() + "/subfolder1/simlink_to_a_file",
    )
    .unwrap();
    symlink(path.clone() + "/file0", path.clone() + "/simlink_to_file0").unwrap();

    path.into()
}

pub fn create_target_directory() -> PathBuf {
    let path = PathBuf::from_str("/tmp/qdfm_tests/target").unwrap();
    if !path.exists() {
        create_dir_all(&path).unwrap();
    } else {
    }
    path
}
pub fn create_empty_target_directory() -> PathBuf {
    let path = PathBuf::from_str("/tmp/qdfm_tests/target").unwrap();
    if path.exists() {
        remove_dir_all(&path).unwrap()
    }
    create_dir_all(&path).unwrap();
    path
}

pub fn assert_files_eq(source: &Path, target: &Path) {
    let mut source_bytes = [0; 200];
    let mut target_bytes = [0; 200];
    //Read source
    File::open(&source)
        .unwrap()
        .read_exact(&mut source_bytes)
        .unwrap();

    //Read original
    File::open(&target)
        .unwrap()
        .read_exact(&mut target_bytes)
        .unwrap();

    //Compare
    assert_eq!(source_bytes, target_bytes);
}
pub fn assert_files_ne(source: &Path, target: &Path) {
    let mut source_bytes = [0; 200];
    let mut target_bytes = [0; 200];
    //Read source
    File::open(&source)
        .unwrap()
        .read_exact(&mut source_bytes)
        .unwrap();

    //Read original
    File::open(&target)
        .unwrap()
        .read_exact(&mut target_bytes)
        .unwrap();

    //Compare
    assert_ne!(source_bytes, target_bytes);
}

///Makes sure that the directory at the given path is an unmodified directory created by
///create_test_directory
pub fn verify_untouched_test_directory(path: PathBuf) {
    let path = path.to_str().unwrap().to_owned();
    //----------------------------------- FOLDERS ------------------------------------
    assert_eq!(Path::new(&(path.clone() + "/subfolder1")).exists(), true);
    assert_eq!(Path::new(&(path.clone() + "/subfolder1")).is_dir(), true);
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder1")).is_symlink(),
        false
    );

    assert_eq!(
        Path::new(&(path.clone() + "/subfolder1/another_subfolder")).exists(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder1/another_subfolder")).is_dir(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder1/another_subfolder")).is_symlink(),
        false
    );
    assert_eq!(Path::new(&(path.clone() + "/subfolder2")).exists(), true);
    assert_eq!(Path::new(&(path.clone() + "/subfolder2")).is_dir(), true);
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder2")).is_symlink(),
        false
    );

    assert_eq!(
        Path::new(&(path.clone() + "/subfolder2/empty_subfolder")).exists(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder2/empty_subfolder")).is_dir(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder2/empty_subfolder")).is_symlink(),
        false
    );

    assert_eq!(
        Path::new(&(path.clone() + "/subfolder2/nonempty_subfolder")).exists(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder2/nonempty_subfolder")).is_dir(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder2/nonempty_subfolder")).is_symlink(),
        false
    );

    //-------------------------------- FILES --------------------------------------
    assert_eq!(Path::new(&(path.clone() + "/file0")).exists(), true);
    assert_eq!(Path::new(&(path.clone() + "/file0")).is_file(), true);
    assert_eq!(Path::new(&(path.clone() + "/file0")).is_symlink(), false);

    assert_eq!(
        Path::new(&(path.clone() + "/subfolder1/file1")).exists(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder1/file1")).is_file(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder1/file1")).is_symlink(),
        false
    );

    assert_eq!(
        Path::new(&(path.clone() + "/subfolder2/nonempty_subfolder/file2")).exists(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder2/nonempty_subfolder/file2")).is_file(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder2/nonempty_subfolder/file2")).is_symlink(),
        false
    );

    //-------------------------------------- SIMLINKS ------------------------------------
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder1/simlink_to_nonempty_subfolder")).exists(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder1/simlink_to_nonempty_subfolder")).is_dir(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder1/simlink_to_nonempty_subfolder")).is_symlink(),
        true
    );

    assert_eq!(
        Path::new(&(path.clone() + "/subfolder1/simlink_to_empty_subfolder")).exists(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder1/simlink_to_empty_subfolder")).is_dir(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder1/simlink_to_empty_subfolder")).is_symlink(),
        true
    );

    assert_eq!(
        Path::new(&(path.clone() + "/subfolder1/simlink_to_a_file")).exists(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder1/simlink_to_a_file")).is_file(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/subfolder1/simlink_to_a_file")).is_symlink(),
        true
    );

    assert_eq!(
        Path::new(&(path.clone() + "/simlink_to_file0")).exists(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/simlink_to_file0")).is_file(),
        true
    );
    assert_eq!(
        Path::new(&(path.clone() + "/simlink_to_file0")).is_symlink(),
        true
    );
}

pub fn create_random_file(path: String) -> PathBuf {
    let mut f = File::create_new(&path).unwrap();
    let mut buf = [0; 200];
    File::open("/dev/urandom")
        .unwrap()
        .read_exact(&mut buf)
        .unwrap();
    f.write_all(&buf).unwrap();
    PathBuf::from_str(&path).unwrap()
}
