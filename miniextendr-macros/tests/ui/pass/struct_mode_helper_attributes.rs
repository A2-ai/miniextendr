//! Attribute shorthand accepts the same helper attrs as its derives (#2024).
#![allow(dead_code)]

use std::cell::RefCell;

use miniextendr_api::{ColumnarFrame, DataFrameRow, IntoR, TryFromList, miniextendr};

#[miniextendr(list)]
#[derive(Clone)]
struct Cfg {
    label: String,
    #[into_list(ignore)]
    cache: RefCell<Option<Vec<u8>>>,
}

#[miniextendr(prefer = "list")]
struct Tuple(i32, #[into_list(ignore)] RefCell<Option<Vec<u8>>>);

#[miniextendr(dataframe)]
#[dataframe(name = "ObsFrame")]
#[derive(Clone, TryFromList)]
struct Obs {
    #[dataframe(rename = "observation_id")]
    id: i32,
    #[dataframe(skip)]
    #[into_list(ignore)]
    cache: RefCell<Option<Vec<u8>>>,
}

#[miniextendr(prefer = "dataframe")]
#[dataframe(name = "PreferredFrame")]
struct Preferred {
    #[dataframe(rename = "weight")]
    value: f64,
}

// A separate derive must still receive its own helpers; Obs's TryFromList
// above also needs the same into_list(ignore) helper as the shorthand.
#[miniextendr(list)]
#[derive(DataFrameRow)]
#[dataframe(name = "DualFrame")]
struct Dual {
    #[dataframe(rename = "weight")]
    value: f64,
}

fn list_contract<T: IntoR + TryFromList>() {}
fn list_reader<T: TryFromList>() {}

fn main() {
    list_contract::<Cfg>();
    list_contract::<Tuple>();
    list_contract::<Dual>();
    list_reader::<Obs>();
    let cfg = Cfg {
        label: String::from("label"),
        cache: RefCell::default(),
    };
    let _ = cfg.clone();
    let row = Obs {
        id: 1,
        cache: RefCell::default(),
    };
    let _ = row.clone();
    let _: ObsFrame = ObsFrame::from_rows(vec![row]);
    let _: PreferredFrame = PreferredFrame::from_rows(vec![Preferred { value: 1.0 }]);
    let _: DualFrame = DualFrame::from_rows(vec![Dual { value: 2.0 }]);
}
