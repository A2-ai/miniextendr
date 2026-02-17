//! Test: same field name with different types across variants should fail.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
#[dataframe(align)]
enum Mixed {
    A { value: f64 },
    B { value: String },
}

fn main() {}
