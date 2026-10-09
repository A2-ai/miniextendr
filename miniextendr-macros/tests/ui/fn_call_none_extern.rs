//! Test: `call = none` on an `extern "C-unwind"` function.
//!
//! The option empties the generated call slot (#1851); an `extern "C-unwind"`
//! function has no generated R wrapper and no call slot.

use miniextendr_api::miniextendr;

#[miniextendr(call = none)]
#[unsafe(no_mangle)]
extern "C-unwind" fn C_no_call_raw(x: miniextendr_api::SEXP) -> miniextendr_api::SEXP {
    x
}

fn main() {}
