//! Test: width = 0 is rejected.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
struct Foo {
    #[dataframe(width = 0)]
    scores: Vec<f64>,
}

fn main() {}
