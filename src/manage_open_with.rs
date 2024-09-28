use std::rc::Rc;

use slint::{ComponentHandle, SharedString, VecModel, Weak};

use crate::{globals::config_lock, ui::*};

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

pub fn ok(win: Rc<Weak<ManageOpenWithWindow>>) {}

pub fn cancel(win: Rc<Weak<ManageOpenWithWindow>>) {
    win.unwrap().hide().ok();
}
