//! Test: extern "C-unwind" missing #[no_mangle]/#[unsafe(no_mangle)] should fail.

use miniextendr_macros::miniextendr;
#[miniextendr]
extern "C-unwind" fn C_bad(_x: miniextendr_api::sys::SEXP) -> miniextendr_api::sys::SEXP {
    _x
}

fn main() {}
