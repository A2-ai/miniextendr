//! Test: Option<HashMap<String, i32>> field without as_list is rejected.

use miniextendr_macros::DataFrameRow;
use std::collections::HashMap;

#[derive(DataFrameRow)]
struct Tallies {
    id: i32,
    counts: Option<HashMap<String, i32>>,
}

fn main() {}
