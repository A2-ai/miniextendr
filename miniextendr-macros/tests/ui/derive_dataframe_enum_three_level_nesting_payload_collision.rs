//! Test: 3-level nesting where the innermost struct has a payload field whose name
//! collides with its own `#[dataframe(tag)]` discriminant column.
//!
//! Structure:
//!   Inner (struct, tag = "variant") { variant: String }  ← collision
//!   Middle (enum) { A { inner: Inner } }
//!   Outer (enum) { X { middle: Middle } }
//!
//! Expected: compile error at `#[derive(DataFrameRow)]` on `Middle`, because
//! `Middle::derive` emits `assert_no_payload_field_collision(<Inner as
//! DataFramePayloadFields>::FIELDS, <Inner as DataFramePayloadFields>::TAG)` —
//! and Inner::FIELDS contains "variant" which equals Inner::TAG = "variant".

use miniextendr_macros::DataFrameRow;

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(tag = "variant")]
enum Inner {
    A,
    B { variant: i32 }, // "variant" == Inner's tag → collision
}

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align)]
enum Middle {
    A { inner: Inner },
    Other,
}

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align)]
enum Outer {
    X { middle: Middle },
    Other,
}

fn main() {}
