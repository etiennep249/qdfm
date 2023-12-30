use crate::ui::*;
use std::rc::Rc;

/*TODO: IS THIS NEEDED?"*/

use slint::{
    private_unstable_api::re_exports::{EventResult, EventResult::Accept, KeyEvent},
    Weak,
};

pub fn key_event(event: KeyEvent, mw: Rc<Weak<MainWindow>>) -> EventResult {
    //println!("Key pressed was: {:?}", event);
    match &event.text as &str {
        /*ESCAPE*/ "\u{1b}" => escape_pressed(mw),
        _ => {}
    }
    Accept
}

fn escape_pressed(mw: Rc<Weak<MainWindow>>) {
    mw.upgrade_in_event_loop(move |w| {
        w.global::<TabsAdapter>().set_path_shown(false);
    })
    .unwrap();
}

pub fn setup_keybinds() {}
