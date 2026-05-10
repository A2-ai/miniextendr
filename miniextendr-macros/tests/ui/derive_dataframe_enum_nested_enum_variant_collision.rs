//! Test: nested enum flatten field whose discriminant column (`kind_variant`)
//! collides with a sibling field named `kind_variant` → compile error.

use miniextendr_macros::DataFrameRow;

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(tag = "variant")]
enum Inner {
    A,
    B { val: i32 },
}

/// Outer enum: `kind: Inner` would produce `kind_variant` discriminant column,
/// but the sibling field is ALSO named `kind_variant` → collision.
#[derive(DataFrameRow)]
#[dataframe(tag = "_type")]
enum Outer {
    Wrap {
        id: i32,
        kind: Inner,
        kind_variant: String, // collides with kind's discriminant column
    },
    Other {
        id: i32,
    },
}

fn main() {}
