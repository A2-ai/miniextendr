//! Test: #[dataframe(tag)] is only for enums.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
struct Point {
    x: f64,
    y: f64,
}

fn main() {}
