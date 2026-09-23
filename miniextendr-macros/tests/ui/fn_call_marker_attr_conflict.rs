//! Test: a `Call` marker parameter and `call = caller` on one function.
//!
//! The marker and the attribute are two spellings of the same decision; when
//! they disagree the macro refuses rather than picking one (#1566).

use miniextendr_macros::miniextendr;

#[miniextendr(noexport, call = caller)]
pub fn disagree(x: i32, _call: miniextendr_api::Call) -> i32 {
    x
}

fn main() {}
