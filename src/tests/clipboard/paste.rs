use crate::{
    clipboard::{
        copy::copy_file,
        cut::cut_file,
        paste::{paste_file, paste_file_with_checks, paste_folder_with_checks},
    },
    core::{empty_file, empty_file_with_path, generate_files_for_path},
    rename_window::{setup_rename_window, RenameOption, RenameWindowReturn},
    ui::FileItem,
};
use arboard::Clipboard;
use slint::ComponentHandle;
use std::{
    collections::VecDeque,
    fs::remove_file,
    os::unix::fs::symlink,
    path::Path,
    thread::{sleep, sleep_ms},
    time::Duration,
};
use std::{rc::Rc, sync::mpsc::channel};

use super::{
    assert_files_eq, assert_files_ne, create_empty_target_directory, create_random_file,
    create_target_directory, create_test_directory, get_mainwindow,
    verify_untouched_test_directory,
};

//If Some, whatever is set here will be returned by show_rename_window() during tests
pub static mut RENAME_WINDOW_RETURN: Option<RenameWindowReturn> = None;

#[test]
pub fn test_paste_file() {
    println!("\nTest 1 - Basic copy-paste of the whole test directory");
    let source_dir = create_test_directory("paste_file", true);
    let target_dir = create_empty_target_directory();
    copy_file(
        vec![empty_file_with_path(source_dir.to_str().unwrap().into())],
        false,
    );
    let clip = Clipboard::new().unwrap().get_text().unwrap();
    paste_file(target_dir.clone(), Rc::new(get_mainwindow().as_weak()));
    verify_untouched_test_directory(target_dir.join("paste_file"));
    assert_eq!(source_dir.exists(), true);

    println!("Test 2 - Multi select (2 files, 2 directories, 2 symlinks)");
    let source_dir1 = create_test_directory("paste_file1", true);
    let source_dir2 = create_test_directory("paste_file2", false);
    let source_file1 = create_random_file("/tmp/qdfm_tests/file1".into());
    let source_file2 = create_random_file("/tmp/qdfm_tests/file2".into());
    symlink(&source_file1, "/tmp/qdfm_tests/link1");
    symlink(&source_file2, "/tmp/qdfm_tests/link2");
    let target_dir = create_empty_target_directory();

    copy_file(
        vec![
            empty_file_with_path(source_dir1.to_str().unwrap().into()),
            empty_file_with_path(source_dir2.to_str().unwrap().into()),
            empty_file_with_path(source_file1.to_str().unwrap().into()),
            empty_file_with_path(source_file2.to_str().unwrap().into()),
            empty_file_with_path("/tmp/qdfm_tests/link1".into()),
            empty_file_with_path("/tmp/qdfm_tests/link2".into()),
        ],
        false,
    );
    paste_file(target_dir.clone(), Rc::new(get_mainwindow().as_weak()));

    verify_untouched_test_directory(target_dir.join("paste_file1"));
    verify_untouched_test_directory(target_dir.join("paste_file2"));
    assert_files_eq(
        &source_file1,
        &target_dir.join(source_file1.file_name().unwrap()),
    );
    assert_files_eq(
        &source_file2,
        &target_dir.join(source_file2.file_name().unwrap()),
    );
    assert_eq!(Path::new("/tmp/qdfm_tests/link1").is_symlink(), true);
    assert_eq!(
        Path::new("/tmp/qdfm_tests/link1").read_link().unwrap(),
        source_file1
    );
    assert_eq!(Path::new("/tmp/qdfm_tests/link2").is_symlink(), true);
    assert_eq!(
        Path::new("/tmp/qdfm_tests/link2").read_link().unwrap(),
        source_file2
    );

    println!("Test 3 - Same as test 2, but with cut instead of copy");
    let source_dir1 = create_test_directory("paste_file1", true);
    let source_dir2 = create_test_directory("paste_file2", false);
    let source_file1 = create_random_file("/tmp/qdfm_tests/file1".into());
    let source_file2 = create_random_file("/tmp/qdfm_tests/file2".into());
    symlink(&source_file1, "/tmp/qdfm_tests/link1");
    symlink(&source_file2, "/tmp/qdfm_tests/link2");
    let target_dir = create_empty_target_directory();

    cut_file(vec![
        empty_file_with_path(source_dir1.to_str().unwrap().into()),
        empty_file_with_path(source_dir2.to_str().unwrap().into()),
        empty_file_with_path(source_file1.to_str().unwrap().into()),
        empty_file_with_path(source_file2.to_str().unwrap().into()),
        empty_file_with_path("/tmp/qdfm_tests/link1".into()),
        empty_file_with_path("/tmp/qdfm_tests/link2".into()),
    ]);
    paste_file(target_dir.clone(), Rc::new(get_mainwindow().as_weak()));

    verify_untouched_test_directory(target_dir.join("paste_file1"));
    verify_untouched_test_directory(target_dir.join("paste_file2"));
    assert_eq!(source_dir1.exists(), false);
    assert_eq!(source_dir2.exists(), false);
    assert_eq!(source_file1.exists(), false);
    assert_eq!(source_file2.exists(), false);
    assert_eq!(
        target_dir.join(source_file1.file_name().unwrap()).exists(),
        true
    );
    assert_eq!(
        target_dir.join(source_file2.file_name().unwrap()).exists(),
        true
    );
    assert_eq!(Path::new("/tmp/qdfm_tests/link1").is_symlink(), true);
    assert_eq!(
        Path::new("/tmp/qdfm_tests/link1").read_link().unwrap(),
        source_file1
    );
    assert_eq!(Path::new("/tmp/qdfm_tests/link2").is_symlink(), true);
    assert_eq!(
        Path::new("/tmp/qdfm_tests/link2").read_link().unwrap(),
        source_file2
    );
}

