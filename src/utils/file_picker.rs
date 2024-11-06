use dbus::{arg::PropMap, Message};
use std::{
    collections::HashMap,
    sync::mpsc::{channel, TryRecvError},
    time::Duration,
};

use dbus::{
    arg::{self, RefArg, Variant},
    blocking::Connection,
    message::MatchRule,
    Path,
};

use super::rand;

pub struct DbusReader {
    pub data: String,
}

impl arg::ReadAll for DbusReader {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        println!("test");
        Ok(DbusReader { data: i.read()? })
    }
}

/*
 *
 *  Using XDG_DESKTOP_PORTAL on the D-Bus, we let the user pick a file
 *
 **/

pub enum FilePickerError {
    Disconnected,
    OperationCanceled,
    BadResponse,
    DbusMatchFail,
    ProcessError,
    MethodCallFail,
    ConnectionFail,
}

pub fn open_file_picker() -> Result<String, FilePickerError> {
    let conn = Connection::new_session();

    if conn.is_err() {
        return Err(FilePickerError::ConnectionFail);
    }
    let conn = conn.unwrap();

    let proxy = conn.with_proxy(
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop",
        Duration::from_millis(5000),
    );

    let mut options = HashMap::<String, Variant<Box<dyn RefArg>>>::new();

    // Accept Button Label
    options.insert(
        "accept_label".into(),
        Variant(Box::new("Open With".to_string()) as Box<dyn RefArg>),
    );

    // Handle Token
    options.insert(
        "handle_token".into(),
        Variant(Box::new("qdfm_".to_string() + &rand().to_string()) as Box<dyn RefArg>),
    );

    // Filters
    options.insert(
        "filters".into(),
        Variant(Box::new(vec![
            (
                "Executables".to_string(),
                vec![
                    (1, "application/x-elf".to_string()),
                    (1, "application/x-sh".to_string()),
                    (1, "application/x-perl".to_string()),
                    (1, "application/x-python".to_string()),
                    (1, "application/x-pie-executable".to_string()),
                    (1, "application/x-executable".to_string()),
                ],
            ),
            ("Any".to_string(), vec![(0 as u32, "*".to_string())]),
        ]) as Box<dyn RefArg>),
    );

    options.insert(
        "current_folder".to_string(),
        Variant(Box::new(std::env::var("HOME").unwrap_or("/".to_string()))),
    );

    // Call
    let res = proxy.method_call(
        "org.freedesktop.portal.FileChooser",
        "OpenFile",
        ("", "Title", options),
    );
    if res.is_err() {
        return Err(FilePickerError::MethodCallFail);
    }
    let (path,): (Path<'static>,) = res.unwrap();

    // Match Rule
    let matches = MatchRule::new()
        .with_type(dbus::MessageType::Signal)
        .with_path(path)
        .with_interface("org.freedesktop.portal.Request")
        .with_member("Response");

    // Channel

    let (send, recv) = channel::<Result<String, FilePickerError>>();

    // On Response Callback, parse the file path chosen and send it through the channel
    // Verbose error checking included
    let match_res = conn.add_match(matches, move |_: (), _, msg: &Message| {
        let mut iter = msg.iter_init();
        iter.next();
        if let Some(map) = &iter.get::<PropMap>() {
            let uris = map.get("uris");
            if uris.is_none() {
                send.send(Err(FilePickerError::BadResponse)).ok();
                return true;
            }
            let files_chosen = uris.unwrap().0.as_any().downcast_ref::<Vec<String>>();
            if files_chosen.is_none() {
                send.send(Err(FilePickerError::BadResponse)).ok();
                return true;
            }
            let files_chosen = files_chosen.unwrap();
            if files_chosen.len() == 0 || files_chosen[0].is_empty() {
                send.send(Err(FilePickerError::OperationCanceled)).ok()
            } else {
                match files_chosen[0].strip_prefix("file://") {
                    Some(s) => send.send(Ok(s.into())).ok(),
                    None => send.send(Err(FilePickerError::BadResponse)).ok(),
                }
            };
        } else {
            send.send(Err(FilePickerError::BadResponse)).ok();
        }
        true
    });
    if match_res.is_err() {
        return Err(FilePickerError::DbusMatchFail);
    }

    loop {
        if conn.process(Duration::from_millis(1000)).is_err() {
            return Err(FilePickerError::ProcessError);
        }
        match recv.try_recv() {
            Ok(s) => return s,
            Err(e) => {
                if e == TryRecvError::Empty {
                    continue;
                } else {
                    return Err(FilePickerError::Disconnected);
                }
            }
        }
    }
}
