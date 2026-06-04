//! Test: `#[derive(IntoR)]` rejects enums (only newtype structs are valid).

use miniextendr_macros::IntoR;

#[derive(IntoR)]
enum Color {
    Red,
    Green,
}

fn main() {}
