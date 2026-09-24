//! Test: a `several_ok` list cannot be `Option<..>`. An omitted argument and
//! `NULL` already select every choice; `Missing<Vec<T>>` is the form that tells
//! an omitted argument apart (#1551).

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_several_ok_option(#[miniextendr(match_arg, several_ok)] modes: Option<Vec<Mode>>) {}

fn main() {}
