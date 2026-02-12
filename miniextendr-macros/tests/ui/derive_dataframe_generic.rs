//! Test: derive DataFrameRow on generic struct should fail.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
struct Row<T> {
    value: T,
}

fn main() {}
