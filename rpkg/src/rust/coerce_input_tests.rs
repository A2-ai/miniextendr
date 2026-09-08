//! Coercion must preserve the ordinary converter's valid inputs (#1112).

use miniextendr_api::miniextendr;

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

/// Worker dispatch preserves logical inputs with coercion enabled.
/// @param x Logical or integer zero/one.
#[cfg(feature = "worker-thread")]
#[miniextendr(worker, coerce, noexport)]
pub fn coerce_input_worker(x: bool) -> bool {
    x
}

/// Coercion does not change the unsupported Option<bool> mapping.
/// @param x Nullable logical scalar.
#[miniextendr(coerce, noexport)]
pub fn coerce_input_optional(x: Option<bool>) -> bool {
    x.unwrap_or(false)
}

/// Fast wrappers reach the coercion checker without an R precondition.
/// @param x Integer-like values, possibly invalid.
#[miniextendr(coerce, fast, noexport)]
pub fn coerce_input_fast(x: Vec<u16>) -> Vec<u16> {
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
