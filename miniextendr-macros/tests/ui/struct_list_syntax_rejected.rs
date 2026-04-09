//! Test: Meta::List syntax on #[miniextendr] struct is rejected.

use miniextendr_macros::miniextendr;

#[miniextendr(list(nested = "bad"))]
struct MyStruct {
    x: i32,
    y: f64,
}

fn main() {}
