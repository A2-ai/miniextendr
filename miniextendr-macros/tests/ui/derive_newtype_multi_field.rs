//! Test: `#[derive(TryFromSexp)]` rejects a struct with more than one field.

use miniextendr_macros::TryFromSexp;

#[derive(TryFromSexp)]
struct Pair(f64, f64);

fn main() {}
