//! Test: inner enum payload field named `variant` (matching the inner tag) →
//! after outer prefix expansion → collides with the outer discriminant column →
//! compile error via `assert_no_payload_field_collision`.

use miniextendr_macros::DataFrameRow;

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "variant")]
enum Inner {
    A,
    B { variant: i32 }, // field named "variant" — same as the tag → collision
}

#[derive(DataFrameRow)]
#[dataframe(align, tag = "_type")]
enum Outer {
    Wrap { id: i32, kind: Inner }, // outer discriminant also → "kind_variant"
    Other { id: i32 },
}

fn main() {}
