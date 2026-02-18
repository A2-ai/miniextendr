//! Test: DataFrameRow does not support unions.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
union Bad {
    x: f64,
    y: i32,
}

fn main() {}
