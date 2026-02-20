//! Test: unknown field-level dataframe attribute.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
struct Foo {
    #[dataframe(bogus)]
    value: f64,
}

fn main() {}
