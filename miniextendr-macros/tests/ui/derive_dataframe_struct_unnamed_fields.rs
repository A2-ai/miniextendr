//! Test: DataFrameRow only supports named fields on structs.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
struct Bad(f64, i32);

fn main() {}
