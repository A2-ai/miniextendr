//! Test: Vctrs requires #[vctrs(class = "name")] attribute.

use miniextendr_macros::Vctrs;

#[derive(Vctrs)]
struct Bad {
    x: f64,
}

fn main() {}
