//! Test: `#[derive(RConvert)]` rejects enums (only newtype structs are valid).

use miniextendr_macros::RConvert;

#[derive(RConvert)]
enum Color {
    Red,
    Green,
}

fn main() {}
