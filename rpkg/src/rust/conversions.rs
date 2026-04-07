//! Comprehensive conversions matrix for [`#[miniextendr]`](miniextendr_api::miniextendr) arguments and returns.

use miniextendr_api::ffi::{RLogical, Rboolean, SEXP};
use miniextendr_api::{IntoR, ListMut, miniextendr};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

// -----------------------------------------------------------------------------
/// Roundtrip test: R integer -> Rust `i32` -> R integer.
/// @param x Input value.
#[miniextendr]
pub fn conv_i32_arg(x: i32) -> i32 {
    x
}

/// Return test: Rust `i32` -> R integer.
#[miniextendr]
pub fn conv_i32_ret() -> i32 {
    1
}

// -----------------------------------------------------------------------------
/// Roundtrip test: R double -> Rust `f64` -> R double.
/// @param x Input value.
#[miniextendr]
pub fn conv_f64_arg(x: f64) -> f64 {
    x
}

/// Return test: Rust `f64` -> R double.
#[miniextendr]
pub fn conv_f64_ret() -> f64 {
    1.25
}

// -----------------------------------------------------------------------------
/// Roundtrip test: R raw element -> Rust `u8` -> R raw element.
/// @param x Input value.
#[miniextendr]
pub fn conv_u8_arg(x: u8) -> u8 {
    x
}

/// Return test: Rust `u8` -> R raw element.
#[miniextendr]
pub fn conv_u8_ret() -> u8 {
    7u8
}

// -----------------------------------------------------------------------------
/// Roundtrip test: R logical -> Rust `Rboolean` -> R logical.
/// @param x Input value.
#[miniextendr]
pub fn conv_rbool_arg(x: Rboolean) -> Rboolean {
    x
}

/// Return test: Rust `Rboolean` -> R logical.
#[miniextendr]
pub fn conv_rbool_ret() -> Rboolean {
    Rboolean::TRUE
}

// -----------------------------------------------------------------------------
/// Roundtrip test: R logical -> Rust `RLogical` -> R logical.
/// @param x Input value.
#[miniextendr]
pub fn conv_rlog_arg(x: RLogical) -> RLogical {
    x
}

/// Return test: Rust `RLogical` -> R logical.
#[miniextendr]
pub fn conv_rlog_ret() -> RLogical {
    RLogical::TRUE
}

// -----------------------------------------------------------------------------
/// Roundtrip test: R character scalar -> Rust `String` -> R character scalar.
/// @param x Input value.
#[miniextendr]
pub fn conv_string_arg(x: String) -> String {
    x
}

/// Return test: Rust `String` -> R character scalar.
#[miniextendr]
pub fn conv_string_ret() -> String {
    "hi".to_string()
}

// -----------------------------------------------------------------------------
/// Return test: Rust `&str` -> R character scalar.
#[miniextendr]
pub fn conv_str_ret() -> &'static str {
    "hi"
}

// -----------------------------------------------------------------------------
/// Roundtrip test: R SEXP -> Rust `SEXP` -> R SEXP (passthrough).
/// @param x Input value.
#[miniextendr]
pub fn conv_sexp_arg(x: SEXP) -> SEXP {
    x
}

/// Return test: Rust `SEXP` -> R (via `IntoR`).
#[miniextendr]
pub fn conv_sexp_ret() -> SEXP {
    1i32.into_sexp()
}

// -----------------------------------------------------------------------------
/// Roundtrip test: R numeric -> Rust `i64` -> R numeric.
/// @param x Input value.
#[miniextendr]
pub fn conv_i64_arg(x: i64) -> i64 {
    x
}

/// Return test: Rust `i64` -> R numeric.
#[miniextendr]
pub fn conv_i64_ret() -> i64 {
    1i64
}

// -----------------------------------------------------------------------------
/// Roundtrip test: R numeric -> Rust `u64` -> R numeric.
/// @param x Input value.
#[miniextendr]
pub fn conv_u64_arg(x: u64) -> u64 {
    x
}

/// Return test: Rust `u64` -> R numeric.
#[miniextendr]
pub fn conv_u64_ret() -> u64 {
    1u64
}

// -----------------------------------------------------------------------------
/// Roundtrip test: R numeric -> Rust `isize` -> R numeric.
/// @param x Input value.
#[miniextendr]
pub fn conv_isize_arg(x: isize) -> isize {
    x
}

/// Return test: Rust `isize` -> R numeric.
#[miniextendr]
pub fn conv_isize_ret() -> isize {
    1isize
}

// -----------------------------------------------------------------------------
/// Roundtrip test: R numeric -> Rust `usize` -> R numeric.
/// @param x Input value.
#[miniextendr]
pub fn conv_usize_arg(x: usize) -> usize {
    x
}

/// Return test: Rust `usize` -> R numeric.
#[miniextendr]
pub fn conv_usize_ret() -> usize {
    1usize
}

// -----------------------------------------------------------------------------
/// Roundtrip test: R integer -> Rust `i8` -> R integer.
/// @param x Input value.
#[miniextendr]
pub fn conv_i8_arg(x: i8) -> i8 {
    x
}

/// Return test: Rust `i8` -> R integer.
#[miniextendr]
pub fn conv_i8_ret() -> i8 {
    1i8
}

// -----------------------------------------------------------------------------
/// Roundtrip test: R integer -> Rust `i16` -> R integer.
/// @param x Input value.
#[miniextendr]
pub fn conv_i16_arg(x: i16) -> i16 {
    x
}

/// Return test: Rust `i16` -> R integer.
#[miniextendr]
pub fn conv_i16_ret() -> i16 {
    1i16
}

// -----------------------------------------------------------------------------
/// Roundtrip test: R integer -> Rust `u16` -> R integer.
/// @param x Input value.
#[miniextendr]
pub fn conv_u16_arg(x: u16) -> u16 {
    x
}

/// Return test: Rust `u16` -> R integer.
#[miniextendr]
pub fn conv_u16_ret() -> u16 {
    1u16
}

// -----------------------------------------------------------------------------
/// Roundtrip test: R integer -> Rust `u32` -> R integer.
/// @param x Input value.
#[miniextendr]
pub fn conv_u32_arg(x: u32) -> u32 {
    x
}

/// Return test: Rust `u32` -> R integer.
#[miniextendr]
pub fn conv_u32_ret() -> u32 {
    1u32
}

// -----------------------------------------------------------------------------
/// Roundtrip test: R double -> Rust `f32` -> R double.
/// @param x Input value.
#[miniextendr]
pub fn conv_f32_arg(x: f32) -> f32 {
    x
}

/// Return test: Rust `f32` -> R double.
#[miniextendr]
pub fn conv_f32_ret() -> f32 {
    1.5f32
}

