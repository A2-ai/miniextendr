//! Test: expand on Vec<T> without width is rejected.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
struct Foo {
    #[dataframe(expand)]
    scores: Vec<f64>,
}

fn main() {}
