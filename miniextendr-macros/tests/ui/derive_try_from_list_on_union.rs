//! Test: TryFromList can only be derived for structs.

use miniextendr_macros::TryFromList;

#[derive(TryFromList)]
union Bad {
    x: f64,
    y: i32,
}

fn main() {}
