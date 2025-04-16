use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::sleep,
    time::Duration,
};

use slint::{invoke_from_event_loop, ComponentHandle, LogicalPosition, SharedString, Weak};

use crate::{
    enclose,
    ui::{MainWindow, RenameAdapter, RenameWindow as RenameWindowUI},
    utils::error_handling::log_error_str,
};

pub struct RenameWindow {
    window: Arc<Mutex<Option<Weak<RenameWindowUI>>>>,
    receiver: Receiver<RenameWindowReturn>,
    _sender: Sender<RenameWindowReturn>, //Required to keep the receiver from dropping
    //Required to keep the window from dropping sooner. The weak reference isn't enough and strong
    //references can't be passed out of the main thread. So we leak it and drop it when this struct
    //goes out of scope.
    _raw_window_ptr: Arc<Mutex<Option<usize>>>,
}

#[derive(Clone)]
pub struct RenameWindowReturn {
    pub filename: Option<String>,
    pub option: RenameOption,
    pub apply_to_all: bool,
}

//Since we leak the window, we need to drop it when our struct goes out of scope
impl Drop for RenameWindow {
    fn drop(&mut self) {
        if let Some(ptr) = self._raw_window_ptr.lock().ok().and_then(|mtx| *mtx) {
            unsafe {
                drop(Box::from_raw(ptr as *mut RenameWindowUI));
            }
        }
    }
}
#[derive(PartialEq, Clone)]
pub enum RenameOption {
    Ignore,
    Rename,
    Overwrite,
}

pub fn setup_rename_window(mw: Weak<MainWindow>) -> RenameWindow {
    let win_mtx = Arc::new(Mutex::new(None));
    let ptr_mtx = Arc::new(Mutex::new(None));
    let (send, recv) = channel();
    let win = RenameWindow {
        window: win_mtx.clone(),
        receiver: recv,
        _sender: send.clone(),
        _raw_window_ptr: ptr_mtx.clone(),
    };

    invoke_from_event_loop(move || {
        if let Ok(win) = RenameWindowUI::new() {
            if let Ok(mut lock) = win_mtx.lock() {
                let adp = win.global::<RenameAdapter>();
                //Position the window slightly top-right from center
                //(Doing it here so it remembers the position in case the user moved it)
                let main_win = mw.unwrap();
                let pos = main_win.window().position();
                let x =
                    pos.x as f32 + (main_win.get_win_width() / 1.7) - (adp.get_win_width() / 1.7);
                let y = pos.y as f32 + (main_win.get_win_height() / 2.3)
                    - (adp.get_win_height() as f32 / 2.3);
                win.window().set_position(LogicalPosition { x, y });

                adp.on_ignore(enclose! { (send) move |b| on_ignore(send.clone(), b)});
                adp.on_rename(enclose! { (send) move |s, b| on_rename(send.clone(), s, b)});
                adp.on_overwrite(enclose! { (send) move |b| on_overwrite(send.clone(), b)});
                *lock = Some(win.as_weak());

                //Turn the window into a raw pointer, even cast it as usize, to bypass
                //rust's ownership feature. Otherwise there is no way to keep the window strong
                //reference in scope after returning from this event loop closure.
                if let Ok(mut lock) = ptr_mtx.lock() {
                    *lock = Some(Box::into_raw(Box::new(win)) as usize);
                }
            } else {
                log_error_str("Could not get the window lock");
            }
        } else {
            log_error_str("Could not create the rename window");
        }
    })
    .ok();
    win
}

impl RenameWindow {
    ///Shows a rename window for the selected filename
    ///Will block the calling thread until the user has chosen
    ///an option.
    ///Filename will be the new name only if rename was chosen
    #[cfg(not(test))]
    pub fn show_rename_window(&self, filename: String) -> Result<RenameWindowReturn, ()> {
        const MAX_LOOPS: usize = 10;
        for _ in 0..MAX_LOOPS {
            if let Ok(win) = self.window.lock() {
                if win.is_none() {
                    //Drop the lock and wait a bit to give the setup thread a chance to finish
                    drop(win);
                    sleep(Duration::from_millis(10));
                    continue;
                }
                let recv = &self.receiver;

                //Empty the receiver incase there are messages left from a previous operation
                while recv.try_recv().is_ok() {}
                let win = win.clone().unwrap();

                win.upgrade_in_event_loop(move |w| {
                    let adp = w.global::<RenameAdapter>();
                    adp.set_new_file_name(filename.clone().into());
                    adp.set_file_name(filename.into());
                    adp.set_apply_to_all(false);
                    w.show().ok();
                })
                .ok();

                //Wait and FREEZE THE CALLING THREAD until the user has chosen an option
                //Then return and resume the paste operation
                while let Ok(msg) = recv.recv() {
                    //TODO: CHECK THE NEW FILENAME IS VALID TOO
                    //Hide the window and return
                    win.upgrade_in_event_loop(|w| {
                        w.hide().ok();
                    })
                    .ok();
                    return Ok(msg);
                }
                return Err(());
            }
        }
        log_error_str("The rename window has not been initialized yet, so it could not be shown.");
        return Err(());
    }
    ///To spoof the user interaction during tests
    #[cfg(test)]
    pub fn show_rename_window(&self, _: String) -> Result<RenameWindowReturn, ()> {
        unsafe {
            if let Some(ret) = crate::tests::clipboard::paste::RENAME_WINDOW_RETURN.as_ref() {
                Ok(ret.clone())
            } else {
                Err(())
            }
        }
    }
}

fn on_overwrite(sender: Sender<RenameWindowReturn>, apply_to_all: bool) {
    sender
        .send(RenameWindowReturn {
            filename: None,
            option: RenameOption::Overwrite,
            apply_to_all,
        })
        .ok();
}
fn on_rename(sender: Sender<RenameWindowReturn>, newname: SharedString, apply_to_all: bool) {
    sender
        .send(RenameWindowReturn {
            filename: Some(newname.into()),
            option: RenameOption::Rename,
            apply_to_all,
        })
        .ok();
}
fn on_ignore(sender: Sender<RenameWindowReturn>, apply_to_all: bool) {
    sender
        .send(RenameWindowReturn {
            filename: None,
            option: RenameOption::Ignore,
            apply_to_all,
        })
        .ok();
}
