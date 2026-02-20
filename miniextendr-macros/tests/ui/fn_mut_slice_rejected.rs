//! Test: &mut [T] parameters at #[miniextendr] boundary are rejected.

use miniextendr_macros::miniextendr;

#[miniextendr]
fn mutate_slice(_data: &mut [i32]) {}

fn main() {}
