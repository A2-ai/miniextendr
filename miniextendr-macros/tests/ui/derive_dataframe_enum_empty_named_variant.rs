//! Test: enum variant must have at least one named field.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
enum Event {
    A {},
    B { id: i32 },
}

fn main() {}
