//! Test: as_factor on Vec<i32> → compile error.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
#[dataframe(tag = "_type")]
enum Outer {
    Ev {
        #[dataframe(as_factor)]
        vals: Vec<i32>,
    },
    Other,
}

fn main() {}
