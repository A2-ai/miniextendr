//! Test: tuple variant in align enum should fail.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
#[dataframe(align)]
enum Event {
    Click { x: f64 },
    Data(i32, i32),
}

fn main() {}
