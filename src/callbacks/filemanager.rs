use crate::core;
use crate::ui::*;
use crate::utils::doubleclicks::check_for_dclick;
use slint::SharedString;
use slint::VecModel;
use slint::Weak;
use std::rc::Rc;

//For now, there is no double click handler, CHANGE TO DOUBLE CLICK WHEN/IF IT'S IMPLEMENTED
pub fn fileitem_clicked(item: FileItem, mw: Weak<MainWindow>) {
    if check_for_dclick() {
        fileitem_doubleclicked(item, mw);
    }
}

pub fn fileitem_doubleclicked(item: FileItem, mw: Weak<MainWindow>) {
    set_current_tab_file(item, mw);
}

pub fn set_current_tab_file(item: FileItem, mw: Weak<MainWindow>) {
    let path = item.path.clone();
    let text = if item.path == "/" {
        SharedString::from("/")
    } else {
        match item.path.rsplit_once("/") {
            None => item.path.clone(),
            Some(e) => e.1.into(),
        }
    };
    let files = core::generate_files_for_path(item.path.as_str());
    mw.upgrade_in_event_loop(move |w| {
        w.global::<TabsAdapter>()
            .invoke_change_current_tab(TabItem {
                internal_path: path,
                text_length: text.len() as i32,
                text,
            });
        w.global::<FileManager>()
            .set_files(Rc::new(VecModel::from(files)).into());
    })
    .unwrap();
}
