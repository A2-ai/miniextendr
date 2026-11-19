//! Test: the newtype derives need exactly one field; the error points at the
//! fields, or at the name of a unit struct.

use miniextendr_macros::{IntoR, TryFromSexp};

#[derive(IntoR)]
pub struct Two(pub i32, pub i32);

#[derive(TryFromSexp)]
pub struct Unit;

fn main() {}
