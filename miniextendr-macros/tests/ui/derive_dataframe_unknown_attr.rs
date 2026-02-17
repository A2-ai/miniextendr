//! Test: unknown dataframe attribute.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
#[dataframe(bogus)]
struct Point {
    x: f64,
    y: f64,
}

fn main() {}
