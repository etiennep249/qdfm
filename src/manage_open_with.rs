use std::rc::Rc;

use slint::{ComponentHandle, SharedString, VecModel, Weak};

use crate::{
    core::run_command,
    globals::config_lock,
    ui::*,
    utils::{error_handling::log_error_str, file_picker::open_file_picker},
};

pub fn setup_manage_open_with(adp: ManageOpenWithAdapter, file: FileItem) {
    let conf = config_lock();
    let default = conf.get_mapping_default(&file.extension);
    let mappings = conf.get_mappings_quick(&file.extension);

    if default.is_some() {
        let default = default.unwrap().clone();
        adp.set_default_mapping(OpenWithMapping {
            cmd: default.command.into(),
            name: default.display_name.into(),
        });
    }
    adp.set_mappings(
        Rc::new(VecModel::from(
            mappings
                .iter()
                .map(|m| OpenWithMapping {
                    cmd: SharedString::from(&m.command),
                    name: SharedString::from(&m.display_name),
                })
                .collect::<Vec<OpenWithMapping>>(),
        ))
        .into(),
    );
}

pub fn ok(win: Rc<Weak<ManageOpenWithWindow>>) {
    win.unwrap().hide().ok();
}

pub fn cancel(win: Rc<Weak<ManageOpenWithWindow>>) {
    win.unwrap().hide().ok();
}

pub fn open_with(win: Rc<Weak<ManageOpenWithWindow>>, with_term: bool, filename: SharedString) {
    if let Ok(file_chosen) = open_file_picker() {
        let cmd = if !with_term {
            file_chosen + " " + &filename
        } else {
            if let Some(term) = config_lock().get::<String>("terminal") {
                term + " " + &file_chosen + " " + &filename
            } else {
                log_error_str("No valid terminal. Fix your config.");
                return;
            }
        };
        run_command(&cmd);
        win.unwrap().hide().ok();
    } else {
        //TODO
    }
}
