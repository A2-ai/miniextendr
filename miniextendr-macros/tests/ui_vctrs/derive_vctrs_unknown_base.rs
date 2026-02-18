//! Test: unknown vctrs base type.

use miniextendr_macros::Vctrs;

#[derive(Vctrs)]
#[vctrs(class = "my_class", base = "invalid")]
struct Bad {
    x: f64,
}

fn main() {}
