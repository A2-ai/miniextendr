//! Test: expand is not valid on struct-typed struct fields.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(DataFrameRow)]
struct Event {
    id: i32,
    #[dataframe(expand)]
    origin: Point,
}

fn main() {}
