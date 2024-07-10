use slint::SharedString;

use crate::ui::*;
use crate::utils::types::{format_size, i32_to_i64};

pub fn format_size_detailed(i: _i64) -> SharedString {
    format_size(i32_to_i64((i.a, i.b)), true)
}
