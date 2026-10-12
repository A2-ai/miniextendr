//! Test: the R getter of a converted `#[r_data]` field reads the field through
//! a shared borrow and converts a clone of it, so the field's type must be
//! `Clone`; the error points at the field's type.

use miniextendr_api::externalptr::RSidecar;
use miniextendr_api::{ExternalPtr, IntoR, TryFromSexp};

#[derive(IntoR, TryFromSexp)]
pub struct Col(f64);

#[derive(ExternalPtr)]
#[externalptr(env)]
pub struct Handle {
    #[r_data]
    _r: RSidecar,
    // `pub`: only a public `#[r_data]` field gets R accessors.
    #[r_data]
    pub col: Col,
}

fn main() {}
