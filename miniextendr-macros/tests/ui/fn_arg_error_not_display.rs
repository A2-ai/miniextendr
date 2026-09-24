//! Test: an argument type whose `TryFromSexp::Error` is neither
//! `RConditionError` nor `Display` is rejected.
//!
//! A failed conversion becomes an R condition built from the error: its
//! `RConditionError` class, message and data when it has the impl, its
//! `Display` text otherwise. With neither, the generated `Err` arm's probe
//! (`__mx_conversion_err_parts!`) finds no method and names both bounds.

use miniextendr_api::{SEXP, TryFromSexp};
use miniextendr_macros::miniextendr;

pub struct Opaque;

pub struct OpaqueError;

impl TryFromSexp for Opaque {
    type Error = OpaqueError;

    fn try_from_sexp(_sexp: SEXP) -> Result<Self, Self::Error> {
        Err(OpaqueError)
    }
}

#[miniextendr]
fn takes_opaque(_x: Opaque) -> i32 {
    0
}

fn main() {}