//TODO: Test if rename target is invalid
#[test]
pub fn test_paste_file_with_checks() {
    let source_dir = create_test_directory("paste_file_with_checks", true);
    let target_dir = create_target_directory();
    let source = source_dir.join("file0");
    let source_alt = source_dir.join("subfolder1").join("file1");
    let target = target_dir.join("file0copy");

    let mut current = 0;
    let mut speed_vec = VecDeque::new();
    let mut avg_speed = 0f64;
    let mut remaining_time = 0f64;
    let mut all_success = false;
    let mut apply_to_all = false;
    let mut apply_to_all_option = RenameOption::Rename;

    let rename_win = setup_rename_window(get_mainwindow().as_weak());
    let (progress, _) = channel();

    //Test 1 - No Overwrite, full copy
    paste_file_with_checks(
        &source,
        &target,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
        false,
    );
    assert_files_eq(&source, &target);

    //Test 2 - No Overwrite, full copy
    paste_file_with_checks(
        &source_alt,
        &target,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
        false,
    );
    assert_files_ne(&source_alt, &target);
    assert_files_eq(&source, &target);

    //Test 3 - Overwrite, full copy
    paste_file_with_checks(
        &source_alt,
        &target,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        true,
        false,
    );
    assert_files_ne(&source, &target);
    assert_files_eq(&source_alt, &target);

    //Test 4 - No Overwrite, rename
    remove_file(&target).unwrap();
    paste_file_with_checks(
        &source_alt,
        &target,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
        true,
    );
    assert_eq!(source_alt.exists(), false);
    assert_eq!(target.exists(), true);

    //------------------------ TESTS WITH RenameOption BUT NO APPLY_TO_ALL --------------

    //Test 5 - User presses overwrite but not apply_to_all
    create_test_directory("paste_file_with_checks", true);
    unsafe {
        RENAME_WINDOW_RETURN = Some(RenameWindowReturn {
            filename: None,
            option: RenameOption::Overwrite,
            apply_to_all: false,
        })
    }

    paste_file_with_checks(
        &source_alt,
        &source,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
        false,
    );
    assert_files_eq(&source, &source_alt);

    //Test 6 - User chooses ignore, but not apply_to_all
    create_test_directory("paste_file_with_checks", true);
    unsafe {
        RENAME_WINDOW_RETURN = Some(RenameWindowReturn {
            filename: None,
            option: RenameOption::Ignore,
            apply_to_all: false,
        })
    }
    paste_file_with_checks(
        &source_alt,
        &source,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
        false,
    );
    assert_files_ne(&source, &source_alt);

    //Test 7 - User chooses rename, but not apply_to_all
    create_test_directory("paste_file_with_checks", true);
    let new_filename = "new_filename".to_string();
    unsafe {
        RENAME_WINDOW_RETURN = Some(RenameWindowReturn {
            filename: Some(new_filename.clone()),
            option: RenameOption::Rename,
            apply_to_all: false,
        })
    }
    paste_file_with_checks(
        &source,
        &source_alt,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
        false,
    );
    assert_files_ne(&source, &source_alt);
    let mut new_target = source_alt.clone();
    new_target.set_file_name(new_filename);
    assert_files_eq(&source, &new_target);

    //------------------------ TESTS WITH RenameOption AND APPLY_TO_ALL --------------

    //Test 8 - User chooses rename and apply_to_all
    create_test_directory("paste_file_with_checks", true);
    let new_filename = "new_filename".to_string();
    apply_to_all = false;
    unsafe {
        RENAME_WINDOW_RETURN = Some(RenameWindowReturn {
            filename: Some(new_filename.clone()),
            option: RenameOption::Rename,
            apply_to_all: true,
        })
    }
    paste_file_with_checks(
        &source,
        &source_alt,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
        false,
    );
    //First rename should have worked
    assert_files_ne(&source, &source_alt);
    let mut new_target = source_alt.clone();
    new_target.set_file_name(&new_filename);
    assert_files_eq(&source, &new_target);

    assert_eq!(apply_to_all, true);
    assert_eq!(
        apply_to_all_option.clone() as u64,
        RenameOption::Rename as u64
    );

    unsafe {
        RENAME_WINDOW_RETURN = Some(RenameWindowReturn {
            filename: Some(new_filename.clone()),
            option: RenameOption::Rename,
            apply_to_all: false,
        })
    }
    create_test_directory("paste_file_with_checks", true);
    paste_file_with_checks(
        &source,
        &source_alt,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
        false,
    );

    //Since apply_to_all doesn't apply to rename, the second call to paste_file_with_checks should
    //have prompted the user again, and since we set apply_to_all=false, it will now be false
    assert_eq!(apply_to_all, false);

    //Test 9 - User chooses overwrite and apply_to_all
    create_test_directory("paste_file_with_checks", true);
    apply_to_all = false;
    unsafe {
        RENAME_WINDOW_RETURN = Some(RenameWindowReturn {
            filename: None,
            option: RenameOption::Overwrite,
            apply_to_all: true,
        })
    }
    paste_file_with_checks(
        &source,
        &source_alt,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
        false,
    );
    //First copy should have worked
    assert_files_eq(&source, &source_alt);

    //This should have *zero* effect because of the previous apply_to_all
    unsafe {
        RENAME_WINDOW_RETURN = Some(RenameWindowReturn {
            filename: None,
            option: RenameOption::Ignore,
            apply_to_all: true,
        })
    }

    create_test_directory("paste_file_with_checks", true);
    paste_file_with_checks(
        &source,
        &source_alt,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
        false,
    );
    //Despite having spoofed the return to ignore, the rename window should not have shown
    //And therefore apply_to_all will overwrite this again.
    assert_files_eq(&source, &source_alt);

    //Test 10 - User chooses ignore and apply_to_all
    create_test_directory("paste_file_with_checks", true);
    apply_to_all = false;
    unsafe {
        RENAME_WINDOW_RETURN = Some(RenameWindowReturn {
            filename: None,
            option: RenameOption::Ignore,
            apply_to_all: true,
        })
    }
    paste_file_with_checks(
        &source,
        &source_alt,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
        false,
    );
    //First copy should have been ignored
    assert_files_ne(&source, &source_alt);

    //This should have *zero* effect because of the previous apply_to_all
    unsafe {
        RENAME_WINDOW_RETURN = Some(RenameWindowReturn {
            filename: None,
            option: RenameOption::Overwrite,
            apply_to_all: true,
        })
    }

    create_test_directory("paste_file_with_checks", true);
    paste_file_with_checks(
        &source,
        &source_alt,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
        false,
    );
    //Despite having spoofed the return to overwrite, the rename window should not have shown
    //And therefore apply_to_all will ignore this again.
    assert_files_ne(&source, &source_alt);
}

