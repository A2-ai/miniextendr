//! Coercion must preserve the ordinary converter's valid inputs (#1112).

use miniextendr_api::{Missing, miniextendr};

/// Identity for a coerced i8 parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_scalar_i8(x: i8) -> i8 {
    x
}

/// Identity for a coerced Vec<i8> parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_vector_i8(x: Vec<i8>) -> Vec<i8> {
    x
}

/// Identity for a coerced i16 parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_scalar_i16(x: i16) -> i16 {
    x
}

/// Identity for a coerced Vec<i16> parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_vector_i16(x: Vec<i16>) -> Vec<i16> {
    x
}

/// Identity for a coerced u16 parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_scalar_u16(x: u16) -> u16 {
    x
}

/// Identity for a coerced Vec<u16> parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_vector_u16(x: Vec<u16>) -> Vec<u16> {
    x
}

/// Identity for a coerced u32 parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_scalar_u32(x: u32) -> u32 {
    x
}

/// Identity for a coerced Vec<u32> parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_vector_u32(x: Vec<u32>) -> Vec<u32> {
    x
}

/// Identity for a coerced i64 parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_scalar_i64(x: i64) -> i64 {
    x
}

/// Identity for a coerced Vec<i64> parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_vector_i64(x: Vec<i64>) -> Vec<i64> {
    x
}

/// Identity for a coerced u64 parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_scalar_u64(x: u64) -> u64 {
    x
}

/// Identity for a coerced Vec<u64> parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_vector_u64(x: Vec<u64>) -> Vec<u64> {
    x
}

/// Identity for a coerced isize parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_scalar_isize(x: isize) -> isize {
    x
}

/// Identity for a coerced Vec<isize> parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_vector_isize(x: Vec<isize>) -> Vec<isize> {
    x
}

/// Identity for a coerced usize parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_scalar_usize(x: usize) -> usize {
    x
}

/// Identity for a coerced Vec<usize> parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_vector_usize(x: Vec<usize>) -> Vec<usize> {
    x
}

/// Identity for a coerced f32 parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_scalar_f32(x: f32) -> f32 {
    x
}

/// Identity for a coerced Vec<f32> parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_vector_f32(x: Vec<f32>) -> Vec<f32> {
    x
}

/// Identity for a coerced native i32 parameter: whole-number doubles widen.
/// @param x Integer, whole-number double, logical, or raw scalar.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_scalar_i32(x: i32) -> i32 {
    x
}

/// Identity for a coerced native Vec<i32> parameter.
/// @param x Integer, whole-number double, logical, or raw vector.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_vector_i32(x: Vec<i32>) -> Vec<i32> {
    x
}

/// Identity for a coerced native f64 parameter: integers, logicals, and raws widen.
/// @param x Numeric, logical, or raw scalar.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_scalar_f64(x: f64) -> f64 {
    x
}

/// Identity for a coerced native Vec<f64> parameter.
/// @param x Numeric, logical, or raw vector.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_vector_f64(x: Vec<f64>) -> Vec<f64> {
    x
}

/// `no_preconditions` wrapper for the widened native vector: the Rust side batches failures.
/// @param x Doubles, possibly fractional, NaN, or out of range.
#[miniextendr(coerce, no_preconditions, noexport)]
pub fn coerce_input_no_preconditions_i32(x: Vec<i32>) -> Vec<i32> {
    x
}

/// Without coerce, a native i32 parameter keeps its INTSXP-only conversion.
/// @param x Integer scalar.
#[miniextendr(no_coerce, noexport)]
pub fn coerce_input_native_default(x: i32) -> i32 {
    x
}

/// Identity for a coerced bool parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_scalar_bool(x: bool) -> bool {
    x
}

/// Identity for a coerced Vec<bool> parameter.
/// @param x Input to convert without narrowing the accepted R types.
#[miniextendr(coerce, no_strict, noexport)]
pub fn coerce_input_vector_bool(x: Vec<bool>) -> Vec<bool> {
    x
}

/// Per-parameter coercion leaves the neighboring logical parameter unchanged.
/// @param x Logical or integer zero/one with coercion enabled.
/// @param y Logical with coercion disabled.
#[miniextendr(no_coerce, noexport)]
pub fn coerce_input_per_arg(#[miniextendr(coerce)] x: bool, y: bool) -> Vec<bool> {
    vec![x, y]
}

/// Identity for a coerced Option<bool> parameter (#2274): NULL and NA read None.
/// @param x NULL, NA, a logical, or integer zero or one.
#[miniextendr(coerce, noexport)]
pub fn coerce_input_optional_bool(x: Option<bool>) -> Option<bool> {
    x
}

/// Identity for a coerced Vec<Option<bool>> parameter: NA elements read None.
/// @param x Logical vector, or integer zeros, ones and NAs.
#[miniextendr(coerce, noexport)]
pub fn coerce_input_optional_vector_bool(x: Vec<Option<bool>>) -> Vec<Option<bool>> {
    x
}

/// Identity for a coerced Option<i32> parameter: NULL and NA read None.
/// @param x NULL, NA, or an integer, whole-number double, logical, or raw scalar.
#[miniextendr(coerce, noexport)]
pub fn coerce_input_optional_i32(x: Option<i32>) -> Option<i32> {
    x
}

/// Identity for a coerced Vec<Option<i32>> parameter: NA elements read None.
/// @param x Integer, whole-number double, logical, or raw vector.
#[miniextendr(coerce, noexport)]
pub fn coerce_input_optional_vector_i32(x: Vec<Option<i32>>) -> Vec<Option<i32>> {
    x
}

