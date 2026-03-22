//! Arrow adapter tests — zero-copy conversions between R and Arrow.

use miniextendr_api::arrow_impl::{
    Array, ArrayRef, BooleanArray, Float64Array, Int32Array, RecordBatch, StringArray, UInt8Array,
};
use miniextendr_api::miniextendr;

// region: Float64Array (zero-copy)

/// @noRd
#[miniextendr]
pub fn arrow_f64_roundtrip(v: Float64Array) -> Float64Array {
    v
}

/// @noRd
#[miniextendr]
pub fn arrow_f64_sum(v: Float64Array) -> f64 {
    v.iter().flatten().sum()
}

/// @noRd
#[miniextendr]
pub fn arrow_f64_len(v: Float64Array) -> i32 {
    v.len() as i32
}

/// @noRd
#[miniextendr]
pub fn arrow_f64_null_count(v: Float64Array) -> i32 {
    v.logical_null_count() as i32
}

// endregion

// region: Int32Array (zero-copy)

/// @noRd
#[miniextendr]
pub fn arrow_i32_roundtrip(v: Int32Array) -> Int32Array {
    v
}

/// @noRd
#[miniextendr]
pub fn arrow_i32_sum(v: Int32Array) -> i32 {
    v.iter().flatten().sum()
}

/// @noRd
#[miniextendr]
pub fn arrow_i32_null_count(v: Int32Array) -> i32 {
    v.logical_null_count() as i32
}

// endregion

// region: UInt8Array (zero-copy)

/// @noRd
#[miniextendr]
pub fn arrow_u8_roundtrip(v: UInt8Array) -> UInt8Array {
    v
}

/// @noRd
#[miniextendr]
pub fn arrow_u8_len(v: UInt8Array) -> i32 {
    v.len() as i32
}

// endregion

// region: BooleanArray (copy)

/// @noRd
#[miniextendr]
pub fn arrow_bool_roundtrip(v: BooleanArray) -> BooleanArray {
    v
}

/// @noRd
#[miniextendr]
pub fn arrow_bool_null_count(v: BooleanArray) -> i32 {
    v.logical_null_count() as i32
}

// endregion

// region: StringArray (copy)

/// @noRd
#[miniextendr]
pub fn arrow_string_roundtrip(v: StringArray) -> StringArray {
    v
}

/// @noRd
#[miniextendr]
pub fn arrow_string_null_count(v: StringArray) -> i32 {
    v.logical_null_count() as i32
}

// endregion

// region: RecordBatch (data.frame)

/// @noRd
#[miniextendr]
pub fn arrow_recordbatch_roundtrip(rb: RecordBatch) -> RecordBatch {
    rb
}

/// @noRd
#[miniextendr]
pub fn arrow_recordbatch_nrow(rb: RecordBatch) -> i32 {
    rb.num_rows() as i32
}

/// @noRd
#[miniextendr]
pub fn arrow_recordbatch_ncol(rb: RecordBatch) -> i32 {
    rb.num_columns() as i32
}

/// @noRd
#[miniextendr]
pub fn arrow_recordbatch_column_names(rb: RecordBatch) -> Vec<String> {
    rb.schema()
        .fields()
        .iter()
        .map(|f: &std::sync::Arc<miniextendr_api::arrow_impl::Field>| f.name().clone())
        .collect()
}

// endregion

// region: ArrayRef (dynamic dispatch)

/// @noRd
#[miniextendr]
pub fn arrow_arrayref_roundtrip(v: ArrayRef) -> ArrayRef {
    v
}

/// @noRd
#[miniextendr]
pub fn arrow_arrayref_len(v: ArrayRef) -> i32 {
    v.len() as i32
}

// endregion

// region: Empty vectors

/// @noRd
#[miniextendr]
pub fn arrow_f64_empty_roundtrip(v: Float64Array) -> Float64Array {
    v
}

/// @noRd
#[miniextendr]
pub fn arrow_i32_empty_roundtrip(v: Int32Array) -> Int32Array {
    v
}

// endregion
