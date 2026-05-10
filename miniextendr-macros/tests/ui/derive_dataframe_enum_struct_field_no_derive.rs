//! Test: struct field without DataFrameRow derive produces a clear trait-bound error.

use miniextendr_macros::DataFrameRow;

// Point deliberately does NOT derive DataFrameRow.
struct Point {
    x: f64,
    y: f64,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "_type")]
enum Event {
    Located { id: i32, origin: Point },
    Other { id: i32 },
}

fn main() {}