/// Identity for a coerced Option<f64> parameter: NULL and NA read None.
/// @param x NULL, NA, or a numeric, logical, or raw scalar.
#[miniextendr(coerce, noexport)]
pub fn coerce_input_optional_f64(x: Option<f64>) -> Option<f64> {
    x
}

/// Identity for a coerced Vec<Option<f64>> parameter: NA elements read None.
/// @param x Numeric, logical, or raw vector.
#[miniextendr(coerce, noexport)]
pub fn coerce_input_optional_vector_f64(x: Vec<Option<f64>>) -> Vec<Option<f64>> {
    x
}

/// Whether a coerced Option<f64> read None (TRUE) or a value (FALSE).
/// @param x NULL, NA, NaN, or a numeric, logical, or raw scalar.
#[miniextendr(coerce, noexport)]
pub fn coerce_input_optional_f64_is_none(x: Option<f64>) -> bool {
    x.is_none()
}

/// Per-parameter coercion widens an optional double; its neighbor keeps the
/// bare conversion.
/// @param x NULL, NA, or a numeric, logical, or raw scalar.
/// @param y NULL, NA, or a double scalar.
#[miniextendr(no_coerce, noexport)]
pub fn coerce_input_optional_per_arg(
    #[miniextendr(coerce)] x: Option<f64>,
    y: Option<f64>,
) -> Vec<Option<f64>> {
    vec![x, y]
}

/// `no_preconditions` wrapper for an optional integer vector: the Rust side batches failures.
/// @param x Doubles, possibly NA, fractional, NaN, or out of range.
#[miniextendr(coerce, no_preconditions, noexport)]
pub fn coerce_input_no_preconditions_optional_i32(x: Vec<Option<i32>>) -> Vec<Option<i32>> {
    x
}

/// `no_preconditions` wrapper for an optional logical vector read from integers.
/// @param x Integers, possibly NA or other than zero and one.
#[miniextendr(coerce, no_preconditions, noexport)]
pub fn coerce_input_no_preconditions_optional_bool(x: Vec<Option<bool>>) -> Vec<Option<bool>> {
    x
}

/// `no_preconditions` wrapper for an optional integer scalar.
/// @param x A scalar, possibly NA, fractional, or NaN.
#[miniextendr(coerce, no_preconditions, noexport)]
pub fn coerce_input_no_preconditions_optional_scalar_i32(x: Option<i32>) -> Option<i32> {
    x
}

/// A coerced integer under `no_na`: the C wrapper refuses an NA before conversion.
/// @param x An integer-like scalar, not NA.
#[miniextendr(noexport)]
pub fn coerce_input_no_na_i32(#[miniextendr(coerce, no_na)] x: i32) -> i32 {
    x
}

/// A coerced optional double under `no_na`: NULL still reads None, an NA is refused.
/// @param x NULL, or a numeric, logical, or raw scalar, not NA.
#[miniextendr(noexport)]
pub fn coerce_input_no_na_optional_f64(
    #[miniextendr(coerce, no_na)] x: Option<f64>,
) -> Option<f64> {
    x
}

/// A coerced optional integer vector under `no_na`: an NA element is refused.
/// @param x Integer-like values, none of them NA.
#[miniextendr(noexport)]
pub fn coerce_input_no_na_optional_vector_i32(
    #[miniextendr(coerce, no_na)] x: Vec<Option<i32>>,
) -> Vec<Option<i32>> {
    x
}

/// A coerced `Missing<i32>`: an absent argument reads NA, a present one converts as a coerced `i32`.
/// @param x An integer-like scalar, or nothing.
#[miniextendr(noexport)]
pub fn coerce_input_missing_i32(#[miniextendr(coerce)] x: Missing<i32>) -> Option<i32> {
    x.into_option()
}

/// A coerced `Missing<Option<f64>>`: tells an absent argument, a None and a value apart.
/// @param x NULL, NA, a numeric, logical, or raw scalar, or nothing.
#[miniextendr(noexport)]
pub fn coerce_input_missing_optional_f64(#[miniextendr(coerce)] x: Missing<Option<f64>>) -> String {
    match x {
        Missing::Absent => "absent".to_string(),
        Missing::Present(None) => "none".to_string(),
        Missing::Present(Some(value)) => format!("{value}"),
    }
}

/// A coerced `Missing<Vec<bool>>`: an absent argument reads an empty vector.
/// @param x Logicals, or integers that are zero or one, or nothing.
#[miniextendr(noexport)]
pub fn coerce_input_missing_vector_bool(#[miniextendr(coerce)] x: Missing<Vec<bool>>) -> Vec<bool> {
    x.unwrap_or_default()
}

/// `no_preconditions` wrappers reach the coercion checker without an R precondition.
/// @param x Integer-like values, possibly invalid.
#[miniextendr(coerce, no_preconditions, noexport)]
pub fn coerce_input_no_preconditions(x: Vec<u16>) -> Vec<u16> {
    x
}

/// Strict conversion takes precedence over coercion.
/// @param x Integer-like scalar.
#[miniextendr(coerce, strict, noexport)]
pub fn coerce_input_strict(x: i64) -> i64 {
    x
}

/// Strict vector conversion takes precedence over coercion.
/// @param x Integer-like vector.
#[miniextendr(coerce, strict, noexport)]
pub fn coerce_input_strict_vector(x: Vec<i64>) -> Vec<i64> {
    x
}
