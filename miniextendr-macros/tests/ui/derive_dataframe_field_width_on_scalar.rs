//! Test: width on a scalar type is rejected.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
struct Foo {
    #[dataframe(width = 3)]
    value: f64,
}

fn main() {}
