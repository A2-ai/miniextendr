//! Test: `call_arg` with `call = none`.
//!
//! `call_arg` adds a `.call` formal naming the call the conditions report
//! (#1834); `call = none` reports none (#1851). The macro refuses rather than
//! picking one.

use miniextendr_macros::miniextendr;

#[miniextendr(call_arg, call = none)]
pub fn no_call_with_call_arg(x: i32) -> i32 {
    x
}

fn main() {}
