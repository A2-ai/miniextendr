//! Test: unnest on scalar field is rejected.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
struct Foo {
    #[dataframe(unnest)]
    score: f64,
}

fn main() {}
