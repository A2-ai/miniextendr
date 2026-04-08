//! Arrow adapter tests — zero-copy conversions between R and Arrow.

use miniextendr_api::arrow_impl::{
    Array, ArrayRef, BooleanArray, Float64Array, Int32Array, RecordBatch, StringArray, UInt8Array,
};
use miniextendr_api::miniextendr;

// region: Float64Array (zero-copy)

/// Test Float64Array roundtrip through Arrow.
/// @param v Arrow Float64Array from R.
#[miniextendr]
pub fn arrow_f64_roundtrip(v: Float64Array) -> Float64Array {
    v
}

/// Test summing elements of a Float64Array.
/// @param v Arrow Float64Array from R.
#[miniextendr]
pub fn arrow_f64_sum(v: Float64Array) -> f64 {
    v.iter().flatten().sum()
}

/// Test getting the length of a Float64Array.
/// @param v Arrow Float64Array from R.
#[miniextendr]
pub fn arrow_f64_len(v: Float64Array) -> i32 {
    v.len() as i32
}

/// Test counting null values in a Float64Array.
/// @param v Arrow Float64Array from R.
#[miniextendr]
pub fn arrow_f64_null_count(v: Float64Array) -> i32 {
    v.logical_null_count() as i32
}

// endregion

// region: Int32Array (zero-copy)

/// Test Int32Array roundtrip through Arrow.
/// @param v Arrow Int32Array from R.
#[miniextendr]
pub fn arrow_i32_roundtrip(v: Int32Array) -> Int32Array {
    v
}

/// Test summing elements of an Int32Array.
/// @param v Arrow Int32Array from R.
#[miniextendr]
pub fn arrow_i32_sum(v: Int32Array) -> i32 {
    v.iter().flatten().sum()
}

/// Test counting null values in an Int32Array.
/// @param v Arrow Int32Array from R.
#[miniextendr]
pub fn arrow_i32_null_count(v: Int32Array) -> i32 {
    v.logical_null_count() as i32
}

// endregion

// region: UInt8Array (zero-copy)

/// Test UInt8Array roundtrip through Arrow.
/// @param v Arrow UInt8Array from R.
#[miniextendr]
pub fn arrow_u8_roundtrip(v: UInt8Array) -> UInt8Array {
    v
}

/// Test getting the length of a UInt8Array.
/// @param v Arrow UInt8Array from R.
#[miniextendr]
pub fn arrow_u8_len(v: UInt8Array) -> i32 {
    v.len() as i32
}

// endregion

// region: BooleanArray (copy)

/// Test BooleanArray roundtrip through Arrow.
/// @param v Arrow BooleanArray from R.
#[miniextendr]
pub fn arrow_bool_roundtrip(v: BooleanArray) -> BooleanArray {
    v
}

/// Test counting null values in a BooleanArray.
/// @param v Arrow BooleanArray from R.
#[miniextendr]
pub fn arrow_bool_null_count(v: BooleanArray) -> i32 {
    v.logical_null_count() as i32
}

// endregion

// region: StringArray (copy)

/// Test StringArray roundtrip through Arrow.
/// @param v Arrow StringArray from R.
#[miniextendr]
pub fn arrow_string_roundtrip(v: StringArray) -> StringArray {
    v
}

/// Test counting null values in a StringArray.
/// @param v Arrow StringArray from R.
#[miniextendr]
pub fn arrow_string_null_count(v: StringArray) -> i32 {
    v.logical_null_count() as i32
}

// endregion

// region: RecordBatch (data.frame)

/// Test RecordBatch roundtrip through Arrow.
/// @param rb Arrow RecordBatch from R.
#[miniextendr]
pub fn arrow_recordbatch_roundtrip(rb: RecordBatch) -> RecordBatch {
    rb
}

/// Test getting the number of rows in a RecordBatch.
/// @param rb Arrow RecordBatch from R.
#[miniextendr]
pub fn arrow_recordbatch_nrow(rb: RecordBatch) -> i32 {
    rb.num_rows() as i32
}

/// Test getting the number of columns in a RecordBatch.
/// @param rb Arrow RecordBatch from R.
#[miniextendr]
pub fn arrow_recordbatch_ncol(rb: RecordBatch) -> i32 {
    rb.num_columns() as i32
}

/// Test extracting column names from a RecordBatch.
/// @param rb Arrow RecordBatch from R.
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

