//! Test: Vctrs requires at least one field.

use miniextendr_macros::Vctrs;

#[derive(Vctrs)]
#[vctrs(class = "my_class")]
struct Bad {}

fn main() {}
