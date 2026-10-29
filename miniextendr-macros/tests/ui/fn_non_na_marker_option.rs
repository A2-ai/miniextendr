//! Test: `NonNa<Option<T>>` is rejected: a value without `NA` leaves
//! `Option` only `NULL` to mean, which `Option<T>` with `no_na` spells.

use miniextendr_api::miniextendr;

#[miniextendr]
pub fn first(x: miniextendr_api::NonNa<Option<f64>>) -> f64 {
    x.into_inner().unwrap_or(0.0)
}

fn main() {}
