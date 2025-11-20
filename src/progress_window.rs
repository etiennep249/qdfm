use std::{
    rc::Rc,
    sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender, TryRecvError},
    time::{Duration, Instant},
};

use slint::{ComponentHandle, LogicalPosition, SharedString, Weak};

use crate::{
    ui::{self, *},
    utils::error_handling::log_error_str,
};

///The receiver passed is expected to receive updates periodically.
///Whenever it receives an update, it will update the UI accordingly.
///The value passed should be an f32 between 0 and 1 representing the progress,
///a string representing the status, a i64 representing the seconds remaining as well as
///a boolean. If true, the delay will be ignored. Otherwise the UI will only update every interval
///A negative value means an unknown amount of time
///
///This is the only function that needs to be called to have a working progress window.
///The UI of the mainwindow is always refreshed when the sender is dropped.
///
///The caller is responsible to check if send() returns a disconnect error. If so, it means
///that the user canceled the operation through the progress window. The caller should react
///accordingly
pub fn show_progress_window(recv: Receiver<(f32, String, f64, bool)>, interval: Duration) {
    //Unwrap everything. If it panics, no big deal, we just get no progress bar.
    //This will often be called in other threads, so need this invoke_from_event_loop
    ui::run_with_main_window(move |main_win| {
        let win = ProgressWindow::new().unwrap();

        let pos = main_win.window().position();
        let x = pos.x as f32 + (main_win.get_win_width() / 2.0) - (win.get_win_width() / 2.0);
        let y = pos.y as f32 + (main_win.get_win_height() / 2.0) - (win.get_win_height() / 2.0);

        win.window().set_position(LogicalPosition { x, y });

        let weak = Rc::new(win.as_weak());
        let adp = win.global::<ProgressAdapter>();

        //Channel for cancel/pause to notify the thread
        let (progress_tx, progress_rx) = channel::<bool>();
        let weak2 = weak.clone();
        adp.on_close(move || cancel(weak.clone(), progress_tx.clone()));
        adp.on_pause(move || pause(weak2.clone()));

        let weak = win.as_weak();

        //Thread to update UI
        std::thread::spawn(move || {
            let mut last_modifed = Instant::now();
            let mut last_msg: Option<(f32, String, f64, bool)> = None;

            loop {
                let msg_received = recv.recv_timeout(interval);
                if let Ok(msg) = msg_received {
                    last_msg = Some(msg);
                } else if let Err(e) = msg_received {
                    if e == RecvTimeoutError::Timeout {
                    } else {
                        break;
                    }
                }
                if last_msg.is_some() {
                    let msg = last_msg.clone().unwrap();
                    if msg.3 || last_modifed.elapsed() > interval {
                        weak.upgrade_in_event_loop(move |w| {
                            let adp = w.global::<ProgressAdapter>();
                            adp.set_progress(msg.0);
                            adp.set_progress_text(msg.1.into());
                            if msg.2 > 0.0 {
                                adp.set_remaining_text(format_seconds(msg.2));
                            }
                        })
                        .unwrap();
                        last_modifed = Instant::now();
                        last_msg = None;
                    }
                }
                //If main thread signals that we should stop, or we are somehow disconnected
                let thread_should_die = progress_rx.try_recv();
                if thread_should_die.is_ok()
                    || thread_should_die.is_err_and(|err| err == TryRecvError::Disconnected)
                {
                    break;
                }
            }

            //Sender probably dropped, get out and update UI
            weak.upgrade_in_event_loop(|w| {
                w.hide().ok();
                ui::send_message(UIMessage::Refresh);
            })
            .unwrap();
        });
        win.show().ok();
    });
}

fn format_seconds(seconds: f64) -> SharedString {
    let whole_seconds = seconds.trunc() as u32;
    let fractional_seconds = seconds.fract();
    let hours = whole_seconds / 3600;
    let minutes = (whole_seconds % 3600) / 60;
    let remaining_seconds = (whole_seconds % 60) as f64 + fractional_seconds;
    (String::from("ETA ")
        + &match (hours, minutes) {
            (0, 0) => format!("{:.1}s", remaining_seconds),
            (0, _) => format!("{minutes}m {:.1}s", remaining_seconds),
            _ => format!("{hours}h {minutes}m {:.1}s", remaining_seconds),
        })
        .into()
}

pub fn cancel(win: Rc<Weak<ProgressWindow>>, sender: Sender<bool>) {
    sender.send(true).ok();
    win.unwrap().hide().ok();
}
pub fn pause(_: Rc<Weak<ProgressWindow>>) {
    //TODO:
    log_error_str("Currently unsupported.");
}
