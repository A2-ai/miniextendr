//! Test: `#[miniextendr(call = parent)]`.
//!
//! `call = ...` takes `wrapper`, `caller` (#1566) or `none` (#1851).

use miniextendr_macros::miniextendr;

#[miniextendr(call = parent)]
pub fn bad_call_value(x: i32) -> i32 {
    x
}

fn main() {}