// -----------------------------------------------------------------------------
/// Test `Option<i32>` argument: returns 1 if Some, 0 if None (NA).
/// @param x Input value or NULL.
#[miniextendr]
pub fn conv_opt_i32_is_some(x: Option<i32>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

/// Return test: `Some(10)` as `Option<i32>` -> R integer.
#[miniextendr]
pub fn conv_opt_i32_some() -> Option<i32> {
    Some(10)
}

/// Return test: `None` as `Option<i32>` -> R NA_integer_.
#[miniextendr]
pub fn conv_opt_i32_none() -> Option<i32> {
    None
}

// -----------------------------------------------------------------------------
/// Test `Option<f64>` argument: returns 1 if Some, 0 if None (NA).
/// @param x Input value or NULL.
#[miniextendr]
pub fn conv_opt_f64_is_some(x: Option<f64>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

/// Return test: `Some(2.5)` as `Option<f64>` -> R double.
#[miniextendr]
pub fn conv_opt_f64_some() -> Option<f64> {
    Some(2.5)
}

/// Return test: `None` as `Option<f64>` -> R NA_real_.
#[miniextendr]
pub fn conv_opt_f64_none() -> Option<f64> {
    None
}

// -----------------------------------------------------------------------------
/// Test `Option<bool>` argument: returns 1 if Some, 0 if None (NA).
/// @param x Input value or NULL.
#[miniextendr]
pub fn conv_opt_bool_is_some(x: Option<bool>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

/// Return test: `Some(true)` as `Option<bool>` -> R logical.
#[miniextendr]
pub fn conv_opt_bool_some() -> Option<bool> {
    Some(true)
}

/// Return test: `None` as `Option<bool>` -> R NA.
#[miniextendr]
pub fn conv_opt_bool_none() -> Option<bool> {
    None
}

// -----------------------------------------------------------------------------
/// Test `Option<String>` argument: returns 1 if Some, 0 if None (NA).
/// @param x Input value or NULL.
#[miniextendr]
pub fn conv_opt_string_is_some(x: Option<String>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

/// Return test: `Some("opt")` as `Option<String>` -> R character scalar.
#[miniextendr]
pub fn conv_opt_string_some() -> Option<String> {
    Some("opt".to_string())
}

/// Return test: `None` as `Option<String>` -> R NA_character_.
#[miniextendr]
pub fn conv_opt_string_none() -> Option<String> {
    None
}

// -----------------------------------------------------------------------------
/// Test `Option<i8>` argument: returns 1 if Some, 0 if None (NA).
/// @param x Input value or NULL.
#[miniextendr]
pub fn conv_opt_i8_is_some(x: Option<i8>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// Test `Option<i16>` argument: returns 1 if Some, 0 if None (NA).
/// @param x Input value or NULL.
#[miniextendr]
pub fn conv_opt_i16_is_some(x: Option<i16>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// Test `Option<i64>` argument: returns 1 if Some, 0 if None (NA).
/// @param x Input value or NULL.
#[miniextendr]
pub fn conv_opt_i64_is_some(x: Option<i64>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// Test `Option<isize>` argument: returns 1 if Some, 0 if None (NA).
/// @param x Input value or NULL.
#[miniextendr]
pub fn conv_opt_isize_is_some(x: Option<isize>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// Test `Option<u16>` argument: returns 1 if Some, 0 if None (NA).
/// @param x Input value or NULL.
#[miniextendr]
pub fn conv_opt_u16_is_some(x: Option<u16>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// Test `Option<u32>` argument: returns 1 if Some, 0 if None (NA).
/// @param x Input value or NULL.
#[miniextendr]
pub fn conv_opt_u32_is_some(x: Option<u32>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// Test `Option<u64>` argument: returns 1 if Some, 0 if None (NA).
/// @param x Input value or NULL.
#[miniextendr]
pub fn conv_opt_u64_is_some(x: Option<u64>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// Test `Option<usize>` argument: returns 1 if Some, 0 if None (NA).
/// @param x Input value or NULL.
#[miniextendr]
pub fn conv_opt_usize_is_some(x: Option<usize>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// Test `Option<f32>` argument: returns 1 if Some, 0 if None (NA).
/// @param x Input value or NULL.
#[miniextendr]
pub fn conv_opt_f32_is_some(x: Option<f32>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// Test `Option<u8>` argument: returns 1 if Some, 0 if None (NA).
/// @param x Input value or NULL.
#[miniextendr]
pub fn conv_opt_u8_is_some(x: Option<u8>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// Test `Option<Rboolean>` argument: returns 1 if Some, 0 if None (NA).
/// @param x Input value or NULL.
#[miniextendr]
pub fn conv_opt_rbool_is_some(x: Option<Rboolean>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// Test `Option<RLogical>` argument: returns 1 if Some, 0 if None (NA).
/// @param x Input value or NULL.
#[miniextendr]
pub fn conv_opt_rlog_is_some(x: Option<RLogical>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// Test `&[i32]` slice argument: returns the length of the slice.
/// @param x Input integer vector.
#[miniextendr]
pub fn conv_slice_i32_len(x: &'static [i32]) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// Test `&[f64]` slice argument: returns the length of the slice.
/// @param x Input double vector.
#[miniextendr]
pub fn conv_slice_f64_len(x: &'static [f64]) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// Test `&[u8]` slice argument: returns the length of the slice.
/// @param x Input raw vector.
#[miniextendr]
pub fn conv_slice_u8_len(x: &'static [u8]) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// Test `&[RLogical]` slice argument: returns the length of the slice.
/// @param x Input logical vector.
#[miniextendr]
pub fn conv_slice_rlog_len(x: &'static [RLogical]) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// Test `&i32` reference argument: dereferences and returns the value.
/// @param x Input integer reference.
#[miniextendr]
pub fn conv_ref_i32_arg(x: &'static i32) -> i32 {
    *x
}

/// Test `&mut i32` mutable reference: increments by one and returns.
/// @param x Input integer (mutated in-place).
#[miniextendr]
pub fn conv_ref_mut_i32_add_one(x: &'static mut i32) -> i32 {
    *x += 1;
    *x
}

/// Test `&mut [i32]` mutable slice: increments each element and returns length.
/// @param x Input integer vector (mutated in-place).
#[miniextendr]
pub fn conv_slice_mut_i32_add_one(x: &'static mut [i32]) -> i32 {
    for v in x.iter_mut() {
        *v += 1;
    }
    x.len() as i32
}

/// Test `&mut [u8]` mutable slice: wrapping-adds one to each element.
/// @param x Input raw vector (mutated in-place).
#[miniextendr]
pub fn conv_slice_mut_u8_add_one(x: &'static mut [u8]) -> i32 {
    for v in x.iter_mut() {
        *v = v.wrapping_add(1);
    }
    x.len() as i32
}

/// Test `Option<&i32>` argument: returns 1 if Some, 0 if None.
/// @param x Input integer reference or NULL.
#[miniextendr]
pub fn conv_opt_ref_i32_is_some(x: Option<&'static i32>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

/// Test `Option<&mut [i32]>` argument: returns 1 if Some, 0 if None.
/// @param x Input integer vector or NULL.
#[miniextendr]
pub fn conv_opt_mut_slice_i32_is_some(x: Option<&'static mut [i32]>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

/// Test `Vec<&i32>` argument: returns the length of the vector.
/// @param x Input list of integer references.
#[miniextendr]
pub fn conv_vec_ref_i32_len(x: Vec<&'static i32>) -> i32 {
    x.len() as i32
}

/// Test `Vec<&[i32]>` argument: returns the total length across all slices.
/// @param x Input list of integer vectors.
#[miniextendr]
pub fn conv_vec_slice_i32_total_len(x: Vec<&'static [i32]>) -> i32 {
    x.iter().map(|s| s.len() as i32).sum()
}

/// Test `Vec<&mut [i32]>` argument: increments all elements and returns total length.
/// @param x Input list of integer vectors (mutated in-place).
#[miniextendr]
pub fn conv_vec_mut_slice_i32_add_one(x: Vec<&'static mut [i32]>) -> i32 {
    let mut total = 0;
    for slice in x {
        for v in slice.iter_mut() {
            *v += 1;
        }
        total += slice.len() as i32;
    }
    total
}

// -----------------------------------------------------------------------------
/// Test `Vec<i32>` argument: returns the length of the vector.
/// @param x Input integer vector.
#[miniextendr]
pub fn conv_vec_i32_len(x: Vec<i32>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `Vec<i32>` -> R integer vector.
#[miniextendr]
pub fn conv_vec_i32_ret() -> Vec<i32> {
    vec![1, 2, 3]
}

// -----------------------------------------------------------------------------
/// Test `Vec<f64>` argument: returns the length of the vector.
/// @param x Input double vector.
#[miniextendr]
pub fn conv_vec_f64_len(x: Vec<f64>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `Vec<f64>` -> R double vector.
#[miniextendr]
pub fn conv_vec_f64_ret() -> Vec<f64> {
    vec![1.0, 2.0, 3.0]
}

// -----------------------------------------------------------------------------
/// Test `Vec<u8>` argument: returns the length of the vector.
/// @param x Input raw vector.
#[miniextendr]
pub fn conv_vec_u8_len(x: Vec<u8>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `Vec<u8>` -> R raw vector.
#[miniextendr]
pub fn conv_vec_u8_ret() -> Vec<u8> {
    vec![1u8, 2u8, 3u8]
}

// -----------------------------------------------------------------------------
/// Test `Vec<RLogical>` argument: returns the length of the vector.
/// @param x Input logical vector.
#[miniextendr]
pub fn conv_vec_rlog_len(x: Vec<RLogical>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `Vec<RLogical>` -> R logical vector.
#[miniextendr]
pub fn conv_vec_rlog_ret() -> Vec<RLogical> {
    vec![RLogical::TRUE, RLogical::FALSE]
}

// -----------------------------------------------------------------------------
/// Test `Vec<bool>` argument: returns the length of the vector.
/// @param x Input logical vector.
#[miniextendr]
pub fn conv_vec_bool_len(x: Vec<bool>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `Vec<bool>` -> R logical vector.
#[miniextendr]
pub fn conv_vec_bool_ret() -> Vec<bool> {
    vec![true, false, true]
}

// -----------------------------------------------------------------------------
/// Test `Vec<String>` argument: returns the length of the vector.
/// @param x Input character vector.
#[miniextendr]
pub fn conv_vec_string_len(x: Vec<String>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `Vec<String>` -> R character vector.
#[miniextendr]
pub fn conv_vec_string_ret() -> Vec<String> {
    vec!["a".to_string(), "b".to_string()]
}

// -----------------------------------------------------------------------------
/// Test `Vec<i8>` argument: returns the length of the vector.
/// @param x Input integer vector (coerced from R integer).
#[miniextendr]
pub fn conv_vec_i8_len(x: Vec<i8>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// Test `Vec<i16>` argument: returns the length of the vector.
/// @param x Input integer vector (coerced from R integer).
#[miniextendr]
pub fn conv_vec_i16_len(x: Vec<i16>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// Test `Vec<i64>` argument: returns the length of the vector.
/// @param x Input numeric vector (coerced from R numeric).
#[miniextendr]
pub fn conv_vec_i64_len(x: Vec<i64>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// Test `Vec<isize>` argument: returns the length of the vector.
/// @param x Input numeric vector (coerced from R numeric).
#[miniextendr]
pub fn conv_vec_isize_len(x: Vec<isize>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// Test `Vec<u16>` argument: returns the length of the vector.
/// @param x Input integer vector (coerced from R integer).
#[miniextendr]
pub fn conv_vec_u16_len(x: Vec<u16>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// Test `Vec<u32>` argument: returns the length of the vector.
/// @param x Input integer vector (coerced from R integer).
#[miniextendr]
pub fn conv_vec_u32_len(x: Vec<u32>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// Test `Vec<u64>` argument: returns the length of the vector.
/// @param x Input numeric vector (coerced from R numeric).
#[miniextendr]
pub fn conv_vec_u64_len(x: Vec<u64>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// Test `Vec<usize>` argument: returns the length of the vector.
/// @param x Input numeric vector (coerced from R numeric).
#[miniextendr]
pub fn conv_vec_usize_len(x: Vec<usize>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// Test `Vec<f32>` argument: returns the length of the vector.
/// @param x Input double vector (coerced from R double).
#[miniextendr]
pub fn conv_vec_f32_len(x: Vec<f32>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// Test `Vec<Option<i32>>` argument: returns the length of the vector.
/// @param x Input integer vector (may contain NA).
#[miniextendr]
pub fn conv_vec_opt_i32_len(x: Vec<Option<i32>>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `Vec<Option<i32>>` -> R integer vector with NA.
#[miniextendr]
pub fn conv_vec_opt_i32_ret() -> Vec<Option<i32>> {
    vec![Some(1), None, Some(3)]
}

// -----------------------------------------------------------------------------
/// Test `Vec<Option<f64>>` argument: returns the length of the vector.
/// @param x Input double vector (may contain NA).
#[miniextendr]
pub fn conv_vec_opt_f64_len(x: Vec<Option<f64>>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `Vec<Option<f64>>` -> R double vector with NA.
#[miniextendr]
pub fn conv_vec_opt_f64_ret() -> Vec<Option<f64>> {
    vec![Some(1.0), None, Some(3.0)]
}

// -----------------------------------------------------------------------------
/// Test `Vec<Option<bool>>` argument: returns the length of the vector.
/// @param x Input logical vector (may contain NA).
#[miniextendr]
pub fn conv_vec_opt_bool_len(x: Vec<Option<bool>>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `Vec<Option<bool>>` -> R logical vector with NA.
#[miniextendr]
pub fn conv_vec_opt_bool_ret() -> Vec<Option<bool>> {
    vec![Some(true), None, Some(false)]
}

// -----------------------------------------------------------------------------
/// Test `Vec<Option<String>>` argument: returns the length of the vector.
/// @param x Input character vector (may contain NA).
#[miniextendr]
pub fn conv_vec_opt_string_len(x: Vec<Option<String>>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `Vec<Option<String>>` -> R character vector with NA.
#[miniextendr]
pub fn conv_vec_opt_string_ret() -> Vec<Option<String>> {
    vec![Some("a".to_string()), None, Some("b".to_string())]
}

// -----------------------------------------------------------------------------
/// Return test: Rust `Vec<Option<RLogical>>` -> R logical vector with NA.
#[miniextendr]
pub fn conv_vec_opt_rlog_ret() -> Vec<Option<RLogical>> {
    vec![Some(RLogical::TRUE), None, Some(RLogical::FALSE)]
}

/// Return test: Rust `Vec<Option<Rboolean>>` -> R logical vector with NA.
#[miniextendr]
pub fn conv_vec_opt_rbool_ret() -> Vec<Option<Rboolean>> {
    vec![Some(Rboolean::TRUE), None, Some(Rboolean::FALSE)]
}

// -----------------------------------------------------------------------------
/// Test `Vec<Option<i8>>` argument: returns the length of the vector.
/// @param x Input integer vector (may contain NA).
#[miniextendr]
pub fn conv_vec_opt_i8_len(x: Vec<Option<i8>>) -> i32 {
    x.len() as i32
}

/// Test `Vec<Option<u8>>` argument: returns the length of the vector.
/// @param x Input raw vector (may contain NA).
#[miniextendr]
pub fn conv_vec_opt_u8_len(x: Vec<Option<u8>>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// Test `HashSet<i32>` argument: returns the number of unique elements.
/// @param x Input integer vector (duplicates removed).
#[miniextendr]
pub fn conv_hashset_i32_len(x: HashSet<i32>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `HashSet<i32>` -> R integer vector.
#[miniextendr]
pub fn conv_hashset_i32_ret() -> HashSet<i32> {
    vec![1, 2, 3].into_iter().collect()
}

// -----------------------------------------------------------------------------
/// Test `HashSet<u8>` argument: returns the number of unique elements.
/// @param x Input raw vector (duplicates removed).
#[miniextendr]
pub fn conv_hashset_u8_len(x: HashSet<u8>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `HashSet<u8>` -> R raw vector.
#[miniextendr]
pub fn conv_hashset_u8_ret() -> HashSet<u8> {
    vec![1u8, 2u8, 3u8].into_iter().collect()
}

// -----------------------------------------------------------------------------
/// Test `HashSet<String>` argument: returns the number of unique elements.
/// @param x Input character vector (duplicates removed).
#[miniextendr]
pub fn conv_hashset_string_len(x: HashSet<String>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `HashSet<String>` -> R character vector.
#[miniextendr]
pub fn conv_hashset_string_ret() -> HashSet<String> {
    vec!["a".to_string(), "b".to_string()].into_iter().collect()
}

// -----------------------------------------------------------------------------
/// Test `HashSet<RLogical>` argument: returns the number of unique elements.
/// @param x Input logical vector (duplicates removed).
#[miniextendr]
pub fn conv_hashset_rlog_len(x: HashSet<RLogical>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `HashSet<RLogical>` -> R logical vector.
#[miniextendr]
pub fn conv_hashset_rlog_ret() -> HashSet<RLogical> {
    vec![RLogical::TRUE, RLogical::FALSE].into_iter().collect()
}

// -----------------------------------------------------------------------------
/// Test `BTreeSet<i32>` argument: returns the number of unique sorted elements.
/// @param x Input integer vector (duplicates removed, sorted).
#[miniextendr]
pub fn conv_btreeset_i32_len(x: BTreeSet<i32>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `BTreeSet<i32>` -> R integer vector (sorted).
#[miniextendr]
pub fn conv_btreeset_i32_ret() -> BTreeSet<i32> {
    vec![1, 2, 3].into_iter().collect()
}

// -----------------------------------------------------------------------------
/// Test `BTreeSet<u8>` argument: returns the number of unique sorted elements.
/// @param x Input raw vector (duplicates removed, sorted).
#[miniextendr]
pub fn conv_btreeset_u8_len(x: BTreeSet<u8>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `BTreeSet<u8>` -> R raw vector (sorted).
#[miniextendr]
pub fn conv_btreeset_u8_ret() -> BTreeSet<u8> {
    vec![1u8, 2u8, 3u8].into_iter().collect()
}

// -----------------------------------------------------------------------------
/// Test `BTreeSet<String>` argument: returns the number of unique sorted elements.
/// @param x Input character vector (duplicates removed, sorted).
#[miniextendr]
pub fn conv_btreeset_string_len(x: BTreeSet<String>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `BTreeSet<String>` -> R character vector (sorted).
#[miniextendr]
pub fn conv_btreeset_string_ret() -> BTreeSet<String> {
    vec!["a".to_string(), "b".to_string()].into_iter().collect()
}

// -----------------------------------------------------------------------------
/// Test `HashMap<String, i32>` argument: returns the number of entries.
/// @param x Input named list of integers.
#[miniextendr]
pub fn conv_hashmap_i32_len(x: HashMap<String, i32>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `HashMap<String, i32>` -> R named list of integers.
#[miniextendr]
pub fn conv_hashmap_i32_ret() -> HashMap<String, i32> {
    vec![("a".to_string(), 1), ("b".to_string(), 2)]
        .into_iter()
        .collect()
}

// -----------------------------------------------------------------------------
/// Test `HashMap<String, f64>` argument: returns the number of entries.
/// @param x Input named list of doubles.
#[miniextendr]
pub fn conv_hashmap_f64_len(x: HashMap<String, f64>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `HashMap<String, f64>` -> R named list of doubles.
#[miniextendr]
pub fn conv_hashmap_f64_ret() -> HashMap<String, f64> {
    vec![("a".to_string(), 1.5), ("b".to_string(), 2.5)]
        .into_iter()
        .collect()
}

// -----------------------------------------------------------------------------
/// Test `HashMap<String, String>` argument: returns the number of entries.
/// @param x Input named list of strings.
#[miniextendr]
pub fn conv_hashmap_string_len(x: HashMap<String, String>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `HashMap<String, String>` -> R named list of strings.
#[miniextendr]
pub fn conv_hashmap_string_ret() -> HashMap<String, String> {
    vec![
        ("a".to_string(), "x".to_string()),
        ("b".to_string(), "y".to_string()),
    ]
    .into_iter()
    .collect()
}

// -----------------------------------------------------------------------------
/// Test `HashMap<String, RLogical>` argument: returns the number of entries.
/// @param x Input named list of logicals.
#[miniextendr]
pub fn conv_hashmap_rlog_len(x: HashMap<String, RLogical>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `HashMap<String, RLogical>` -> R named list of logicals.
#[miniextendr]
pub fn conv_hashmap_rlog_ret() -> HashMap<String, RLogical> {
    vec![
        ("a".to_string(), RLogical::TRUE),
        ("b".to_string(), RLogical::FALSE),
    ]
    .into_iter()
    .collect()
}

// -----------------------------------------------------------------------------
/// Test `BTreeMap<String, i32>` argument: returns the number of entries.
/// @param x Input named list of integers (sorted by key).
#[miniextendr]
pub fn conv_btreemap_i32_len(x: BTreeMap<String, i32>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `BTreeMap<String, i32>` -> R named list of integers (sorted).
#[miniextendr]
pub fn conv_btreemap_i32_ret() -> BTreeMap<String, i32> {
    vec![("a".to_string(), 1), ("b".to_string(), 2)]
        .into_iter()
        .collect()
}

// -----------------------------------------------------------------------------
/// Test `BTreeMap<String, f64>` argument: returns the number of entries.
/// @param x Input named list of doubles (sorted by key).
#[miniextendr]
pub fn conv_btreemap_f64_len(x: BTreeMap<String, f64>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `BTreeMap<String, f64>` -> R named list of doubles (sorted).
#[miniextendr]
pub fn conv_btreemap_f64_ret() -> BTreeMap<String, f64> {
    vec![("a".to_string(), 1.5), ("b".to_string(), 2.5)]
        .into_iter()
        .collect()
}

// -----------------------------------------------------------------------------
/// Test `BTreeMap<String, String>` argument: returns the number of entries.
/// @param x Input named list of strings (sorted by key).
#[miniextendr]
pub fn conv_btreemap_string_len(x: BTreeMap<String, String>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `BTreeMap<String, String>` -> R named list of strings (sorted).
#[miniextendr]
pub fn conv_btreemap_string_ret() -> BTreeMap<String, String> {
    vec![
        ("a".to_string(), "x".to_string()),
        ("b".to_string(), "y".to_string()),
    ]
    .into_iter()
    .collect()
}

// -----------------------------------------------------------------------------
/// Test `BTreeMap<String, RLogical>` argument: returns the number of entries.
/// @param x Input named list of logicals (sorted by key).
#[miniextendr]
pub fn conv_btreemap_rlog_len(x: BTreeMap<String, RLogical>) -> i32 {
    x.len() as i32
}

/// Return test: Rust `BTreeMap<String, RLogical>` -> R named list of logicals (sorted).
#[miniextendr]
pub fn conv_btreemap_rlog_ret() -> BTreeMap<String, RLogical> {
    vec![
        ("a".to_string(), RLogical::TRUE),
        ("b".to_string(), RLogical::FALSE),
    ]
    .into_iter()
    .collect()
}

// -----------------------------------------------------------------------------
/// Note: `mut` tests macro handling of mutable parameters; becomes unused after expansion.
#[allow(unused_mut)]
/// Test `ListMut` argument: sets the first element to 99 and returns the length.
/// @param x Input list (first element mutated in-place).
#[miniextendr]
pub fn conv_list_mut_set_first(mut x: ListMut) -> i32 {
    let len = x.len() as i32;
    if len > 0 {
        let _ = x.set(0, 99i32.into_sexp());
    }
    len
}

// -----------------------------------------------------------------------------
/// Test `Result<i32, ()>` argument: returns 1 if Ok, 0 if Err (NULL).
/// @param x Input value (NULL becomes Err).
#[miniextendr]
pub fn conv_result_i32_arg(x: Result<i32, ()>) -> i32 {
    if x.is_ok() { 1 } else { 0 }
}

/// Test `Result<Vec<i32>, ()>` argument: returns 1 if Ok, 0 if Err (NULL).
/// @param x Input vector (NULL becomes Err).
#[miniextendr]
pub fn conv_result_vec_i32_arg(x: Result<Vec<i32>, ()>) -> i32 {
    if x.is_ok() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// Return test: `Ok(9)` as `Result<i32, ()>` -> R integer.
#[miniextendr]
pub fn conv_result_i32_ok() -> Result<i32, ()> {
    Ok(9)
}

/// Return test: `Err(())` as `Result<i32, ()>` -> R NULL.
#[miniextendr]
pub fn conv_result_i32_err() -> Result<i32, ()> {
    Err(())
}

// -----------------------------------------------------------------------------
/// Return test: `Ok(9.5)` as `Result<f64, ()>` -> R double.
#[miniextendr]
pub fn conv_result_f64_ok() -> Result<f64, ()> {
    Ok(9.5)
}

/// Return test: `Err(())` as `Result<f64, ()>` -> R NULL.
#[miniextendr]
pub fn conv_result_f64_err() -> Result<f64, ()> {
    Err(())
}

// -----------------------------------------------------------------------------
/// Return test: `Ok("ok")` as `Result<String, ()>` -> R character scalar.
#[miniextendr]
pub fn conv_result_string_ok() -> Result<String, ()> {
    Ok("ok".to_string())
}

/// Return test: `Err(())` as `Result<String, ()>` -> R NULL.
#[miniextendr]
pub fn conv_result_string_err() -> Result<String, ()> {
    Err(())
}

// -----------------------------------------------------------------------------
/// Return test: `Ok(vec![1, 2])` as `Result<Vec<i32>, ()>` -> R integer vector.
#[miniextendr]
pub fn conv_result_vec_i32_ok() -> Result<Vec<i32>, ()> {
    Ok(vec![1, 2])
}

/// Return test: `Err(())` as `Result<Vec<i32>, ()>` -> R NULL.
#[miniextendr]
pub fn conv_result_vec_i32_err() -> Result<Vec<i32>, ()> {
    Err(())
}

// region: Extended conversions - nested types, coercion, char

// --- char conversions (char ↔ length-1 string) ---
/// Roundtrip test: R character scalar -> Rust `char` -> R character scalar.
/// @param x Input single character.
#[miniextendr]
pub fn conv_char_arg(x: char) -> char {
    x
}

/// Return test: Rust `char` (Unicode) -> R character scalar.
#[miniextendr]
pub fn conv_char_ret() -> char {
    'α' // Unicode char to test UTF-8
}

// --- Vec coercion (i8/i16/u16 → i32, f32 → f64) ---
/// Return test: Rust `Vec<i8>` -> R integer vector (coerced via i32).
#[miniextendr]
pub fn conv_vec_i8_ret() -> Vec<i8> {
    vec![1i8, -1i8, 127i8]
}

/// Return test: Rust `Vec<i16>` -> R integer vector (coerced via i32).
#[miniextendr]
pub fn conv_vec_i16_ret() -> Vec<i16> {
    vec![1i16, -1i16, 32767i16]
}

/// Return test: Rust `Vec<u16>` -> R integer vector (coerced via i32).
#[miniextendr]
pub fn conv_vec_u16_ret() -> Vec<u16> {
    vec![1u16, 100u16, 65535u16]
}

/// Return test: Rust `Vec<f32>` -> R double vector (coerced via f64).
#[miniextendr]
pub fn conv_vec_f32_ret() -> Vec<f32> {
    vec![1.5f32, 2.5f32, -0.5f32]
}

// --- HashSet/BTreeSet coercion (i8/i16/u16 → i32) ---
/// Return test: Rust `HashSet<i8>` -> R integer vector (coerced via i32).
#[miniextendr]
pub fn conv_hashset_i8_ret() -> HashSet<i8> {
    vec![1i8, 2i8, -1i8].into_iter().collect()
}

/// Return test: Rust `BTreeSet<i8>` -> R integer vector (coerced via i32, sorted).
#[miniextendr]
pub fn conv_btreeset_i8_ret() -> BTreeSet<i8> {
    vec![1i8, 2i8, -1i8].into_iter().collect()
}

/// Return test: Rust `HashSet<i16>` -> R integer vector (coerced via i32).
#[miniextendr]
pub fn conv_hashset_i16_ret() -> HashSet<i16> {
    vec![1i16, 2i16, -1i16].into_iter().collect()
}

/// Return test: Rust `BTreeSet<i16>` -> R integer vector (coerced via i32, sorted).
#[miniextendr]
pub fn conv_btreeset_i16_ret() -> BTreeSet<i16> {
    vec![1i16, 2i16, -1i16].into_iter().collect()
}

/// Return test: Rust `HashSet<u16>` -> R integer vector (coerced via i32).
#[miniextendr]
pub fn conv_hashset_u16_ret() -> HashSet<u16> {
    vec![1u16, 2u16, 100u16].into_iter().collect()
}

/// Return test: Rust `BTreeSet<u16>` -> R integer vector (coerced via i32, sorted).
#[miniextendr]
pub fn conv_btreeset_u16_ret() -> BTreeSet<u16> {
    vec![1u16, 2u16, 100u16].into_iter().collect()
}

// --- Option<&T> return (copies value, None → NULL) ---
static OPT_REF_VALUE: i32 = 42;

/// Return test: `Some(&42)` as `Option<&i32>` -> R integer.
#[miniextendr]
pub fn conv_opt_ref_i32_some_ret() -> Option<&'static i32> {
    Some(&OPT_REF_VALUE)
}

/// Return test: `None` as `Option<&i32>` -> R NULL.
#[miniextendr]
pub fn conv_opt_ref_i32_none_ret() -> Option<&'static i32> {
    None
}

// --- Option<Vec<T>> (None → NULL, Some → vector) ---
/// Test `Option<Vec<i32>>` argument: returns sum if Some, -999 if None (NULL).
/// @param x Input integer vector or NULL.
#[miniextendr]
pub fn conv_opt_vec_i32_arg(x: Option<Vec<i32>>) -> i32 {
    match x {
        Some(v) => v.iter().sum(),
        None => -999,
    }
}

/// Return test: `Some(vec![1, 2, 3])` as `Option<Vec<i32>>` -> R integer vector.
#[miniextendr]
pub fn conv_opt_vec_i32_some_ret() -> Option<Vec<i32>> {
    Some(vec![1, 2, 3])
}

/// Return test: `None` as `Option<Vec<i32>>` -> R NULL.
#[miniextendr]
pub fn conv_opt_vec_i32_none_ret() -> Option<Vec<i32>> {
    None
}

/// Test `Option<Vec<String>>` argument: returns length if Some, -999 if None (NULL).
/// @param x Input character vector or NULL.
#[miniextendr]
pub fn conv_opt_vec_string_arg(x: Option<Vec<String>>) -> i32 {
    match x {
        Some(v) => v.len() as i32,
        None => -999,
    }
}

/// Return test: `Some(vec!["a", "b"])` as `Option<Vec<String>>` -> R character vector.
#[miniextendr]
pub fn conv_opt_vec_string_some_ret() -> Option<Vec<String>> {
    Some(vec!["a".to_string(), "b".to_string()])
}

/// Return test: `None` as `Option<Vec<String>>` -> R NULL.
#[miniextendr]
pub fn conv_opt_vec_string_none_ret() -> Option<Vec<String>> {
    None
}

// --- Option<HashMap> (None → NULL, Some → named list) ---
/// Test `Option<HashMap<String, i32>>` argument: returns sum if Some, -999 if None.
/// @param x Input named list of integers or NULL.
#[miniextendr]
pub fn conv_opt_hashmap_i32_arg(x: Option<HashMap<String, i32>>) -> i32 {
    match x {
        Some(m) => m.values().sum(),
        None => -999,
    }
}

/// Return test: `Some(HashMap)` as `Option<HashMap<String, i32>>` -> R named list.
#[miniextendr]
pub fn conv_opt_hashmap_i32_some_ret() -> Option<HashMap<String, i32>> {
    let mut m = HashMap::new();
    m.insert("a".to_string(), 1);
    m.insert("b".to_string(), 2);
    Some(m)
}

/// Return test: `None` as `Option<HashMap<String, i32>>` -> R NULL.
#[miniextendr]
pub fn conv_opt_hashmap_i32_none_ret() -> Option<HashMap<String, i32>> {
    None
}

// --- Option<HashSet> (None → NULL, Some → vector) ---
/// Test `Option<HashSet<i32>>` argument: returns sum if Some, -999 if None.
/// @param x Input integer vector or NULL.
#[miniextendr]
pub fn conv_opt_hashset_i32_arg(x: Option<HashSet<i32>>) -> i32 {
    match x {
        Some(s) => s.iter().sum(),
        None => -999,
    }
}

/// Return test: `Some(HashSet)` as `Option<HashSet<i32>>` -> R integer vector.
#[miniextendr]
pub fn conv_opt_hashset_i32_some_ret() -> Option<HashSet<i32>> {
    Some(vec![1, 2, 3].into_iter().collect())
}

/// Return test: `None` as `Option<HashSet<i32>>` -> R NULL.
#[miniextendr]
pub fn conv_opt_hashset_i32_none_ret() -> Option<HashSet<i32>> {
    None
}

// --- Option<BTreeMap> (None → NULL, Some → named list) ---
/// Test `Option<BTreeMap<String, i32>>` argument: returns sum if Some, -999 if None.
/// @param x Input named list of integers or NULL.
#[miniextendr]
pub fn conv_opt_btreemap_i32_arg(x: Option<BTreeMap<String, i32>>) -> i32 {
    match x {
        Some(m) => m.values().sum(),
        None => -999,
    }
}

/// Return test: `Some(BTreeMap)` as `Option<BTreeMap<String, i32>>` -> R named list.
#[miniextendr]
pub fn conv_opt_btreemap_i32_some_ret() -> Option<BTreeMap<String, i32>> {
    let mut m = BTreeMap::new();
    m.insert("a".to_string(), 1);
    m.insert("b".to_string(), 2);
    Some(m)
}

/// Return test: `None` as `Option<BTreeMap<String, i32>>` -> R NULL.
#[miniextendr]
pub fn conv_opt_btreemap_i32_none_ret() -> Option<BTreeMap<String, i32>> {
    None
}

// --- Vec<HashMap> (list of named lists → Vec<HashMap>) ---
/// Test `Vec<HashMap<String, i32>>` argument: returns the sum across all maps.
/// @param x Input list of named integer lists.
#[miniextendr]
pub fn conv_vec_hashmap_i32_arg(x: Vec<HashMap<String, i32>>) -> i32 {
    x.iter().map(|m| m.values().sum::<i32>()).sum()
}

/// Return test: Rust `Vec<HashMap<String, i32>>` -> R list of named lists.
#[miniextendr]
pub fn conv_vec_hashmap_i32_ret() -> Vec<HashMap<String, i32>> {
    vec![
        {
            let mut m = HashMap::new();
            m.insert("a".to_string(), 1);
            m
        },
        {
            let mut m = HashMap::new();
            m.insert("b".to_string(), 2);
            m.insert("c".to_string(), 3);
            m
        },
    ]
}

// --- Vec<BTreeMap> (list of named lists → Vec<BTreeMap>) ---
/// Test `Vec<BTreeMap<String, i32>>` argument: returns the sum across all maps.
/// @param x Input list of named integer lists.
#[miniextendr]
pub fn conv_vec_btreemap_i32_arg(x: Vec<BTreeMap<String, i32>>) -> i32 {
    x.iter().map(|m| m.values().sum::<i32>()).sum()
}

/// Return test: Rust `Vec<BTreeMap<String, i32>>` -> R list of named lists (sorted).
#[miniextendr]
pub fn conv_vec_btreemap_i32_ret() -> Vec<BTreeMap<String, i32>> {
    vec![
        {
            let mut m = BTreeMap::new();
            m.insert("x".to_string(), 10);
            m
        },
        {
            let mut m = BTreeMap::new();
            m.insert("y".to_string(), 20);
            m.insert("z".to_string(), 30);
            m
        },
    ]
}

// --- Vec<Vec<T>> (list of vectors) ---
/// Test `Vec<Vec<i32>>` argument: returns the sum across all inner vectors.
/// @param x Input list of integer vectors.
#[miniextendr]
pub fn conv_vec_vec_i32_arg(x: Vec<Vec<i32>>) -> i32 {
    x.iter().map(|v| v.iter().sum::<i32>()).sum()
}

/// Return test: Rust `Vec<Vec<i32>>` -> R list of integer vectors.
#[miniextendr]
pub fn conv_vec_vec_i32_ret() -> Vec<Vec<i32>> {
    vec![vec![1, 2], vec![3, 4, 5]]
}

/// Return test: Rust `Vec<Vec<String>>` -> R list of character vectors.
#[miniextendr]
pub fn conv_vec_vec_string_ret() -> Vec<Vec<String>> {
    vec![
        vec!["a".to_string(), "b".to_string()],
        vec!["c".to_string()],
    ]
}

// --- Vec<Option<T>> for extended numeric types ---

/// Return test: Rust `Vec<Option<i64>>` with small values -> R integer vector with NA.
#[miniextendr]
pub fn conv_vec_option_i64_ret_small() -> Vec<Option<i64>> {
    vec![Some(1), None, Some(3)]
}

/// Return test: Rust `Vec<Option<i64>>` with large values -> R numeric vector with NA.
#[miniextendr]
pub fn conv_vec_option_i64_ret_big() -> Vec<Option<i64>> {
    vec![Some(i64::MAX), None, Some(1)]
}

/// Return test: Rust `Vec<Option<u32>>` -> R integer vector with NA.
#[miniextendr]
pub fn conv_vec_option_u32_ret() -> Vec<Option<u32>> {
    vec![Some(1), None, Some(42)]
}

/// Return test: Rust `Vec<Option<f32>>` -> R double vector with NA.
#[miniextendr]
pub fn conv_vec_option_f32_ret() -> Vec<Option<f32>> {
    vec![Some(1.5), None, Some(3.0)]
}

/// Roundtrip test: R numeric vector with NA -> Rust `Vec<Option<i64>>` -> R.
/// @param x Input numeric vector (may contain NA).
#[miniextendr]
pub fn conv_vec_option_i64_roundtrip(x: Vec<Option<i64>>) -> Vec<Option<i64>> {
    x
}

/// Return test: Rust `Vec<Option<u64>>` with small values -> R integer vector with NA.
#[miniextendr]
pub fn conv_vec_option_u64_ret_small() -> Vec<Option<u64>> {
    vec![Some(1), None, Some(3)]
}

/// Return test: Rust `Vec<Option<u64>>` with large values -> R numeric vector with NA.
#[miniextendr]
pub fn conv_vec_option_u64_ret_big() -> Vec<Option<u64>> {
    vec![Some(u64::MAX), None, Some(1)]
}

/// Return test: Rust `Vec<Option<isize>>` with small values -> R integer vector with NA.
#[miniextendr]
pub fn conv_vec_option_isize_ret_small() -> Vec<Option<isize>> {
    vec![Some(1), None, Some(3)]
}

/// Return test: Rust `Vec<Option<usize>>` with small values -> R integer vector with NA.
#[miniextendr]
pub fn conv_vec_option_usize_ret_small() -> Vec<Option<usize>> {
    vec![Some(1), None, Some(3)]
}

/// Return test: Rust `Vec<Option<i8>>` -> R integer vector with NA.
#[miniextendr]
pub fn conv_vec_option_i8_ret() -> Vec<Option<i8>> {
    vec![Some(1), None, Some(-1)]
}

/// Return test: Rust `Vec<Option<i16>>` -> R integer vector with NA.
#[miniextendr]
pub fn conv_vec_option_i16_ret() -> Vec<Option<i16>> {
    vec![Some(1), None, Some(-1)]
}

/// Return test: Rust `Vec<Option<u16>>` -> R integer vector with NA.
#[miniextendr]
pub fn conv_vec_option_u16_ret() -> Vec<Option<u16>> {
    vec![Some(1), None, Some(100)]
}

// --- Scalar Option<T> for extended numeric types ---

/// Return test: `Some(42)` as `Option<i64>` -> R integer (fits in i32).
#[miniextendr]
pub fn conv_option_i64_some_small() -> Option<i64> {
    Some(42)
}

/// Return test: `Some(i64::MAX)` as `Option<i64>` -> R double (too large for i32).
#[miniextendr]
pub fn conv_option_i64_some_big() -> Option<i64> {
    Some(i64::MAX)
}

/// Return test: `None` as `Option<i64>` -> R NA.
#[miniextendr]
pub fn conv_option_i64_none() -> Option<i64> {
    None
}

/// Return test: `Some(1.5)` as `Option<f32>` -> R double.
#[miniextendr]
pub fn conv_option_f32_some() -> Option<f32> {
    Some(1.5)
}

/// Return test: `Some(100)` as `Option<u32>` -> R integer.
#[miniextendr]
pub fn conv_option_u32_some() -> Option<u32> {
    Some(100)
}
// endregion

// region: Named pair wrappers (AsNamedList / AsNamedVector)

use miniextendr_api::{AsNamedList, AsNamedListExt, AsNamedVector, AsNamedVectorExt};

/// Return test: `AsNamedList<Vec<(String, i32)>>` -> R named list.
#[miniextendr]
pub fn conv_as_named_list_vec() -> AsNamedList<Vec<(String, i32)>> {
    AsNamedList(vec![
        ("width".into(), 100),
        ("height".into(), 200),
        ("depth".into(), 300),
    ])
}

/// Return test: `AsNamedList<[(String, f64); 2]>` -> R named list from fixed-size array.
#[miniextendr]
pub fn conv_as_named_list_array() -> AsNamedList<[(String, f64); 2]> {
    AsNamedList([
        ("pi".into(), std::f64::consts::PI),
        ("e".into(), std::f64::consts::E),
    ])
}

/// Return test: `AsNamedList` with heterogeneous SEXP values -> R named list.
#[miniextendr]
pub fn conv_as_named_list_heterogeneous() -> AsNamedList<Vec<(String, miniextendr_api::ffi::SEXP)>>
{
    use miniextendr_api::IntoR;
    AsNamedList(vec![
        ("name".into(), "Alice".to_string().into_sexp()),
        ("age".into(), 30i32.into_sexp()),
        ("score".into(), 95.5f64.into_sexp()),
    ])
}

/// Return test: `AsNamedList<Vec<(&str, i32)>>` -> R named list with `&str` keys.
#[miniextendr]
pub fn conv_as_named_list_str_keys() -> AsNamedList<Vec<(&'static str, i32)>> {
    AsNamedList(vec![("a", 1), ("b", 2)])
}

/// Return test: empty `AsNamedList` -> R empty named list.
#[miniextendr]
pub fn conv_as_named_list_empty() -> AsNamedList<Vec<(String, i32)>> {
    AsNamedList(vec![])
}

/// Return test: `AsNamedList` with duplicate key names -> R named list (duplicates preserved).
#[miniextendr]
pub fn conv_as_named_list_duplicate_names() -> AsNamedList<Vec<(&'static str, i32)>> {
    AsNamedList(vec![("x", 1), ("x", 2), ("x", 3)])
}

/// Return test: `AsNamedVector<Vec<(String, i32)>>` -> R named integer vector.
#[miniextendr]
pub fn conv_as_named_vector_i32() -> AsNamedVector<Vec<(String, i32)>> {
    AsNamedVector(vec![
        ("alice".into(), 95),
        ("bob".into(), 87),
        ("carol".into(), 92),
    ])
}

/// Return test: `AsNamedVector<Vec<(&str, f64)>>` -> R named double vector.
#[miniextendr]
pub fn conv_as_named_vector_f64() -> AsNamedVector<Vec<(&'static str, f64)>> {
    AsNamedVector(vec![
        ("pi", std::f64::consts::PI),
        ("e", std::f64::consts::E),
    ])
}

/// Return test: `AsNamedVector<Vec<(String, String)>>` -> R named character vector.
#[miniextendr]
pub fn conv_as_named_vector_string() -> AsNamedVector<Vec<(String, String)>> {
    AsNamedVector(vec![
        ("greeting".into(), "hello".into()),
        ("farewell".into(), "goodbye".into()),
    ])
}

/// Return test: `AsNamedVector<Vec<(&str, Option<i32>)>>` -> R named integer vector with NA.
#[miniextendr]
pub fn conv_as_named_vector_option_i32() -> AsNamedVector<Vec<(&'static str, Option<i32>)>> {
    AsNamedVector(vec![("a", Some(1)), ("b", None), ("c", Some(3))])
}

/// Return test: `AsNamedVector` from fixed-size array -> R named double vector.
#[miniextendr]
pub fn conv_as_named_vector_array() -> AsNamedVector<[(&'static str, f64); 3]> {
    AsNamedVector([("x", 1.0), ("y", 2.0), ("z", 3.0)])
}

/// Return test: empty `AsNamedVector` -> R empty named double vector.
#[miniextendr]
pub fn conv_as_named_vector_empty() -> AsNamedVector<Vec<(String, f64)>> {
    AsNamedVector(vec![])
}

/// Return test: `AsNamedVectorExt` trait method -> R named integer vector.
#[miniextendr]
pub fn conv_as_named_vector_ext_trait() -> AsNamedVector<Vec<(&'static str, i32)>> {
    vec![("one", 1), ("two", 2)].as_named_vector()
}

/// Return test: `AsNamedListExt` trait method -> R named list.
#[miniextendr]
pub fn conv_as_named_list_ext_trait() -> AsNamedList<Vec<(&'static str, i32)>> {
    vec![("one", 1), ("two", 2)].as_named_list()
}

/// Return test: `AsNamedList` from a borrowed slice -> R named list via SEXP.
#[miniextendr]
pub fn conv_as_named_list_slice() -> miniextendr_api::ffi::SEXP {
    let pairs: &[(&str, i32)] = &[("x", 10), ("y", 20), ("z", 30)];
    AsNamedList(pairs).into_sexp()
}

/// Return test: `AsNamedVector` from a borrowed slice -> R named double vector via SEXP.
#[miniextendr]
pub fn conv_as_named_vector_slice() -> miniextendr_api::ffi::SEXP {
    let pairs: &[(&str, f64)] = &[("a", 1.5), ("b", 2.5)];
    AsNamedVector(pairs).into_sexp()
}

// NamedList (O(1) lookup wrapper)

use miniextendr_api::NamedList;

/// Test `NamedList` O(1) lookup: retrieves "width" and "height" and returns their product.
/// @param config Input named list with "width" and "height" entries.
#[miniextendr]
pub fn conv_named_list_get(config: NamedList) -> i32 {
    let width: i32 = config.get("width").unwrap_or(0);
    let height: i32 = config.get("height").unwrap_or(0);
    width * height
}

/// Test `NamedList::contains`: checks existence of "a", "b", and "missing".
/// @param config Input named list.
#[miniextendr]
pub fn conv_named_list_contains(config: NamedList) -> Vec<bool> {
    vec![
        config.contains("a"),
        config.contains("b"),
        config.contains("missing"),
    ]
}

/// Test `NamedList` length queries: returns total length and named-element count.
/// @param config Input named list.
#[miniextendr]
pub fn conv_named_list_len(config: NamedList) -> Vec<i32> {
    vec![config.len() as i32, config.named_len() as i32]
}

/// Test `NamedList` roundtrip: converts to `List` and back to R.
/// @param config Input named list.
#[miniextendr]
pub fn conv_named_list_roundtrip(config: NamedList) -> miniextendr_api::list::List {
    config.into_list()
}
// endregion
