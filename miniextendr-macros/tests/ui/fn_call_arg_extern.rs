//! Test: `call_arg` on an `extern "C-unwind"` function.
//!
//! The option adds a `.call` formal whose call reaches Rust through the
//! generated call slot (#1834); an `extern "C-unwind"` function has no
//! generated R wrapper and no call slot.

use miniextendr_api::miniextendr;

#[miniextendr(call_arg)]
#[unsafe(no_mangle)]
extern "C-unwind" fn C_call_arg_raw(x: miniextendr_api::SEXP) -> miniextendr_api::SEXP {
    x
}

fn main() {}
