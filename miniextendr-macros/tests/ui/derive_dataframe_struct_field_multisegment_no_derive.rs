//! Test: a multi-segment path whose last segment has no generic args but whose
//! type does NOT implement `DataFrameRow` produces a clear compile-time error.
//!
//! Before fix #514, `classify_field_type` silently fell through to `Scalar`
//! for any path with `segs.len() > 1`. After the fix, multi-segment paths
//! (e.g. `local::NotADataFrameRow`) are correctly classified as `Struct`,
//! so the compile-time `_assert_inner_is_dataframe_row` assertion fires with
//! a clear diagnostic rather than producing a silent scalar column.
//!
//! Uses a local non-implementing type (no stdlib) so the snapshot is stable
//! across toolchain updates — stdlib types emit version-dependent help notes.

use miniextendr_macros::DataFrameRow;

mod local {
    #[derive(Debug, Clone)]
    pub struct NotADataFrameRow;
}

#[derive(DataFrameRow)]
struct Wrapper {
    id: i32,
    label: local::NotADataFrameRow,
}

fn main() {}
