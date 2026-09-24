//! Test: an omittable `several_ok` list is `Missing<Vec<T>>` or
//! `Missing<Box<[T]>>`. A fixed-size array or a borrowed slice under
//! `Missing<..>` has no decoder and errors at compile time (#1551).

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_missing_array(#[miniextendr(match_arg, several_ok)] modes: Missing<[Mode; 2]>) {}

fn main() {}
