//! Test: extern "C-unwind" missing #[no_mangle]/#[unsafe(no_mangle)] should fail.

use miniextendr_macros::miniextendr;
#[miniextendr]
extern "C-unwind" fn C_bad(_x: miniextendr_api::ffi::SEXP) -> miniextendr_api::ffi::SEXP {
    _x
}

fn main() {}
