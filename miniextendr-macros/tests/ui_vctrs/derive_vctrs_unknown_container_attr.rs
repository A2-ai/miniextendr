//! Test: unknown vctrs container attribute.

use miniextendr_macros::Vctrs;

#[derive(Vctrs)]
#[vctrs(class = "my_class", bogus = true)]
struct Bad {
    x: f64,
}

fn main() {}
