//! Test: unknown vctrs field attribute.

use miniextendr_macros::Vctrs;

#[derive(Vctrs)]
#[vctrs(class = "my_class")]
struct Bad {
    #[vctrs(bogus)]
    x: f64,
}

fn main() {}
