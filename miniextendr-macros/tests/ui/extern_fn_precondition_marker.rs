//! Test: a precondition marker on an `extern "C-unwind"` function is
//! rejected: it takes R values as they are, with no generated conversion to
//! unwrap the marker.

use miniextendr_api::miniextendr;

#[miniextendr]
#[unsafe(no_mangle)]
extern "C-unwind" fn C_raw_marker(x: miniextendr_api::Checked<i32>) -> miniextendr_api::SEXP {
    let _ = x;
    miniextendr_api::SEXP::null()
}

fn main() {}
