//! Test: unknown key in #[miniextendr(...)] on a 1-field struct (ALTREP path).

use miniextendr_macros::miniextendr;

#[miniextendr(class = "MyInts", typo = "bad")]
struct MyInts(Vec<i32>);

fn main() {}
