//! Test: `impl Trait` in return position without `IntoR` should fail.
//!
//! The C wrapper calls `IntoR::into_sexp(result)` but `impl Display` does not
//! implement `IntoR`, so this should produce a trait bound error.

use miniextendr_macros::miniextendr;
use std::fmt::Display;

#[miniextendr]
fn returns_impl_display() -> impl Display {
    42i32
}

fn main() {}
