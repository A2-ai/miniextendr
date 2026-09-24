//! Test: `Missing<..>` has to be the outermost wrapper of a match_arg/choices
//! parameter. `Option<Missing<T>>` cannot report omission (the missing-argument
//! sentinel is not NULL), so it errors at compile time instead of failing the
//! `MatchArg` bound on `Missing<T>` (#1551).

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_layer_order(#[miniextendr(match_arg)] mode: Option<Missing<Mode>>) {}

fn main() {}
