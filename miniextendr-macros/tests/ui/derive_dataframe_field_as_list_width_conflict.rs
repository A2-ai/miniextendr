//! Test: as_list and width are mutually exclusive.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
struct Foo {
    #[dataframe(as_list, width = 3)]
    scores: Vec<f64>,
}

fn main() {}
