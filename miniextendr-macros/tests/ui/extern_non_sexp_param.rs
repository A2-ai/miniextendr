//! Test: extern "C-unwind" function with non-SEXP param should fail.

use miniextendr_macros::miniextendr;
#[miniextendr]
#[unsafe(no_mangle)]
extern "C-unwind" fn C_bad(_x: i32) -> miniextendr_api::ffi::SEXP {
    std::ptr::null_mut()
}

fn main() {}
