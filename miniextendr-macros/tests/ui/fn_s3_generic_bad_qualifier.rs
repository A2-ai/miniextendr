//! Test: `s3(generic = "pkg:::generic")` names an internal function.

use miniextendr_macros::miniextendr;

#[miniextendr(s3(generic = "dplyr:::dplyr_reconstruct", class = "foo"))]
fn dplyr_reconstruct_foo(x: miniextendr_api::SEXP) -> miniextendr_api::SEXP {
    x
}

fn main() {}
