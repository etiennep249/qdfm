use std::collections::VecDeque;

use std::io::{stdout, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread::sleep;
use std::time::Duration;

use arboard::Clipboard;

use crate::clipboard::copy::{copy_file, copy_single_file_operation};
use crate::tests::clipboard::assert_files_eq;
use crate::ui::*;

use super::{create_target_directory, create_test_directory};

#[test]
pub fn test_copy_file() {
    let path1 = "/test1/test2/test3/test4/test5.txt";
    let path2 = "/test5/test4/test3/test2/test1.txt";
    let pfx = "file://".to_string();
    let files: Vec<FileItem> = vec![
        FileItem {
            date: _i64 { a: 0, b: 0 },
            extension: "txt".into(),
            file_name: "text_file_1.txt".into(),
            file_type: "txt".into(),
            is_dir: false,
            is_link: false,
            size: _i64 { a: 0, b: 0 },
            path: path1.into(),
            selected: false,
        },
        FileItem {
            date: _i64 { a: 0, b: 0 },
            extension: "txt".into(),
            file_name: "text_file_2.txt".into(),
            file_type: "txt".into(),
            is_dir: false,
            is_link: false,
            size: _i64 { a: 0, b: 0 },
            path: path2.into(),
            selected: false,
        },
    ];
    copy_file(files, false);
    let clip = Clipboard::new().unwrap().get().text().unwrap();
    let clip_should_be = pfx.clone() + &path1 + "\n" + &pfx + &path2 + "\n";
    assert_eq!(clip, clip_should_be);
}

#[test]
pub fn test_copy_single_file_operation() {
    let path = create_test_directory("copy_single_file_operation", true);
    let source = path.join("file0");
    let target = create_target_directory().join("test_file_copy");

    let mut current = 0;
    let mut speed_vec = VecDeque::new();
    let mut avg_speed = 0f64;
    let mut remaining_time = 0f64;
    let total = 0;
    let mut all_success = false;
    let (progress, _) = channel();

    //Test 1 - Copy
    copy_single_file_operation(
        target.clone(),
        &source,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        total,
        &mut all_success,
        &progress,
        false,
    )
    .err(); //Always err because our channel isn't listening

    assert_files_eq(&source, &target);

    //Test 2 - Rename
    copy_single_file_operation(
        target.clone(),
        &source,
        &mut current,
        &mut speed_vec,
        &mut avg_speed,
        &mut remaining_time,
        total,
        &mut all_success,
        &progress,
        true,
    )
    .err();
    assert_eq!(Path::new(&target).exists(), true);
    assert_eq!(Path::new(&source).exists(), false);
    assert_eq!(Path::new(&target).metadata().unwrap().size(), 200);
}
