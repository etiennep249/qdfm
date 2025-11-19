use super::error_handling::{log_debug, log_error_str};
use crate::callbacks::filemanager::set_current_tab_file;
use crate::clipboard::file_exists_in_dir;
use crate::globals::{qdfm_win_id, selected_files_lock, set_qdfm_win_id, x_conn_lock};
use crate::ui::*;
use core::{panic, str};

use slint::{ComponentHandle, Weak};
use std::collections::HashMap;
use std::error::Error;
use std::ops::Shl;
use std::rc::Rc;
use std::sync::mpsc::Receiver;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread::spawn;
use std::time::Duration;
use x11rb::protocol::xproto::SELECTION_NOTIFY_EVENT;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ClientMessageEvent, ConnectionExt, EventMask, PropMode, SelectionNotifyEvent,
};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::CURRENT_TIME;

/**
  Called when a file is dropped in the window
  The file is moved from its original location to the current folder
*/
pub fn move_file(mw: Rc<Weak<MainWindow>>, buf: &str, destination: &str) {
    if file_exists_in_dir(destination, buf) == Ok(false) {
        if let Some(filename) = buf.split("/").last() {
            if std::fs::copy(buf, destination.to_owned() + "/" + &filename).is_ok() {
                /*if std::fs::remove_file(buf).is_err() {
                    log_error_str(
                        "Could not remove the original file, so it was copied instead of moved.",
                    );
                }*/
            }
        }
    }
    //Refresh UI
    set_current_tab_file(
        mw.unwrap().global::<TabsAdapter>().invoke_get_current_tab(),
        mw,
        false,
    );
}
//==============================================================================================
//TODO BIG TODO implement drag and drop from qdfm to some other window ON WAYLAND

// This is a very simplified implementation of Xdnd. It probably doesn't work everywhere and
// can probably be broken if you really want to. However, I've been able to make it work everywhere
// I needed it to.
//
// This requires a forked winit to work. Basically, create a channel to send ClientMessages through
// otherwise winit consumes them without letting them through. In winit::platform::mod.rs
// and winit::platform_impl::x11::event_processor.rs

#[derive(Clone)]
struct XdndInfo {
    current_window: u32,        //Window we are currently hovering
    exchange_started_with: u32, //Window we started an exchange with, 0 if not
    can_accept: bool,
    has_listening_thread: bool,
    got_status: bool, //We don't send more than one XdndPosition per XdndStatus.
    version: u32,
    data_to_send: String,
    is_dnd_drag: bool,
}
// Sync to be accessible in the listener thread
static XDND_INFO: OnceLock<Mutex<XdndInfo>> = OnceLock::new();
fn xdnd_info_lock() -> MutexGuard<'static, XdndInfo> {
    match XDND_INFO
        .get_or_init(|| {
            Mutex::new(XdndInfo {
                current_window: 0,
                exchange_started_with: 0,
                can_accept: false,
                has_listening_thread: false,
                got_status: true, //Start to true for the first XdndPosition
                version: 0,
                data_to_send: String::new(),
                is_dnd_drag: false,
            })
        })
        .lock()
    {
        Ok(e) => e,
        Err(_) => {
            panic!("Could not get CURRENT_WINDOW lock.");
        }
    }
}

fn reset_xdndinfo(update_has_listening_thread: bool) {
    let mut info = xdnd_info_lock();
    if update_has_listening_thread {
        info.has_listening_thread = false;
    }
    info.current_window = 0;
    info.exchange_started_with = 0;
    info.can_accept = false;
    info.got_status = true;
    info.version = 0;
    info.data_to_send = String::new();
    info.is_dnd_drag = false;
}

pub fn dnd_press(_: Rc<Weak<MainWindow>>) {
    let files = selected_files_lock();
    if files.len() != 1 {
        return;
    }

    let c = x_conn_lock();
    let qdfm_id = qdfm_win_id();

    //The current window is currently our own
    let mut info = xdnd_info_lock();
    info.current_window = qdfm_id;
    info.is_dnd_drag = true;

    info.data_to_send = String::from("file://") + &files.iter().next().unwrap().1.path;
    if c.set_selection_owner(qdfm_id, atom("XdndSelection"), CURRENT_TIME)
        .is_err()
    {
        log_debug("Could not set XdndSelection owner.");
    }
}

pub fn dnd_release(_: Rc<Weak<MainWindow>>) {
    if !xdnd_info_lock().is_dnd_drag {
        return;
    }
    let xdnd_info = xdnd_info_lock();
    let target_win = xdnd_info.exchange_started_with;
    let can_accept = xdnd_info.can_accept;
    drop(xdnd_info);

    // If a drop can be accepted
    if target_win != 0 && can_accept {
        send_xdnddrop(target_win);
    //If a drop has been refused
    } else if target_win != 0 && !can_accept {
        send_xdndleave(target_win);
    }

    if target_win == 0 || !can_accept {
        reset_xdndinfo(false);
    }
}

