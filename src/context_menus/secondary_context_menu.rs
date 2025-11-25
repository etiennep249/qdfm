use std::rc::Rc;

use crate::{
    callbacks::context_menu::ContextCallback,
    ui::{main_window::run_with_main_window, ContextAdapter, ContextItem, Theme},
};
use slint::{ComponentHandle, VecModel};

///Shows a secondary menu with the given items, relative to the position of the parent,
///with the parent being the item that triggers the secondary menu on hover.
pub fn show_secondary_context_menu(menu: Vec<ContextItem>, parent_index: i32) {
    run_with_main_window(move |mw| {
        let ctx_adapter = mw.global::<ContextAdapter>();
        let theme = mw.global::<Theme>().get_current();
        ctx_adapter.set_secondary_items(Rc::new(VecModel::from(menu)).into());
        ctx_adapter.set_secondary_x_pos(ctx_adapter.get_x_pos() + theme.context_menu_width);
        ctx_adapter.set_secondary_y_pos(
            ctx_adapter.get_y_pos() + (parent_index as f32 * theme.context_menu_entry_height) + 1.0,
        );
        ctx_adapter.set_current_hover_callback_id(ContextCallback::OpenWith as i32);
        ctx_adapter.set_is_secondary_visible(true);
    });
}
