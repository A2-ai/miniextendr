//! Test: Cow<'a, str> field without as_list is rejected.
//!
//! Cow's first generic argument is a lifetime, not a type.  The rejection
//! must fire on path identity alone — not by inspecting the first generic arg
//! — to catch this shape.

use miniextendr_macros::DataFrameRow;
use std::borrow::Cow;

#[derive(DataFrameRow)]
struct WithCow<'a> {
    id: i32,
    label: Cow<'a, str>,
}

fn main() {}
