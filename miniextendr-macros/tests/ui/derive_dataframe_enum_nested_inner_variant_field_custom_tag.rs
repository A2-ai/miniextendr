//! Test: inner enum payload field named `kind` matching a custom tag `kind` →
//! after outer prefix expansion → collides with the outer discriminant column →
//! compile error via `assert_no_payload_field_collision`.

use miniextendr_macros::DataFrameRow;

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "kind")]
enum Inner {
    A,
    B { kind: i32 }, // field "kind" → outer prefix "status_" → "status_kind"
                     // outer discriminant also → "status_kind"
}

#[derive(DataFrameRow)]
#[dataframe(align)]
enum Outer {
    Wrap { id: i32, status: Inner },
    Other { id: i32 },
}

fn main() {}
