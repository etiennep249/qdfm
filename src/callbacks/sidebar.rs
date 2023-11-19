use slint::{SharedString, Weak};

use crate::ui::*;

pub fn sidebar_item_clicked(item: SidebarItem, weak: Weak<MainWindow>) {
    let path = item.internal_path.clone();
    let text = if item.internal_path == "/" {
        SharedString::from("/")
    } else {
        match item.internal_path.rsplit_once("/") {
            None => item.internal_path,
            Some(e) => e.1.into(),
        }
    };

    weak.upgrade_in_event_loop(move |w| {
        let adapter = w.global::<TabsAdapter>();
        adapter.invoke_change_current_tab(TabItem {
            internal_path: path,
            text_length: text.len() as i32,
            text,
        });
    })
    .unwrap();
}
