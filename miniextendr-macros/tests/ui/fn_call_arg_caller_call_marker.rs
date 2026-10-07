//! Test: `call_arg` on a function taking a `CallerCall` parameter.
//!
//! A `caller` wrapper already takes `.call`, where NULL means the caller's
//! call; `call_arg` would give the same formal NULL as the wrapper's own call
//! (#1834). The macro refuses rather than picking one.

use miniextendr_macros::miniextendr;

#[miniextendr(noexport, call_arg)]
pub fn call_arg_with_caller_call(x: i32, _call: miniextendr_api::CallerCall) -> i32 {
    x
}

fn main() {}
