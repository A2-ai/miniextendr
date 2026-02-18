//! Test: Vctrs does not support generic structs.

use miniextendr_macros::Vctrs;

#[derive(Vctrs)]
#[vctrs(class = "my_class")]
struct Bad<T> {
    x: T,
}

fn main() {}
