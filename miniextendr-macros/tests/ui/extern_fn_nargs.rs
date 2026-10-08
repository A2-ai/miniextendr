//! Test: an `NArgs` parameter on an `extern "C-unwind"` function.
//!
//! The generated R wrapper passes `nargs()` for the parameter; an extern
//! function has no generated wrapper (#1860).

use miniextendr_api::miniextendr;

#[miniextendr]
#[unsafe(no_mangle)]
extern "C-unwind" fn C_counted_raw(
    x: miniextendr_api::SEXP,
    n: miniextendr_api::NArgs,
) -> miniextendr_api::SEXP {
    let _ = n;
    x
}

fn main() {}
