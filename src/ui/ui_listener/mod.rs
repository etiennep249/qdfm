use internal::{refresh_ui, set_current_tab_file};
use slint::Weak;

use crate::ui::*;
use std::{
    sync::{
        mpsc::{channel, Sender},
        OnceLock,
    },
    thread,
};

mod internal;

//The goal of this is to stop having to move Rc<Weak<MainWindow>> around.
//Instead, functions may call these global static functions to send messages to a single thread and
//scope that owns a single one of those weak references. This also helps limit the length of time
//spend doing possibly expensive operations in the main thread.

pub enum UIMessage {
    Refresh,
    SetCurrentTabFile(TabItem, bool), //(tabitem, remember_in_breadcrumbs)
    HideContextMenu,
}

///Sends a message to the UI thread. There is no guarantee it will be executed immediately.
///Uses the standard MPSC Channel queue system.
pub fn send_message(msg: UIMessage) {
    SENDER.get().unwrap().send(msg).ok();
}

///Runs the given closure with the main window.
pub fn run_with_main_window(func: impl FnOnce(MainWindow) + Send + 'static) {
    if let Some(mw) = MAINWINDOW.get() {
        if let Some(mw) = mw.upgrade() {
            func(mw);
        } else {
            mw.upgrade_in_event_loop(|mw| func(mw)).ok();
        }
    }
}

static SENDER: OnceLock<Sender<UIMessage>> = OnceLock::new();
static MAINWINDOW: OnceLock<Weak<MainWindow>> = OnceLock::new();

pub fn start_ui_listener(mw: Weak<MainWindow>) {
    let (send, recv) = channel();
    SENDER.set(send).ok();
    MAINWINDOW.set(mw).ok();

    thread::spawn(move || {
        while let Ok(msg) = recv.recv() {
            match msg {
                UIMessage::Refresh => run_with_main_window(|mw| refresh_ui(mw)),
                UIMessage::SetCurrentTabFile(item, remember) => run_with_main_window(move |mw| {
                    set_current_tab_file(Some(item.clone()), remember, mw)
                }),

                UIMessage::HideContextMenu => run_with_main_window(move |mw| {
                    mw.invoke_hide_context_menu();
                }),
            }
        }
    });
}
