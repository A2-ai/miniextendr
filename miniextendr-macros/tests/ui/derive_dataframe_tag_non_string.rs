//! Test: dataframe tag must be string literal.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
#[dataframe(tag = 123)]
enum Event {
    A { id: i32 },
    B { id: i32 },
}

fn main() {}
