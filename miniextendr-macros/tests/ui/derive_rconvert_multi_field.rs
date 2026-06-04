//! Test: `#[derive(RConvert)]` rejects a struct with more than one field.

use miniextendr_macros::RConvert;

#[derive(RConvert)]
struct Pair(f64, f64);

fn main() {}