pub fn dnd_move(_: Rc<Weak<MainWindow>>, x: f32, y: f32) {
    if !xdnd_info_lock().is_dnd_drag {
        return;
    }
    let c = x_conn_lock();

    //Loop over every screen to look for the window we are hovering
    let mut window = 0;
    for screen in &c.setup().roots {
        if let Ok(w) = find_hovered_window(&c, screen.root) {
            if w != 0 {
                window = w;
                break;
            }
        }
    }

    //Current window doesn't support Xdnd
    if window == 0 {
        return;
    }

    let info = (*xdnd_info_lock()).clone();
    let qdfm_id = qdfm_win_id();

    //Window change
    if info.current_window != window {
        //Send XdndEnter and XdndLeave as appropriate
        if window != qdfm_id {
            enter_event(&c, window);
        }
        if info.current_window != qdfm_id {
            drop(c);
            send_xdndleave(info.current_window);
        }
        let mut info_lock = xdnd_info_lock();
        info_lock.got_status = true;
        info_lock.current_window = window;
    } else if info.exchange_started_with != 0 && window != qdfm_id {
        let (abs_x, abs_y) = get_absolute_position(&c, x as i16, y as i16);
        send_position(&c, window, abs_x, abs_y);
    }
}

//Clear the ClientMessage buffer
fn clear_clientmessage_buffer(receiver: &MutexGuard<'_, Receiver<(u64, Vec<i64>)>>) {
    while receiver.try_recv().is_ok() {}
}

fn send_xdnddrop(window: u32) {
    let c = x_conn_lock();
    let data: [u32; 5] = [qdfm_win_id(), 0, CURRENT_TIME, 0, atom("XdndActionMove")];

    let msg: ClientMessageEvent = ClientMessageEvent::new(32, window, atom("XdndDrop"), data);

    let res = c.send_event(false, window, EventMask::NO_EVENT, msg);

    if res.is_err() {
        log_debug("Failed to send XdndDrop");
        return;
    }
    c.flush().ok();
}

fn get_absolute_position(c: &MutexGuard<'_, RustConnection>, x: i16, y: i16) -> (u32, u32) {
    let roots = &c.setup().roots;
    if roots.len() >= 1 {
        if let Ok(cookie) = c.translate_coordinates(qdfm_win_id(), roots[0].root, x, y) {
            if let Ok(reply) = cookie.reply() {
                return (reply.dst_x as u32, reply.dst_y as u32);
            }
        }
    }
    (0, 0)
}

fn send_position(c: &MutexGuard<'_, RustConnection>, window: u32, x: u32, y: u32) {
    let mut xdnd_info = xdnd_info_lock();

    //We don't send more than one XdndPosition per XdndStatus received
    if !xdnd_info.got_status {
        return;
    }

    let data: [u32; 5] = [
        qdfm_win_id(),
        0,
        (x << 16) | y,
        CURRENT_TIME,
        atom("XdndActionMove"),
    ];

    let msg: ClientMessageEvent = ClientMessageEvent::new(32, window, atom("XdndPosition"), data);
    xdnd_info.got_status = false;
    let res = c.send_event(false, window, EventMask::NO_EVENT, msg);

    if res.is_err() {
        log_debug("Failed to send XdndPosition");
        return;
    }
    c.flush().ok();
}

fn send_selectionnotify(c: &MutexGuard<'_, RustConnection>, window: u32, ev: SelectionNotifyEvent) {
    let res = c.send_event(true, window, EventMask::NO_EVENT, ev);
    if res.is_err() {
        log_debug("Failed to send SelectionNotify");
        return;
    }
    c.flush().ok();
}

