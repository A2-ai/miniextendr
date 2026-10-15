//! #2111: a `"#` in a generated R wrapper must not end the raw string
//! literal it is embedded in.
#![allow(dead_code)]
use miniextendr_api::{SEXP, miniextendr};

/// `print()`: the last line reads "# Not shown: sd". Nothing else.
#[miniextendr(s3(generic = "print", class = "foo"), invisible)]
pub fn print_foo(x: SEXP, _dots: ...) -> SEXP {
    x
}

/// Splits on `sep`, which is "##" unless given.
#[miniextendr]
pub fn split_on(x: &str, #[miniextendr(default = "\"##\"")] sep: &str) -> Vec<String> {
    x.split(sep).map(str::to_owned).collect()
}

fn main() {}
