use std::rc::Rc;

use slint::{ComponentHandle, Model, SharedString, VecModel, Weak};

use crate::{
    config::Mapping,
    core::run_command,
    globals::config_lock,
    ui::*,
    utils::{
        error_handling::{log_error_str, user_notice},
        file_picker::open_file_picker,
    },
};

pub fn setup_manage_open_with(adp: ManageOpenWithAdapter, files: Rc<Vec<FileItem>>) {
    let conf = config_lock();
    let ext = adp.get_extension();

    //If there is one file or all the files have the same extension
    if ext != "NOEXT" {
        let file = files[0].clone();
        let default = conf.get_mapping_default(&file.extension);
        let mappings = conf.get_mappings_quick(&file.extension);

        if default.is_some() {
            let default = default.unwrap().clone();
            adp.set_default_mapping(OpenWithMapping {
                cmd: "N/A".into(),
                name: default.into(),
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
}

pub fn ok(win: Rc<Weak<ManageOpenWithWindow>>, ext: SharedString) {
    let win = win.unwrap();
    let adp = win.global::<ManageOpenWithAdapter>();
    let mut conf = config_lock();
    conf.set_mappings_quick(
        &ext,
        adp.get_mappings()
            .iter()
            .map(|open_with_mapping| Mapping {
                display_name: open_with_mapping.name.to_string(),
                command: open_with_mapping.cmd.to_string(),
            })
            .collect::<Vec<Mapping>>(),
    );

    conf.set_default_for(&ext, &adp.get_default_mapping().name);

    win.hide().ok();
}

pub fn cancel(win: Rc<Weak<ManageOpenWithWindow>>) {
    win.unwrap().hide().ok();
}

pub fn open_with(win: Rc<Weak<ManageOpenWithWindow>>, with_term: bool, files: Rc<Vec<FileItem>>) {
    if let Ok(file_chosen) = open_file_picker() {
        for file in files.iter() {
            let cmd = if !with_term {
                file_chosen.to_owned() + " " + &file.file_name
            } else {
                if let Some(term) = config_lock().get::<String>("terminal") {
                    term + " " + &file_chosen + " " + &file.file_name
                } else {
                    log_error_str("No valid terminal. Fix your config.");
                    return;
                }
            };
            run_command(&cmd);
        }
        win.unwrap().hide().ok();
    } else {
        //TODO xdg not supported? Also parse error type
    }
}

pub fn set_default(ext: SharedString, s: SharedString) {
    let mut conf = config_lock();
    conf.set_default_for(&ext, &s);
}

pub fn add_mapping(win: Rc<Weak<ManageOpenWithWindow>>, mapping: OpenWithMapping) {
    let win = win.unwrap();
    if let Some(mappings) = win
        .global::<ManageOpenWithAdapter>()
        .get_mappings()
        .as_any()
        .downcast_ref::<VecModel<OpenWithMapping>>()
    {
        //Check to make sure it isn't already there.
        if mappings.iter().find(|m| m.name == mapping.name).is_none() {
            mappings.push(mapping);
        } else {
            user_notice("There is already an existing mapping with that name.");
        }
    }
}

pub fn remove_mapping(win: Rc<Weak<ManageOpenWithWindow>>, i: usize) {
    let win = win.unwrap();
    if let Some(mappings) = win
        .global::<ManageOpenWithAdapter>()
        .get_mappings()
        .as_any()
        .downcast_ref::<VecModel<OpenWithMapping>>()
    {
        mappings.remove(i);
    }
}
