//! Test: a `Checked<T>` parameter that also carries the per-parameter
//! `no_preconditions` is rejected: the marker is a type, not one more flag of
//! the pair, so the two must agree.

use miniextendr_api::miniextendr;

#[miniextendr]
pub fn scale(#[miniextendr(no_preconditions)] n: miniextendr_api::Checked<i32>) -> i32 {
    *n
}

fn main() {}
