use std::rc::Rc;

use slint::{Model, ModelRc, Weak};

use crate::{sort, ui::*};

pub fn on_header_click(header: Header, mw: Rc<Weak<MainWindow>>) {
    let new_sort = if header.sort == 0 {
        1
    } else if header.sort == 1 {
        2
    } else if header.sort == 2 {
        1
    } else {
        return;
    };

    //Sort
    match header.inner_value {
        0 => sort::sort_by_name(mw.clone(), new_sort == 1),
        _ => {}
    };

    //Change sort values
    let mw_upgraded = mw.unwrap();
    let headers_rc = mw_upgraded.global::<ColumnHeadersAdapter>().get_headers();
    for i in 0..headers_rc.row_count() {
        //Change that header's sort
        if headers_rc.row_data(i).unwrap().inner_value == header.inner_value {
            let mut new_header = headers_rc.row_data(i).unwrap();
            new_header.sort = new_sort;
            headers_rc.set_row_data(i, new_header);
        //Remove sorting for all other headers
        } else if headers_rc.row_data(i).unwrap().sort != 0 {
            let mut new_header = headers_rc.row_data(i).unwrap();
            new_header.sort = 0;
            headers_rc.set_row_data(i, new_header);
        }
    }
}
const MINIMUM_HEADER_PCT: f32 = 4.0;
pub fn on_header_resize(
    header: Header,
    size_offset: f32,
    original_size: f32,
    mw: Rc<Weak<MainWindow>>,
) {
    /*[old pct    - old size]*/
    /*[new pct(?) - new size]*/
    let new_pct = header.width_pct * (original_size + size_offset) / original_size;
    let diff_pct = new_pct - header.width_pct;

    //Do not go below the minimum width if we are downsizing the current header
    if size_offset < 0.0 && new_pct < MINIMUM_HEADER_PCT {
        return;
    }

    let mw_upgraded = mw.unwrap();
    let headers_rc = mw_upgraded.global::<ColumnHeadersAdapter>().get_headers();

    //Get the header index - avoids nesing the rest of all this
    let mut i = 0;
    for j in 0..headers_rc.row_count() {
        if headers_rc.row_data(j).unwrap().inner_value == header.inner_value {
            i = j;
            break;
        }
    }

    //Set the new pct
    //Increment the size of the right header
    if size_offset < 0.0 {
        if headers_rc.row_data(i + 1).unwrap().width_pct > (-size_offset) {
            incr_header_pct(headers_rc.clone(), i + 1, diff_pct);
        } else {
            return;
        }
    }
    //Decrement the size of the right header
    else {
        if let Some(h_idx_to_decr) =
            get_next_non_min_header_idx(headers_rc.clone(), i + 1, size_offset)
        {
            if headers_rc.row_data(h_idx_to_decr).unwrap().width_pct > size_offset {
                incr_header_pct(headers_rc.clone(), h_idx_to_decr, diff_pct);
            } else {
                return;
            }
        } else {
            return;
        }
    }
    //Now that the other headers are fine, resize the current one
    incr_header_pct(headers_rc, i, -diff_pct);

    /*Debug, print all widths*/
    /*let mut vec: Vec<f32> = Vec::new();
    for i in 0..headers_rc.row_count() {
        vec.push(headers_rc.row_data(i).unwrap().width_pct);
    }
    println!("New Widths(sum:{}): {:?}", vec.iter().sum::<f32>(), vec);*/
}

pub fn set_header_width(headers_rc: ModelRc<Header>, i: usize, new_size: f32) {
    let mut new_header = headers_rc.row_data(i).unwrap();
    new_header.width_pct = new_size;
    headers_rc.set_row_data(i, new_header);
}

pub fn incr_header_pct(
    headers_rc: ModelRc<Header>, //Header_rc
    i: usize,                    //Header index in headers_rc
    pct_to_add: f32,
) {
    let right_header = headers_rc.row_data(i).unwrap();
    set_header_width(headers_rc, i, right_header.width_pct - pct_to_add);
}
//start_idx included
//Used to find which header to reduce in size
pub fn get_next_non_min_header_idx(
    headers_rc: ModelRc<Header>,
    start_idx: usize,
    offset: f32,
) -> Option<usize> {
    if start_idx >= headers_rc.row_count() {
        return None;
    }
    for i in start_idx..headers_rc.row_count() {
        if headers_rc.row_data(i).unwrap().width_pct - offset > MINIMUM_HEADER_PCT {
            return Some(i);
        }
    }
    None
}
// TODO: Save header width's to a file (add to config struct)
