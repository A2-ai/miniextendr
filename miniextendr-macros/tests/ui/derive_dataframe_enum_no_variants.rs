//! Test: enum must have at least one variant.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
enum Empty {}

fn main() {}
