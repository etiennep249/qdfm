use crate::{core, ui::*};
use slint::{SharedString, VecModel, Weak};
use std::rc::Rc;

pub fn sidebar_item_clicked(item: SidebarItem, weak: Weak<MainWindow>) {
    let path = item.internal_path.clone();
    let text = if item.internal_path == "/" {
        SharedString::from("/")
    } else {
        match item.internal_path.rsplit_once("/") {
            None => item.internal_path.clone(),
            Some(e) => e.1.into(),
        }
    };
    let files = core::generate_files_for_path(item.internal_path.as_str());
    weak.upgrade_in_event_loop(move |w| {
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
