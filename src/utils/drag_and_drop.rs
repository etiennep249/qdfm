use super::error_handling::log_error_str;
use crate::callbacks::filemanager::set_current_tab_file;
use crate::clipboard::file_exists_in_dir;
use crate::globals::{qdfm_win_id, set_qdfm_win_id, x_conn_lock};
use crate::ui::*;
use slint::Weak;
use std::error::Error;
use std::ops::Shl;
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread::{sleep, spawn};
use std::time::Duration;
use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::xfixes::SelectionNotifyEvent;
use x11rb::protocol::xinput::PropertyEvent;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ClientMessageData, ClientMessageEvent, ConnectionExt, EventMask, PropMode,
};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;
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

struct XdndInfo {
    current_window: u32,        //Window we are currently hovering
    exchange_started_with: u32, //Window we started an exchange with, 0 if not
    received_status: bool,
    waiting_started: bool,
}

static XDND_INFO: OnceLock<Mutex<XdndInfo>> = OnceLock::new();
fn xdnd_info_lock() -> MutexGuard<'static, XdndInfo> {
    match XDND_INFO
        .get_or_init(|| {
            Mutex::new(XdndInfo {
                current_window: 0,
                exchange_started_with: 0,
                received_status: false,
                waiting_started: false,
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

pub fn dnd_press() {
    let c = x_conn_lock();
    let qdfm_id = qdfm_win_id();

    unsafe {
        if c.set_selection_owner(qdfm_id, XDNDSELECTION, CURRENT_TIME)
            .is_err()
        {
            log_error_str("Could not set XdndSelection owner.");
        }
    }
    //The current window is currently our own
    xdnd_info_lock().current_window = qdfm_id;
}

pub fn dnd_release() {
    let mut xdnd_info = xdnd_info_lock();

    if xdnd_info.exchange_started_with != 0 && xdnd_info.received_status {
        //xdnddrop
    }

    xdnd_info.current_window = 0;
    xdnd_info.exchange_started_with = 0;
    xdnd_info.received_status = false;
    xdnd_info.waiting_started = false;
}

pub fn dnd_move(x: f32, y: f32) {
    let c = x_conn_lock();

    //Loop over every screen to look for the window we are hovering
    let mut window = 0;
    for screen in &c.setup().roots {
        if let Ok(w) = find_hovered_window(&c, screen.root, x, y) {
            if w != 0 {
                window = w;
                break;
            }
        }
    }

    let mut xdnd_info = xdnd_info_lock();
    let old_window = xdnd_info.current_window;
    let qdfm_id = qdfm_win_id();

    if old_window != window {
        //Send XdndEnter and XdndLeave as appropriate
        drop(xdnd_info);
        if window != qdfm_id {
            enter_event(&c, window);
        }
        if old_window != qdfm_id {
            leave_event(&c, old_window);
        }
        xdnd_info_lock().current_window = window;
    } else if xdnd_info.exchange_started_with != 0 && !xdnd_info.received_status {
        //Send position first, then wait for a response
        let (abs_x, abs_y) = get_absolute_position(&c, x as i16, y as i16);
        send_position(&c, xdnd_info.current_window, abs_x, abs_y);
        c.flush().ok();
        drop(c);
        if !xdnd_info.waiting_started {
            xdnd_info.waiting_started = true;
            spawn(move || {
                loop {
                    if let Ok(Some(event)) = x_conn_lock().poll_for_event() {
                        println!("Got an event!: {:?}", event);
                        match event {
                            Event::SelectionNotify(e) => {
                                println!("SelectionNotify {:?}", e);
                            }
                            _ => {}
                        }
                    } else {
                        println!("No events :( ");
                    } /*else {
                          log_error_str("Connection dropped while waiting for event. Aborting.");
                          break;
                      }*/
                    sleep(Duration::from_millis(200));
                }
            });
        }
    }
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
    let data: [u32; 5] = [qdfm_win_id(), 0, (x << 16) | y, CURRENT_TIME, unsafe {
        XDNDACTIONCOPY
    }];

    let msg: ClientMessageEvent =
        ClientMessageEvent::new(32, window, unsafe { XDNDPOSITION }, data);

    let res = c.send_event(false, window, EventMask::NO_EVENT, msg);

    if res.is_err() {
        log_error_str("Failed to send XdndPosition");
        return;
    }
}

fn enter_event(c: &MutexGuard<'_, RustConnection>, window: u32) {
    //Check for XdndSupport
    let version = if let Ok(cookie) =
        c.get_property(false, window, unsafe { XDNDAWARE }, AtomEnum::ANY, 0, 1024)
    {
        if let Ok(reply) = cookie.reply() {
            if reply.value_len == 0 {
                0
            } else {
                reply.value[0]
            }
        } else {
            0
        }
    } else {
        0
    };

    if version == 0 {
        //Xdnd not supported, move on
        return;
    }

    let data: [u32; 5] = [
        qdfm_win_id(),
        (version as u32).shl(24),
        unsafe { ACCEPTED_TYPE },
        0,
        0,
    ];

    let msg: ClientMessageEvent = ClientMessageEvent::new(32, window, unsafe { XDNDENTER }, data);

    let res = c.send_event(false, window, EventMask::NO_EVENT, msg);

    if res.is_err() {
        log_error_str("Failed to send XdndEnter");
        return;
    }
    xdnd_info_lock().exchange_started_with = window;
}

pub fn leave_event(c: &MutexGuard<'_, RustConnection>, window: u32) {
    let w = xdnd_info_lock().exchange_started_with;
    if w != window {
        return;
    }
    let data: [u32; 5] = [qdfm_win_id(), 0, 0, 0, 0];

    let msg: ClientMessageEvent = ClientMessageEvent::new(32, window, unsafe { XDNDLEAVE }, data);

    let res = c.send_event(false, window, EventMask::NO_EVENT, msg);

    if res.is_err() {
        log_error_str("Failed to send XdndLeave");
        return;
    }

    xdnd_info_lock().exchange_started_with = 0;
}

/**
   Recursively goes over every window and returns the one we are hovering
   As far as I know, there is no better way to do it as every mouse event
   returns the starting window and not the currently hovering one.
* */
fn find_hovered_window(
    c: &MutexGuard<'static, RustConnection>,
    w: u32,
    x: f32,
    y: f32,
) -> Result<u32, Box<dyn Error>> {
    let mut ret = 0;

    let mut children = c.query_tree(w)?.reply()?.children;
    children.reverse();
    for child in children {
        let geo = c.get_geometry(child)?.reply()?;
        let geo_x = geo.x as f32;
        let geo_y = geo.y as f32;
        let geo_w = geo.width as f32;
        let geo_h = geo.height as f32;

        //If cursor is within the window's confines
        if geo_x < x && x < (geo_x + geo_w) && geo_y < y && y < (geo_y + geo_h) {
            ret = find_hovered_window(c, child, x, y).unwrap_or(0);
            break;
        }
    }

    if ret == 0 {
        ret = w;
    }

    return Ok(ret);
}

pub fn xdnd_init(id: u32) {
    //Connection

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
static mut XDNDSELECTION: Atom = 0;
static mut XDNDENTER: Atom = 0;
static mut XDNDAWARE: Atom = 0;
static mut XDNDLEAVE: Atom = 0;
static mut XDNDPOSITION: Atom = 0;
static mut XDNDACTIONCOPY: Atom = 0;
static mut ACCEPTED_TYPE: Atom = 0;
fn xdnd_atom_setup() -> Result<(), Box<dyn Error>> {
    unsafe {
        let c = x_conn_lock();
        XDNDSELECTION = c
            .intern_atom(false, "XdndSelection".as_bytes())?
            .reply()?
            .atom;

        XDNDENTER = c.intern_atom(false, "XdndEnter".as_bytes())?.reply()?.atom;
        XDNDAWARE = c.intern_atom(false, "XdndAware".as_bytes())?.reply()?.atom;
        XDNDLEAVE = c.intern_atom(false, "XdndLeave".as_bytes())?.reply()?.atom;
        XDNDPOSITION = c
            .intern_atom(false, "XdndPosition".as_bytes())?
            .reply()?
            .atom;
        ACCEPTED_TYPE = c.intern_atom(false, "image/png".as_bytes())?.reply()?.atom;
        XDNDACTIONCOPY = c
            .intern_atom(false, "XdndActionCopy".as_bytes())?
            .reply()?
            .atom;
    }
    Ok(())
}
