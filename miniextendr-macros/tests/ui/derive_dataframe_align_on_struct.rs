//! Test: #[dataframe(align)] is only for enums, not structs.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
#[dataframe(align)]
struct Point {
    x: f64,
    y: f64,
}

fn main() {}
