//! Test: nested enum flatten field whose discriminant column (`kind_status`)
//! collides with a sibling field named `kind_status` when the inner enum uses
//! a non-default tag `#[dataframe(tag = "status")]`.
//!
//! The parse-time B1 check only fires for the default `"variant"` tag.
//! This case must be caught by the `assert_no_sibling_field_collision` const assertion.

use miniextendr_macros::DataFrameRow;

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(tag = "status")]
enum Inner {
    Active,
    Inactive { code: i32 },
}

/// Outer enum: `kind: Inner` would produce `kind_status` discriminant column
/// (because Inner uses `tag = "status"`), but the sibling field `kind_status`
/// also produces a column with that exact name → B1 collision.
#[derive(DataFrameRow)]
#[dataframe(tag = "_type")]
enum Outer {
    Wrap {
        id: i32,
        kind: Inner,
        kind_status: String, // collides with `kind`'s discriminant column
    },
    Other {
        id: i32,
    },
}

fn main() {}
