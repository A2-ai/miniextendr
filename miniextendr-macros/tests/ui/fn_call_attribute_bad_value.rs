//! Test: `#[miniextendr(call = parent)]`.
//!
//! `call = ...` takes `none`, `wrapper` or `caller` (#1566).

use miniextendr_macros::miniextendr;

#[miniextendr(call = parent)]
pub fn bad_call_value(x: i32) -> i32 {
    x
}

fn main() {}
