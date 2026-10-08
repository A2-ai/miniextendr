//! Test: `call = none` on a function taking a `Call` parameter.
//!
//! `call = none` passes no call (every condition has a NULL call, #1851), and
//! a `Call` / `CallerCall` parameter is bound from the call slot, so there is
//! nothing for it to receive.

use miniextendr_macros::miniextendr;

#[miniextendr(call = none)]
pub fn no_call_with_marker(x: i32, _call: miniextendr_api::Call) -> i32 {
    x
}

fn main() {}
