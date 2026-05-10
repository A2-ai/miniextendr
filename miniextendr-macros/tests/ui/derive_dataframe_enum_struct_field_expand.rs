//! Test: expand is not valid on struct-typed enum variant fields.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "_type")]
enum Event {
    Located {
        id: i32,
        #[dataframe(expand)]
        origin: Point,
    },
    Other {
        id: i32,
    },
}

fn main() {}
