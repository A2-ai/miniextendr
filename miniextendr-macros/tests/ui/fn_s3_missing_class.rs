//! Test: s3() requires class argument.

use miniextendr_macros::miniextendr;

#[miniextendr(s3(generic = "print"))]
fn print_myclass(x: miniextendr_api::sys::SEXP) -> miniextendr_api::sys::SEXP {
    x
}

fn main() {}
