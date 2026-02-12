//! Test: derive DataFrameRow on zero-field struct should fail.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
struct Empty {}

fn main() {}
