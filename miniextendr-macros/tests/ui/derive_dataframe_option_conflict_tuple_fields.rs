//! Test: a DataFrameRow option conflict points at the option written second,
//! also on a tuple field, which has no name to point at (it used to fall back
//! to `#[derive(DataFrameRow)]`). The two options may sit in separate
//! attributes.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
struct Pair(
    #[dataframe(as_list)]
    #[dataframe(width = 2)]
    Vec<i32>,
);

#[derive(DataFrameRow)]
enum Event {
    Moved(#[dataframe(expand, as_factor)] [f64; 2]),
}

fn main() {}
