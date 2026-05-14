//! Test: a multi-segment path whose last segment has no generic args but whose
//! type does NOT implement `DataFrameRow` produces a clear compile-time error.
//!
//! Before fix #514, `classify_field_type` silently fell through to `Scalar`
//! for any path with `segs.len() > 1`. After the fix, `std::ffi::CString`
//! (and similar multi-segment non-DataFrameRow types) are correctly classified
//! as `Struct`, so the compile-time `_assert_inner_is_dataframe_row` assertion
//! fires with a clear diagnostic rather than producing a silent scalar column.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
struct Wrapper {
    id: i32,
    label: std::ffi::CString,
}

fn main() {}
