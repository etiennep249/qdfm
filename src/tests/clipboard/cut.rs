use std::sync::Mutex;

use arboard::Clipboard;

use crate::{
    clipboard::{cut::cut_file, CUT_BUFFER},
    ui::{FileItem, _i64},
};

#[test]
pub fn test_cut() {
    let path1 = "/test1/test2/test3/test4/test5.txt";
    let path2 = "/test5/test4/test3/test2/test1.txt";
    let item1 = FileItem {
        date: _i64 { a: 0, b: 0 },
        extension: "txt".into(),
        file_name: "text_file_1.txt".into(),
        file_type: "txt".into(),
        is_dir: false,
        is_link: false,
        size: _i64 { a: 0, b: 0 },
        path: path1.into(),
    };
    let item2 = FileItem {
        date: _i64 { a: 0, b: 0 },
        extension: "txt".into(),
        file_name: "text_file_2.txt".into(),
        file_type: "txt".into(),
        is_dir: false,
        is_link: false,
        size: _i64 { a: 0, b: 0 },
        path: path2.into(),
    };
    let files = vec![item1.clone(), item2.clone()];

    cut_file(files);

    //Check regular clipboard first
    let pfx = "file://".to_string();
    let clip = Clipboard::new().unwrap().get().text().unwrap();
    let clip_should_be = pfx.clone() + &path1 + "\n" + &pfx + &path2 + "\n";
    assert_eq!(clip, clip_should_be);

    //Then check CUT_BUFFER
    let buf = CUT_BUFFER
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap();
    assert_eq!(buf[0], item1);
    assert_eq!(buf[1], item2);
}
