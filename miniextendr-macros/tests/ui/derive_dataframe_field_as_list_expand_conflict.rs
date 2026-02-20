//! Test: as_list and expand are mutually exclusive.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
struct Foo {
    #[dataframe(as_list, expand)]
    coords: [f64; 3],
}

fn main() {}
