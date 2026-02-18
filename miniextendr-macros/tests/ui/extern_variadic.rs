//! Test: extern function cannot have variadic parameter (Dots type is not SEXP).

use miniextendr_macros::miniextendr;

#[miniextendr]
#[unsafe(no_mangle)]
extern "C-unwind" fn C_bad(_dots: &miniextendr_api::Dots) -> miniextendr_api::ffi::SEXP {
    std::ptr::null_mut()
}

fn main() {}
