//! Test: a `LazyDots` parameter on an `extern "C-unwind"` function.
//!
//! `LazyDots` is the generated R wrapper's frame; an extern function has no
//! generated wrapper (#1892).

use miniextendr_api::miniextendr;

#[miniextendr]
#[unsafe(no_mangle)]
extern "C-unwind" fn C_lazy_raw(
    x: miniextendr_api::SEXP,
    rest: miniextendr_api::LazyDots,
) -> miniextendr_api::SEXP {
    let _ = rest;
    x
}

fn main() {}