#[test]
///This one is a bit harder to test as we can't really compare two directories without comparing
///their content, and paste_folder_with_checks doesn't copy contents.
pub fn test_paste_folder_with_checks() {
    let source_dir = create_test_directory("paste_folder_with_checks", true);
    let target_dir = create_target_directory();
    let source = source_dir.join("subfolder1");
    let source_alt = source_dir.join("subfolder2/nonempty_subfolder");
    let target = target_dir.join("subfolder1_copy");

    let mut current = 0;
    let mut speed_vec = VecDeque::new();
    let mut avg_speed = 0f64;
    let mut remaining_time = 0f64;
    let mut all_success = false;
    let mut apply_to_all = false;
    let mut apply_to_all_option = RenameOption::Rename;

    let rename_win = setup_rename_window(get_mainwindow().as_weak());
    let (progress, _) = channel();

    //Test 1 - Basic
    let mut target_mod = target.clone();
    paste_folder_with_checks(
        &source,
        &mut target_mod,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
    )
    .unwrap();
    assert_eq!(target_mod.exists(), true);

    //------------------------ TESTS WITH RenameOption BUT NO APPLY_TO_ALL --------------

    //Test 5 - User presses overwrite but not apply_to_all
    create_test_directory("paste_folder_with_checks", true);
    unsafe {
        RENAME_WINDOW_RETURN = Some(RenameWindowReturn {
            filename: None,
            option: RenameOption::Overwrite,
            apply_to_all: false,
        })
    }
    let mut src_mut = source.clone();
    paste_folder_with_checks(
        &source_alt,
        &mut src_mut,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
    )
    .unwrap();
    assert_eq!(src_mut.exists(), true);
    assert_eq!(source_alt.exists(), true);

    //Test 6 - User chooses ignore, but not apply_to_all
    create_test_directory("paste_folder_with_checks", true);
    let mut src_mut = source.clone();
    unsafe {
        RENAME_WINDOW_RETURN = Some(RenameWindowReturn {
            filename: None,
            option: RenameOption::Ignore,
            apply_to_all: false,
        })
    }
    assert_eq!(
        paste_folder_with_checks(
            &source_alt,
            &mut src_mut,
            0,
            &mut current,
            &mut speed_vec,
            &mut avg_speed,
            &mut remaining_time,
            &mut all_success,
            &progress,
            &rename_win,
            &mut apply_to_all,
            &mut apply_to_all_option,
            false,
        )
        .is_err(),
        true
    );

    //Test 7 - User chooses rename, but not apply_to_all
    create_test_directory("paste_folder_with_checks", true);
    let new_filename = "new_filename".to_string();
    unsafe {
        RENAME_WINDOW_RETURN = Some(RenameWindowReturn {
            filename: Some(new_filename.clone()),
            option: RenameOption::Rename,
            apply_to_all: false,
        })
    }
    let mut src_alt_mut = source_alt.clone();
    paste_folder_with_checks(
        &source,
        &mut src_alt_mut,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
    );

    assert_eq!(source.exists(), true);
    assert_eq!(src_alt_mut.exists(), true);
    let mut new_target = source_alt.clone();
    new_target.set_file_name(new_filename);
    assert_eq!(
        src_alt_mut
            .to_string_lossy()
            .to_string()
            .eq(&new_target.to_string_lossy().to_string()),
        true
    );
    assert_eq!(
        src_alt_mut
            .to_string_lossy()
            .to_string()
            .eq(&source_alt.to_string_lossy().to_string()),
        false
    );

    //------------------------ TESTS WITH RenameOption AND APPLY_TO_ALL --------------

    //Test 8 - User chooses rename and apply_to_all
    create_test_directory("paste_folder_with_checks", true);
    let new_filename = "new_filename".to_string();
    apply_to_all = false;
    unsafe {
        RENAME_WINDOW_RETURN = Some(RenameWindowReturn {
            filename: Some(new_filename.clone()),
            option: RenameOption::Rename,
            apply_to_all: true,
        })
    }
    let mut src_alt_mut = source_alt.clone();
    paste_folder_with_checks(
        &source,
        &mut src_alt_mut,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
    );
    //First rename should have worked
    assert_eq!(source.exists(), true);
    assert_eq!(src_alt_mut.exists(), true);
    let mut new_target = source_alt.clone();
    new_target.set_file_name(new_filename.clone());
    assert_eq!(
        src_alt_mut
            .to_string_lossy()
            .to_string()
            .eq(&new_target.to_string_lossy().to_string()),
        true
    );
    assert_eq!(
        src_alt_mut
            .to_string_lossy()
            .to_string()
            .eq(&source_alt.to_string_lossy().to_string()),
        false
    );

    assert_eq!(apply_to_all, true);
    assert_eq!(
        apply_to_all_option.clone() as u64,
        RenameOption::Rename as u64
    );

    unsafe {
        RENAME_WINDOW_RETURN = Some(RenameWindowReturn {
            filename: Some(new_filename.clone()),
            option: RenameOption::Rename,
            apply_to_all: false,
        })
    }

    let mut src_alt_mut = source_alt.clone();
    create_test_directory("paste_folder_with_checks", true);
    paste_folder_with_checks(
        &source,
        &mut src_alt_mut,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
    );

    //Since apply_to_all doesn't apply to rename, the second call to paste_file_with_checks should
    //have prompted the user again, and since we set apply_to_all=false, it will now be false
    assert_eq!(apply_to_all, false);

    //Test 10 - User chooses ignore and apply_to_all
    create_test_directory("paste_folder_with_checks", true);
    apply_to_all = false;
    unsafe {
        RENAME_WINDOW_RETURN = Some(RenameWindowReturn {
            filename: None,
            option: RenameOption::Ignore,
            apply_to_all: true,
        })
    }
    let mut target_mod = target.clone();
    paste_folder_with_checks(
        &source,
        &mut target_mod,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
    );
    //First copy should have been ignored
    assert_eq!(target_mod.exists(), false);

    //This should have *zero* effect because of the previous apply_to_all

    unsafe {
        RENAME_WINDOW_RETURN = Some(RenameWindowReturn {
            filename: Some(new_filename.clone()),
            option: RenameOption::Rename,
            apply_to_all: false,
        })
    }

    create_test_directory("paste_folder_with_checks", true);

    let mut target_mod = target.clone();
    paste_folder_with_checks(
        &source,
        &mut target_mod,
        0,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        &mut all_success,
        &progress,
        &rename_win,
        &mut apply_to_all,
        &mut apply_to_all_option,
        false,
    );
    //Despite having spoofed the return to rename, the rename window should not have shown
    //And so the target_mod name won't have been renamed
    let mut new_target = target.clone();
    new_target.set_file_name(new_filename.clone());
    assert_eq!(
        target_mod
            .to_string_lossy()
            .to_string()
            .eq(&new_target.to_string_lossy().to_string()),
        false
    );
}
