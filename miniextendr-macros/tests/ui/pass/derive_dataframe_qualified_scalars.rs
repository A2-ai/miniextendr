//! Qualified scalar names keep struct and enum readers (#2036).
#![allow(dead_code)]

extern crate alloc;

use miniextendr_api::{DataFrameRow, SEXP};

#[derive(DataFrameRow)]
struct Qualified {
    id: ::core::primitive::i32,
    label: ::std::string::String,
    note: ::core::option::Option<::alloc::string::String>,
    weight: Option<std::primitive::f64>,
    relative: Option<std::string::String>,
    pair: [::core::primitive::f64; 2],
    #[dataframe(width = 2)]
    pinned: Vec<::std::string::String>,
    #[dataframe(expand)]
    rest: Vec<::std::primitive::i32>,
    keyed: std::collections::BTreeMap<::core::primitive::i32, ::std::string::String>,
    named: std::collections::HashMap<::std::string::String, ::core::primitive::f64>,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
enum QualifiedEvent {
    Value {
        label: ::alloc::string::String,
        pair: [::core::primitive::f64; 2],
        #[dataframe(width = 2)]
        pinned: Vec<::std::string::String>,
        #[dataframe(expand)]
        rest: Vec<::std::primitive::i32>,
        keyed: std::collections::BTreeMap<::core::primitive::i32, ::std::string::String>,
    },
    Empty,
}

// Calling the inherent reader is deliberate: DataFrameRowConvert's default
// exists even when the derive silently omits an unsupported shape's reader.
fn read(df: SEXP) {
    let _: Result<Vec<Qualified>, String> = Qualified::try_from_dataframe(df);
    let _: Result<Vec<QualifiedEvent>, String> = QualifiedEvent::try_from_dataframe(df);
    miniextendr_api::__dataframe_row_when_rayon! {
        let _: Result<Vec<Qualified>, String> = Qualified::try_from_dataframe_par(df);
        let _: Result<Vec<QualifiedEvent>, String> = QualifiedEvent::try_from_dataframe_par(df);
    }
}

fn main() {}
