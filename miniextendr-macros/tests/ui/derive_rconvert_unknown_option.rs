//! Test: `#[derive(RConvert)]` rejects unknown `#[rconvert(...)]` options.

use miniextendr_macros::RConvert;

#[derive(RConvert)]
#[rconvert(backward)]
struct Wrong(f64);

fn main() {}
