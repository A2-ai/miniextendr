//! Test: DataFrameRow does not support generic enums.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
enum Event<T> {
    A { val: T },
}

fn main() {}
