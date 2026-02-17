//! Test: unit variant in align enum should fail.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
#[dataframe(align)]
enum Event {
    Click { x: f64 },
    Nothing,
}

fn main() {}
