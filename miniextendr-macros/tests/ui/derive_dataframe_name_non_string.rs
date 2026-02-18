//! Test: dataframe name must be string literal.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
#[dataframe(name = 123)]
struct Point {
    x: f64,
    y: f64,
}

fn main() {}
