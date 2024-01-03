use chrono::{DateTime, Utc};
use slint::SharedString;

pub fn i32_to_i64((a, b): (i32, i32)) -> i64 {
    ((a as i64) << 32) | (b as i64 & 0xFFFFFFFF)
}
pub fn i64_to_i32(i: i64) -> (i32, i32) {
    ((i >> 32) as i32, i as i32)
}

//Utility to convert bytes into a human readable format
const KIB: f64 = 1024.0;
const MIB: f64 = 1024.0 * KIB;
const GIB: f64 = 1024.0 * MIB;
const TIB: f64 = 1024.0 * GIB;
pub fn format_size(i: i64) -> SharedString {
    let f = i as f64;
    let suffix;
    (format!(
        "{:.2}", //0.00
        if f < KIB {
            suffix = " B";
            f
        } else if f < MIB {
            suffix = " KiB";
            f / KIB
        } else if f < GIB {
            suffix = " MiB";
            f / MIB
        } else if f < TIB {
            suffix = " GiB";
            f / GIB
        } else {
            suffix = " TiB";
            f / TIB
        }
    ) + suffix)
        .into()
}

//Utility to convert seconds to a human readable format
//TODO: If this is the only place where we use chrono, possibly write it manually
//TODO: Possibly support more formats (for now, ISO 8601 is fine)
//Where i = number of seconds since UNIX_EPOCH
pub fn format_date(i: i64) -> SharedString {
    if let Some(datetime) = DateTime::<Utc>::from_timestamp(i, 0) {
        format!("{}", datetime.format("%F %I:%M %p")).into()
    } else {
        "ERR".into()
    }
}
