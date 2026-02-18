//! Test: Vctrs requires named fields.

use miniextendr_macros::Vctrs;

#[derive(Vctrs)]
#[vctrs(class = "my_class")]
struct Bad(f64, i32);

fn main() {}
