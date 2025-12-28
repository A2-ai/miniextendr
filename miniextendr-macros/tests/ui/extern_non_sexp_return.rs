//! Test: extern "C-unwind" function with non-SEXP return should fail.

use miniextendr_macros::miniextendr;
#[miniextendr]
#[unsafe(no_mangle)]
extern "C-unwind" fn C_bad(_x: miniextendr_api::ffi::SEXP) -> i32 {
    0
}

fn main() {}
