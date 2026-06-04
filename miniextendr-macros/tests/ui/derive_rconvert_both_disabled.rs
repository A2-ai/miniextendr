//! Test: `#[derive(RConvert)]` rejects disabling both conversion directions.

use miniextendr_macros::RConvert;

#[derive(RConvert)]
#[rconvert(from = false, into = false)]
struct Useless(f64);

fn main() {}
