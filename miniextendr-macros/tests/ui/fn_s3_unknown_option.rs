//! Test: unknown option inside s3().

use miniextendr_macros::miniextendr;

#[miniextendr(s3(class = "foo", unknown = "bar"))]
fn print_foo(x: miniextendr_api::sys::SEXP) -> miniextendr_api::sys::SEXP {
    x
}

fn main() {}
