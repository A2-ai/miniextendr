//! Test: Arc<Inner> field without as_list is rejected.

use miniextendr_macros::DataFrameRow;
use std::sync::Arc;

#[derive(DataFrameRow)]
struct WithArc {
    id: i32,
    payload: Arc<String>,
}

fn main() {}
