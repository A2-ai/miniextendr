//! Test: `serialize` is refused on a generic struct: its class is registered
//! on first use, so a fresh session reading the save may not have it.

use miniextendr_macros::AltrepInteger;

#[derive(AltrepInteger)]
#[altrep(serialize)]
struct Dice<const N: usize>([i32; N]);

fn main() {}