/// Test ArrayRef roundtrip through Arrow dynamic dispatch.
/// @param v Arrow ArrayRef from R.
#[miniextendr]
pub fn arrow_arrayref_roundtrip(v: ArrayRef) -> ArrayRef {
    v
}

/// Test getting the length of a dynamically-typed ArrayRef.
/// @param v Arrow ArrayRef from R.
#[miniextendr]
pub fn arrow_arrayref_len(v: ArrayRef) -> i32 {
    v.len() as i32
}

// endregion

// region: Empty vectors

/// Test empty Float64Array roundtrip through Arrow.
/// @param v Empty Arrow Float64Array from R.
#[miniextendr]
pub fn arrow_f64_empty_roundtrip(v: Float64Array) -> Float64Array {
    v
}

/// Test empty Int32Array roundtrip through Arrow.
/// @param v Empty Arrow Int32Array from R.
#[miniextendr]
pub fn arrow_i32_empty_roundtrip(v: Int32Array) -> Int32Array {
    v
}

// endregion

// region: Factor (DictionaryArray)

/// Test factor roundtrip through Arrow StringDictionaryArray.
/// @param v Arrow StringDictionaryArray from R factor.
#[miniextendr]
pub fn arrow_factor_roundtrip(
    v: miniextendr_api::arrow_impl::StringDictionaryArray,
) -> miniextendr_api::arrow_impl::StringDictionaryArray {
    v
}

/// Test getting the length of a StringDictionaryArray from an R factor.
/// @param v Arrow StringDictionaryArray from R factor.
#[miniextendr]
pub fn arrow_factor_len(v: miniextendr_api::arrow_impl::StringDictionaryArray) -> i32 {
    v.len() as i32
}

// endregion

// region: Date (Date32Array)

/// Test Date roundtrip through Arrow Date32Array.
/// @param v Arrow Date32Array from R Date.
#[miniextendr]
pub fn arrow_date_roundtrip(
    v: miniextendr_api::arrow_impl::Date32Array,
) -> miniextendr_api::arrow_impl::Date32Array {
    v
}

/// Test getting the length of a Date32Array.
/// @param v Arrow Date32Array from R Date.
#[miniextendr]
pub fn arrow_date_len(v: miniextendr_api::arrow_impl::Date32Array) -> i32 {
    v.len() as i32
}

// endregion

// region: POSIXct (TimestampSecondArray via helper)

/// Test POSIXct roundtrip through Arrow TimestampSecondArray.
/// @param v R POSIXct SEXP to convert to Arrow timestamp.
#[miniextendr]
pub fn arrow_posixct_roundtrip(v: miniextendr_api::ffi::SEXP) -> miniextendr_api::ffi::SEXP {
    use miniextendr_api::arrow_impl::posixct_to_timestamp;
    use miniextendr_api::into_r::IntoR;
    let arr = posixct_to_timestamp(v).expect("posixct_to_timestamp failed");
    arr.into_sexp()
}

/// Test getting the length of a POSIXct converted to Arrow timestamp.
/// @param v R POSIXct SEXP to convert to Arrow timestamp.
#[miniextendr]
pub fn arrow_posixct_len(v: miniextendr_api::ffi::SEXP) -> i32 {
    use miniextendr_api::arrow_impl::posixct_to_timestamp;
    let arr = posixct_to_timestamp(v).expect("posixct_to_timestamp failed");
    arr.len() as i32
}

// endregion

// region: RecordBatch with typed columns

/// Test RecordBatch roundtrip with typed columns.
/// @param rb Arrow RecordBatch with typed columns from R.
#[miniextendr]
pub fn arrow_recordbatch_typed_roundtrip(rb: RecordBatch) -> RecordBatch {
    rb
}

// endregion

// region: Upstream example-derived fixtures

/// Filter a Float64Array to keep only non-null values.
/// @param v Arrow Float64Array from R.
#[miniextendr]
pub fn arrow_f64_filter_non_null(v: Float64Array) -> Float64Array {
    let values: Vec<f64> = v.iter().flatten().collect();
    Float64Array::from(values)
}

/// Compute the mean of a Float64Array (ignoring nulls).
/// @param v Arrow Float64Array from R.
#[miniextendr]
pub fn arrow_f64_mean(v: Float64Array) -> Option<f64> {
    let values: Vec<f64> = v.iter().flatten().collect();
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

/// Get the data type name of an ArrayRef.
/// @param v Arrow ArrayRef from R.
#[miniextendr]
pub fn arrow_arrayref_type_name(v: ArrayRef) -> String {
    format!("{}", v.data_type())
}

// endregion
