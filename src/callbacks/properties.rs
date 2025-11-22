use crate::{
    file_properties::{self},
    ui::*,
};
use prop_window::run_with_prop_window;
use slint::ComponentHandle;

/*
 *  Save and close
 * */
pub fn ok() {
    file_properties::save();
}
pub fn cancel() {
    run_with_prop_window(|w| w.hide().unwrap());
}

///Flips the bit corresponding to mask
pub fn recalculate_bitmask() {
    run_with_prop_window(|prop_win| {
        let prop_adp = prop_win.global::<PropertiesAdapter>();
        prop_adp.set_perm_bits_str(format!("{:o}", get_merged_bitmask(&prop_win)).into());
    });
}
///To update the UI, bits must be stored in their own variable.
///This merges them back to an i32 to actually be able to do something with it.
pub fn get_merged_bitmask(prop_win: &PropertiesWindow) -> i32 {
    let prop_adp = prop_win.global::<PropertiesAdapter>();

    let a_x = if prop_adp.get_a_x() { 1 } else { 0 };
    let a_w = if prop_adp.get_a_w() { 2 } else { 0 };
    let a_r = if prop_adp.get_a_r() { 4 } else { 0 };
    let g_x = if prop_adp.get_g_x() { 8 } else { 0 };
    let g_w = if prop_adp.get_g_w() { 16 } else { 0 };
    let g_r = if prop_adp.get_g_r() { 32 } else { 0 };
    let o_x = if prop_adp.get_o_x() { 64 } else { 0 };
    let o_w = if prop_adp.get_o_w() { 128 } else { 0 };
    let o_r = if prop_adp.get_o_r() { 256 } else { 0 };

    a_x | a_w | a_r | g_x | g_w | g_r | o_x | o_w | o_r
}
///To update the UI, bits must be stored in their own variable.
///This sets them from a single variable
pub fn set_split_bitmask(prop_adp: &PropertiesAdapter, i: i32) {
    prop_adp.set_a_x((i & 1) > 0);
    prop_adp.set_a_w((i & 2) > 0);
    prop_adp.set_a_r((i & 4) > 0);
    prop_adp.set_g_x((i & 8) > 0);
    prop_adp.set_g_w((i & 16) > 0);
    prop_adp.set_g_r((i & 32) > 0);
    prop_adp.set_o_x((i & 64) > 0);
    prop_adp.set_o_w((i & 128) > 0);
    prop_adp.set_o_r((i & 256) > 0);
}
