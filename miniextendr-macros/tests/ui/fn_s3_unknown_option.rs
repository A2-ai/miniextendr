//! Test: unknown option inside s3().

use miniextendr_macros::miniextendr;

#[miniextendr(s3(class = "foo", unknown = "bar"))]
fn print_foo(x: miniextendr_api::ffi::SEXP) -> miniextendr_api::ffi::SEXP {
    x
}

fn main() {}
