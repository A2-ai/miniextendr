//! Compile-pass test: the list, `ExternalPtr` and preference derives ask of a
//! field type only the conversion that field needs. `Opaque` implements
//! nothing; `DefaultOnly` implements only the `Default` an ignored
//! `TryFromList` field is filled with.

#![allow(dead_code)]

use std::rc::Rc;

use miniextendr_api::externalptr::RSidecar;
use miniextendr_api::{ExternalPtr, IntoList, IntoR, PreferList, TryFromList, TryFromSexp};

/// Implements nothing.
struct Opaque(Rc<i32>);

/// Converts both ways and does nothing else.
#[derive(IntoR, TryFromSexp)]
struct Col(f64);

#[derive(Default)]
struct DefaultOnly(Rc<i32>);

#[derive(IntoList, PreferList)]
struct Out {
    a: i32,
    c: Col,
    #[into_list(ignore)]
    skipped: Opaque,
}

#[derive(TryFromList)]
struct In {
    a: i32,
    c: Col,
    #[into_list(ignore)]
    skipped: DefaultOnly,
}

/// Neither `Send`, `Clone` nor `Debug`.
#[derive(ExternalPtr)]
#[externalptr(env)]
struct Handle {
    opaque: Opaque,
    #[r_data]
    _r: RSidecar,
    #[r_data]
    count: i32,
    #[r_data]
    name: String,
}

fn main() {}
