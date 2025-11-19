use slint::{Image, SharedPixelBuffer};

use crate::{callbacks::context_menu::ContextCallback, ui::ContextItem};
use std::{cell::UnsafeCell, collections::HashMap, sync::Once};

struct StaticContextItems {
    inner: UnsafeCell<Option<HashMap<&'static str, ContextItem>>>,
}

unsafe impl Sync for StaticContextItems {}

static INIT: Once = Once::new();
static CONTEXT_ITEMS: StaticContextItems = StaticContextItems {
    inner: UnsafeCell::new(None),
};

pub fn get_ci(k: &str) -> ContextItem {
    INIT.call_once(|| {
        init_context_items();
    });
    let map = unsafe { (*CONTEXT_ITEMS.inner.get()).as_ref().unwrap() };
    map[k].clone()
}

pub fn get_ci_capacity() -> usize {
    INIT.call_once(|| {
        init_context_items();
    });
    unsafe { (*CONTEXT_ITEMS.inner.get()).as_ref().unwrap().len() }
}

fn init_context_items() {
    let mut map = HashMap::new();
    map.insert(
        "open_with_default",
        ContextItem {
            display: "".into(),
            callback_id: ContextCallback::OpenWithDefault as i32,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: true,
            click_on_hover: false,
            internal_id: 0,
        },
    );
    map.insert(
        "open_with",
        ContextItem {
            display: ("Open With").into(),
            callback_id: ContextCallback::OpenWith as i32,
            shortcut: "▶".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: true,
            click_on_hover: true,
            internal_id: 0,
        },
    );
    map.insert(
        "cut",
        ContextItem {
            display: "Cut".into(),
            callback_id: ContextCallback::Cut as i32,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: false,
            click_on_hover: false,
            internal_id: 0,
        },
    );
    map.insert(
        "copy",
        ContextItem {
            display: "Copy".into(),
            callback_id: ContextCallback::Copy as i32,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: false,
            click_on_hover: false,
            internal_id: 0,
        },
    );

    map.insert(
        "paste_into",
        ContextItem {
            display: "Paste Into".into(),
            callback_id: ContextCallback::PasteIntoSelected as i32,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: true,
            click_on_hover: false,
            internal_id: 0,
        },
    );
    map.insert(
        "paste_here",
        ContextItem {
            display: "Paste Here".into(),
            callback_id: ContextCallback::PasteHere as i32,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: true,
            click_on_hover: false,
            internal_id: 0,
        },
    );

    map.insert(
        "delete",
        ContextItem {
            display: "Delete".into(),
            callback_id: ContextCallback::Delete as i32,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: true,
            click_on_hover: false,
            internal_id: 0,
        },
    );
    map.insert(
        "properties",
        ContextItem {
            display: "Properties".into(),
            callback_id: ContextCallback::ShowProperties as i32,
            shortcut: "".into(),
            icon: Image::from_rgb8(SharedPixelBuffer::new(0, 0)),
            has_separator: false,
            click_on_hover: false,
            internal_id: 0,
        },
    );

    unsafe { *CONTEXT_ITEMS.inner.get() = Some(map) }
}
