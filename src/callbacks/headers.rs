use std::rc::Rc;

use slint::{Model, Weak};

use crate::{sort::sort_by_name, ui::*};

pub fn on_header_click(header: Header, mw: Rc<Weak<MainWindow>>) {
    let new_sort = if header.sort == 0 {
        1
    } else if header.sort == 1 {
        2
    } else if header.sort == 2 {
        1
    } else {
        -1 /*Should not happen*/
    };

    //Sort
    match header.inner_value {
        0 => sort_by_name(mw.clone(), new_sort == 1),
        _ => {}
    };

    //Change sort value for header
    let mw_upgraded = mw.unwrap();
    let headers_rc = mw_upgraded.global::<ColumnHeadersAdapter>().get_headers();
    for i in 0..headers_rc.row_count() {
        if headers_rc.row_data(i).unwrap().inner_value == header.inner_value {
            let mut new_header = headers_rc.row_data(i).unwrap().clone();
            new_header.sort = new_sort;
            headers_rc.set_row_data(i, new_header);
        } else if headers_rc.row_data(i).unwrap().sort != 0 {
            let mut new_header = headers_rc.row_data(i).unwrap().clone();
            new_header.sort = 0;
            headers_rc.set_row_data(i, new_header);
        }
    }
}