fn enter_event(c: &MutexGuard<'_, RustConnection>, window: u32) {
    let mut xdnd_info = xdnd_info_lock();
    let data: [u32; 5] = [
        qdfm_win_id(),
        (xdnd_info.version as u32).shl(24),
        atom("accepted_type"),
        0,
        0,
    ];

    let msg: ClientMessageEvent = ClientMessageEvent::new(32, window, atom("XdndEnter"), data);

    let res = c.send_event(false, window, EventMask::NO_EVENT, msg);

    if res.is_err() {
        log_debug("Failed to send XdndEnter");
        return;
    }

    xdnd_info.exchange_started_with = window;
    xdnd_info.can_accept = false;

    if !xdnd_info.has_listening_thread {
        xdnd_info.has_listening_thread = true;
        drop(xdnd_info);

        spawn(move || {
            let receiver = winit::platform::client_msg_recv();
            clear_clientmessage_buffer(&receiver);
            loop {
                let c = x_conn_lock();
                if let Ok(Some(ev)) = c.poll_for_event() {
                    match ev {
                        Event::SelectionRequest(e) => {
                            if e.target == atom("accepted_type") {
                                let info = xdnd_info_lock();
                                let data = info.data_to_send.as_bytes();
                                c.change_property(
                                    PropMode::REPLACE,
                                    e.requestor,
                                    e.property,
                                    e.target,
                                    8,
                                    data.len() as u32,
                                    data,
                                )
                                .ok();
                                send_selectionnotify(
                                    &c,
                                    e.requestor,
                                    SelectionNotifyEvent {
                                        time: e.time,
                                        property: e.property,
                                        requestor: e.requestor,
                                        selection: e.selection,
                                        sequence: e.sequence,
                                        response_type: SELECTION_NOTIFY_EVENT,
                                        target: e.target,
                                    },
                                );
                                c.flush().ok();
                            }
                        }
                        _ => {}
                    }
                }

                drop(c);

                //Exit point
                if xdnd_info_lock().exchange_started_with == 0 {
                    break;
                }
                //ClientMessage
                if let Ok((msg_type, msg)) = receiver.recv_timeout(Duration::from_millis(10)) {
                    if msg_type as u32 == atom("XdndStatus") {
                        let mut xdnd_info = xdnd_info_lock();
                        if msg[0] == xdnd_info.exchange_started_with as i64 {
                            xdnd_info.got_status = true;
                            let can_accept = msg[1] & 1;

                            if can_accept == 1 {
                                xdnd_info.can_accept = true;
                            } else {
                                xdnd_info.can_accept = false;
                            }
                        }
                    } else if msg_type as u32 == atom("XdndFinished") {
                        reset_xdndinfo(false);
                        break;
                    }
                }
            }
            xdnd_info_lock().has_listening_thread = false;
        });
    }
}

fn send_xdndleave(window: u32) {
    let c = x_conn_lock();
    let mut xdnd_info = xdnd_info_lock();
    if xdnd_info.exchange_started_with != window {
        return;
    }
    let data: [u32; 5] = [qdfm_win_id(), 0, 0, 0, 0];

    let msg: ClientMessageEvent = ClientMessageEvent::new(32, window, atom("XdndLeave"), data);

    let res = c.send_event(false, window, EventMask::NO_EVENT, msg);

    if res.is_err() {
        log_debug("Failed to send XdndLeave");
        return;
    }
    xdnd_info.exchange_started_with = 0;
    xdnd_info.can_accept = false;
}

///May be less accurate than XQueryTree, but probably faster?
fn find_hovered_window(
    c: &MutexGuard<'static, RustConnection>,
    w: u32,
) -> Result<u32, Box<dyn Error>> {
    let pointer_reply = c.query_pointer(w)?.reply()?;

    let version_reply = c
        .get_property(
            false,
            pointer_reply.child,
            atom("XdndAware"),
            AtomEnum::ANY,
            0,
            1024,
        )?
        .reply()?;

    if version_reply.value_len != 0 && version_reply.value[0] >= 3 {
        xdnd_info_lock().version = version_reply.value[0] as u32;
        return Ok(pointer_reply.child);
    }

    Ok(0)
}

pub fn xdnd_init(id: u32) {
    set_qdfm_win_id(id);

    if xdnd_atom_setup().is_err() {
        log_error_str("Error setuping X Atoms");
    }
}

/***
    Setup the atoms that will get reused
    Can be optimized to get all cookies then all replies instead TODO
    These are all safe to read/write to/from
*/

static ATOM_MAP: OnceLock<HashMap<&str, Atom>> = OnceLock::new();

///Returns the atom associated with s
fn atom(s: &str) -> Atom {
    let map = ATOM_MAP.get().expect("Atom map was never created");
    *map.get(s).expect("Atom map does not contain this atom ")
}

fn xdnd_atom_setup() -> Result<(), Box<dyn Error>> {
    let c = x_conn_lock();
    let atoms_arr = [
        "XdndSelection",
        "XdndEnter",
        "XdndAware",
        "XdndLeave",
        "XdndStatus",
        "XdndDrop",
        "XdndFinished",
        "XdndPosition",
        "XdndActionPrivate",
        "XdndActionPrivate",
        "XdndActionMove",
    ];

    let mut atoms_cookies = HashMap::new();
    for atom in atoms_arr {
        atoms_cookies.insert(atom, c.intern_atom(false, atom.as_bytes())?);
    }
    let mut atoms_reply = HashMap::new();
    for (atom_s, atom_v) in atoms_cookies {
        atoms_reply.insert(atom_s, atom_v.reply()?.atom);
    }
    atoms_reply.insert(
        "accepted_type",
        c.intern_atom(false, b"text/uri-list")?.reply()?.atom,
    );
    ATOM_MAP.set(atoms_reply).expect("Failed to set atom map");

    Ok(())
}
