//! Comprehensive conversions matrix for [`#[miniextendr]`](miniextendr_api::miniextendr) arguments and returns.

use miniextendr_api::ffi::{RLogical, Rboolean, SEXP};
use miniextendr_api::{IntoR, ListMut, miniextendr, miniextendr_module};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_i32_arg(x: i32) -> i32 {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_i32_ret() -> i32 {
    1
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_f64_arg(x: f64) -> f64 {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_f64_ret() -> f64 {
    1.25
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_u8_arg(x: u8) -> u8 {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_u8_ret() -> u8 {
    7u8
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_rbool_arg(x: Rboolean) -> Rboolean {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_rbool_ret() -> Rboolean {
    Rboolean::TRUE
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_rlog_arg(x: RLogical) -> RLogical {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_rlog_ret() -> RLogical {
    RLogical::TRUE
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_string_arg(x: String) -> String {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_string_ret() -> String {
    "hi".to_string()
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_str_ret() -> &'static str {
    "hi"
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_sexp_arg(x: SEXP) -> SEXP {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_sexp_ret() -> SEXP {
    1i32.into_sexp()
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_i64_arg(x: i64) -> i64 {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_i64_ret() -> i64 {
    1i64
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_u64_arg(x: u64) -> u64 {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_u64_ret() -> u64 {
    1u64
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_isize_arg(x: isize) -> isize {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_isize_ret() -> isize {
    1isize
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_usize_arg(x: usize) -> usize {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_usize_ret() -> usize {
    1usize
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_i8_arg(x: i8) -> i8 {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_i8_ret() -> i8 {
    1i8
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_i16_arg(x: i16) -> i16 {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_i16_ret() -> i16 {
    1i16
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_u16_arg(x: u16) -> u16 {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_u16_ret() -> u16 {
    1u16
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_u32_arg(x: u32) -> u32 {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_u32_ret() -> u32 {
    1u32
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_f32_arg(x: f32) -> f32 {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_f32_ret() -> f32 {
    1.5f32
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_opt_i32_is_some(x: Option<i32>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

/// @noRd
#[miniextendr]
pub fn conv_opt_i32_some() -> Option<i32> {
    Some(10)
}

/// @noRd
#[miniextendr]
pub fn conv_opt_i32_none() -> Option<i32> {
    None
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_opt_f64_is_some(x: Option<f64>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

/// @noRd
#[miniextendr]
pub fn conv_opt_f64_some() -> Option<f64> {
    Some(2.5)
}

/// @noRd
#[miniextendr]
pub fn conv_opt_f64_none() -> Option<f64> {
    None
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_opt_bool_is_some(x: Option<bool>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

/// @noRd
#[miniextendr]
pub fn conv_opt_bool_some() -> Option<bool> {
    Some(true)
}

/// @noRd
#[miniextendr]
pub fn conv_opt_bool_none() -> Option<bool> {
    None
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_opt_string_is_some(x: Option<String>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

/// @noRd
#[miniextendr]
pub fn conv_opt_string_some() -> Option<String> {
    Some("opt".to_string())
}

/// @noRd
#[miniextendr]
pub fn conv_opt_string_none() -> Option<String> {
    None
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_opt_i8_is_some(x: Option<i8>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_opt_i16_is_some(x: Option<i16>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_opt_i64_is_some(x: Option<i64>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_opt_isize_is_some(x: Option<isize>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_opt_u16_is_some(x: Option<u16>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_opt_u32_is_some(x: Option<u32>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_opt_u64_is_some(x: Option<u64>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_opt_usize_is_some(x: Option<usize>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_opt_f32_is_some(x: Option<f32>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_opt_u8_is_some(x: Option<u8>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_opt_rbool_is_some(x: Option<Rboolean>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_opt_rlog_is_some(x: Option<RLogical>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_slice_i32_len(x: &'static [i32]) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_slice_f64_len(x: &'static [f64]) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_slice_u8_len(x: &'static [u8]) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_slice_rlog_len(x: &'static [RLogical]) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_ref_i32_arg(x: &'static i32) -> i32 {
    *x
}

/// @noRd
#[miniextendr]
pub fn conv_ref_mut_i32_add_one(x: &'static mut i32) -> i32 {
    *x += 1;
    *x
}

/// @noRd
#[miniextendr]
pub fn conv_slice_mut_i32_add_one(x: &'static mut [i32]) -> i32 {
    for v in x.iter_mut() {
        *v += 1;
    }
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_slice_mut_u8_add_one(x: &'static mut [u8]) -> i32 {
    for v in x.iter_mut() {
        *v = v.wrapping_add(1);
    }
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_opt_ref_i32_is_some(x: Option<&'static i32>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

/// @noRd
#[miniextendr]
pub fn conv_opt_mut_slice_i32_is_some(x: Option<&'static mut [i32]>) -> i32 {
    if x.is_some() { 1 } else { 0 }
}

/// @noRd
#[miniextendr]
pub fn conv_vec_ref_i32_len(x: Vec<&'static i32>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_vec_slice_i32_total_len(x: Vec<&'static [i32]>) -> i32 {
    x.iter().map(|s| s.len() as i32).sum()
}

/// @noRd
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
/// @noRd
#[miniextendr]
pub fn conv_vec_i32_len(x: Vec<i32>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_vec_i32_ret() -> Vec<i32> {
    vec![1, 2, 3]
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_f64_len(x: Vec<f64>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_vec_f64_ret() -> Vec<f64> {
    vec![1.0, 2.0, 3.0]
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_u8_len(x: Vec<u8>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_vec_u8_ret() -> Vec<u8> {
    vec![1u8, 2u8, 3u8]
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_rlog_len(x: Vec<RLogical>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_vec_rlog_ret() -> Vec<RLogical> {
    vec![RLogical::TRUE, RLogical::FALSE]
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_bool_len(x: Vec<bool>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_vec_bool_ret() -> Vec<bool> {
    vec![true, false, true]
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_string_len(x: Vec<String>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_vec_string_ret() -> Vec<String> {
    vec!["a".to_string(), "b".to_string()]
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_i8_len(x: Vec<i8>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_i16_len(x: Vec<i16>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_i64_len(x: Vec<i64>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_isize_len(x: Vec<isize>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_u16_len(x: Vec<u16>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_u32_len(x: Vec<u32>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_u64_len(x: Vec<u64>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_usize_len(x: Vec<usize>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_f32_len(x: Vec<f32>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_opt_i32_len(x: Vec<Option<i32>>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_vec_opt_i32_ret() -> Vec<Option<i32>> {
    vec![Some(1), None, Some(3)]
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_opt_f64_len(x: Vec<Option<f64>>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_vec_opt_f64_ret() -> Vec<Option<f64>> {
    vec![Some(1.0), None, Some(3.0)]
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_opt_bool_len(x: Vec<Option<bool>>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_vec_opt_bool_ret() -> Vec<Option<bool>> {
    vec![Some(true), None, Some(false)]
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_opt_string_len(x: Vec<Option<String>>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_vec_opt_string_ret() -> Vec<Option<String>> {
    vec![Some("a".to_string()), None, Some("b".to_string())]
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_opt_rlog_ret() -> Vec<Option<RLogical>> {
    vec![Some(RLogical::TRUE), None, Some(RLogical::FALSE)]
}

/// @noRd
#[miniextendr]
pub fn conv_vec_opt_rbool_ret() -> Vec<Option<Rboolean>> {
    vec![Some(Rboolean::TRUE), None, Some(Rboolean::FALSE)]
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_vec_opt_i8_len(x: Vec<Option<i8>>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_vec_opt_u8_len(x: Vec<Option<u8>>) -> i32 {
    x.len() as i32
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_hashset_i32_len(x: HashSet<i32>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_hashset_i32_ret() -> HashSet<i32> {
    vec![1, 2, 3].into_iter().collect()
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_hashset_u8_len(x: HashSet<u8>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_hashset_u8_ret() -> HashSet<u8> {
    vec![1u8, 2u8, 3u8].into_iter().collect()
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_hashset_string_len(x: HashSet<String>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_hashset_string_ret() -> HashSet<String> {
    vec!["a".to_string(), "b".to_string()].into_iter().collect()
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_hashset_rlog_len(x: HashSet<RLogical>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_hashset_rlog_ret() -> HashSet<RLogical> {
    vec![RLogical::TRUE, RLogical::FALSE].into_iter().collect()
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_btreeset_i32_len(x: BTreeSet<i32>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_btreeset_i32_ret() -> BTreeSet<i32> {
    vec![1, 2, 3].into_iter().collect()
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_btreeset_u8_len(x: BTreeSet<u8>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_btreeset_u8_ret() -> BTreeSet<u8> {
    vec![1u8, 2u8, 3u8].into_iter().collect()
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_btreeset_string_len(x: BTreeSet<String>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_btreeset_string_ret() -> BTreeSet<String> {
    vec!["a".to_string(), "b".to_string()].into_iter().collect()
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_hashmap_i32_len(x: HashMap<String, i32>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_hashmap_i32_ret() -> HashMap<String, i32> {
    vec![("a".to_string(), 1), ("b".to_string(), 2)]
        .into_iter()
        .collect()
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_hashmap_f64_len(x: HashMap<String, f64>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_hashmap_f64_ret() -> HashMap<String, f64> {
    vec![("a".to_string(), 1.5), ("b".to_string(), 2.5)]
        .into_iter()
        .collect()
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_hashmap_string_len(x: HashMap<String, String>) -> i32 {
    x.len() as i32
}

/// @noRd
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
/// @noRd
#[miniextendr]
pub fn conv_hashmap_rlog_len(x: HashMap<String, RLogical>) -> i32 {
    x.len() as i32
}

/// @noRd
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
/// @noRd
#[miniextendr]
pub fn conv_btreemap_i32_len(x: BTreeMap<String, i32>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_btreemap_i32_ret() -> BTreeMap<String, i32> {
    vec![("a".to_string(), 1), ("b".to_string(), 2)]
        .into_iter()
        .collect()
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_btreemap_f64_len(x: BTreeMap<String, f64>) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn conv_btreemap_f64_ret() -> BTreeMap<String, f64> {
    vec![("a".to_string(), 1.5), ("b".to_string(), 2.5)]
        .into_iter()
        .collect()
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_btreemap_string_len(x: BTreeMap<String, String>) -> i32 {
    x.len() as i32
}

/// @noRd
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
/// @noRd
#[miniextendr]
pub fn conv_btreemap_rlog_len(x: BTreeMap<String, RLogical>) -> i32 {
    x.len() as i32
}

/// @noRd
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
/// @noRd
#[miniextendr]
pub fn conv_list_mut_set_first(mut x: ListMut) -> i32 {
    let len = x.len() as i32;
    if len > 0 {
        let _ = x.set(0, 99i32.into_sexp());
    }
    len
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_result_i32_arg(x: Result<i32, ()>) -> i32 {
    if x.is_ok() { 1 } else { 0 }
}

/// @noRd
#[miniextendr]
pub fn conv_result_vec_i32_arg(x: Result<Vec<i32>, ()>) -> i32 {
    if x.is_ok() { 1 } else { 0 }
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_result_i32_ok() -> Result<i32, ()> {
    Ok(9)
}

/// @noRd
#[miniextendr]
pub fn conv_result_i32_err() -> Result<i32, ()> {
    Err(())
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_result_f64_ok() -> Result<f64, ()> {
    Ok(9.5)
}

/// @noRd
#[miniextendr]
pub fn conv_result_f64_err() -> Result<f64, ()> {
    Err(())
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_result_string_ok() -> Result<String, ()> {
    Ok("ok".to_string())
}

/// @noRd
#[miniextendr]
pub fn conv_result_string_err() -> Result<String, ()> {
    Err(())
}

// -----------------------------------------------------------------------------
/// @noRd
#[miniextendr]
pub fn conv_result_vec_i32_ok() -> Result<Vec<i32>, ()> {
    Ok(vec![1, 2])
}

/// @noRd
#[miniextendr]
pub fn conv_result_vec_i32_err() -> Result<Vec<i32>, ()> {
    Err(())
}

// =============================================================================
// Extended conversions - nested types, coercion, char
// =============================================================================

// --- char conversions (char ↔ length-1 string) ---
/// @noRd
#[miniextendr]
pub fn conv_char_arg(x: char) -> char {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_char_ret() -> char {
    'α' // Unicode char to test UTF-8
}

// --- Vec coercion (i8/i16/u16 → i32, f32 → f64) ---
/// @noRd
#[miniextendr]
pub fn conv_vec_i8_ret() -> Vec<i8> {
    vec![1i8, -1i8, 127i8]
}

/// @noRd
#[miniextendr]
pub fn conv_vec_i16_ret() -> Vec<i16> {
    vec![1i16, -1i16, 32767i16]
}

/// @noRd
#[miniextendr]
pub fn conv_vec_u16_ret() -> Vec<u16> {
    vec![1u16, 100u16, 65535u16]
}

/// @noRd
#[miniextendr]
pub fn conv_vec_f32_ret() -> Vec<f32> {
    vec![1.5f32, 2.5f32, -0.5f32]
}

// --- HashSet/BTreeSet coercion (i8/i16/u16 → i32) ---
/// @noRd
#[miniextendr]
pub fn conv_hashset_i8_ret() -> HashSet<i8> {
    vec![1i8, 2i8, -1i8].into_iter().collect()
}

/// @noRd
#[miniextendr]
pub fn conv_btreeset_i8_ret() -> BTreeSet<i8> {
    vec![1i8, 2i8, -1i8].into_iter().collect()
}

/// @noRd
#[miniextendr]
pub fn conv_hashset_i16_ret() -> HashSet<i16> {
    vec![1i16, 2i16, -1i16].into_iter().collect()
}

/// @noRd
#[miniextendr]
pub fn conv_btreeset_i16_ret() -> BTreeSet<i16> {
    vec![1i16, 2i16, -1i16].into_iter().collect()
}

/// @noRd
#[miniextendr]
pub fn conv_hashset_u16_ret() -> HashSet<u16> {
    vec![1u16, 2u16, 100u16].into_iter().collect()
}

/// @noRd
#[miniextendr]
pub fn conv_btreeset_u16_ret() -> BTreeSet<u16> {
    vec![1u16, 2u16, 100u16].into_iter().collect()
}

// --- Option<&T> return (copies value, None → NULL) ---
static OPT_REF_VALUE: i32 = 42;

/// @noRd
#[miniextendr]
pub fn conv_opt_ref_i32_some_ret() -> Option<&'static i32> {
    Some(&OPT_REF_VALUE)
}

/// @noRd
#[miniextendr]
pub fn conv_opt_ref_i32_none_ret() -> Option<&'static i32> {
    None
}

// --- Option<Vec<T>> (None → NULL, Some → vector) ---
/// @noRd
#[miniextendr]
pub fn conv_opt_vec_i32_arg(x: Option<Vec<i32>>) -> i32 {
    match x {
        Some(v) => v.iter().sum(),
        None => -999,
    }
}

/// @noRd
#[miniextendr]
pub fn conv_opt_vec_i32_some_ret() -> Option<Vec<i32>> {
    Some(vec![1, 2, 3])
}

/// @noRd
#[miniextendr]
pub fn conv_opt_vec_i32_none_ret() -> Option<Vec<i32>> {
    None
}

/// @noRd
#[miniextendr]
pub fn conv_opt_vec_string_arg(x: Option<Vec<String>>) -> i32 {
    match x {
        Some(v) => v.len() as i32,
        None => -999,
    }
}

/// @noRd
#[miniextendr]
pub fn conv_opt_vec_string_some_ret() -> Option<Vec<String>> {
    Some(vec!["a".to_string(), "b".to_string()])
}

/// @noRd
#[miniextendr]
pub fn conv_opt_vec_string_none_ret() -> Option<Vec<String>> {
    None
}

// --- Option<HashMap> (None → NULL, Some → named list) ---
/// @noRd
#[miniextendr]
pub fn conv_opt_hashmap_i32_arg(x: Option<HashMap<String, i32>>) -> i32 {
    match x {
        Some(m) => m.values().sum(),
        None => -999,
    }
}

/// @noRd
#[miniextendr]
pub fn conv_opt_hashmap_i32_some_ret() -> Option<HashMap<String, i32>> {
    let mut m = HashMap::new();
    m.insert("a".to_string(), 1);
    m.insert("b".to_string(), 2);
    Some(m)
}

/// @noRd
#[miniextendr]
pub fn conv_opt_hashmap_i32_none_ret() -> Option<HashMap<String, i32>> {
    None
}

// --- Option<HashSet> (None → NULL, Some → vector) ---
/// @noRd
#[miniextendr]
pub fn conv_opt_hashset_i32_arg(x: Option<HashSet<i32>>) -> i32 {
    match x {
        Some(s) => s.iter().sum(),
        None => -999,
    }
}

/// @noRd
#[miniextendr]
pub fn conv_opt_hashset_i32_some_ret() -> Option<HashSet<i32>> {
    Some(vec![1, 2, 3].into_iter().collect())
}

/// @noRd
#[miniextendr]
pub fn conv_opt_hashset_i32_none_ret() -> Option<HashSet<i32>> {
    None
}

// --- Vec<Vec<T>> (list of vectors) ---
/// @noRd
#[miniextendr]
pub fn conv_vec_vec_i32_arg(x: Vec<Vec<i32>>) -> i32 {
    x.iter().map(|v| v.iter().sum::<i32>()).sum()
}

/// @noRd
#[miniextendr]
pub fn conv_vec_vec_i32_ret() -> Vec<Vec<i32>> {
    vec![vec![1, 2], vec![3, 4, 5]]
}

/// @noRd
#[miniextendr]
pub fn conv_vec_vec_string_ret() -> Vec<Vec<String>> {
    vec![
        vec!["a".to_string(), "b".to_string()],
        vec!["c".to_string()],
    ]
}

// --- Vec<Option<T>> for extended numeric types ---

/// @noRd
#[miniextendr]
pub fn conv_vec_option_i64_ret_small() -> Vec<Option<i64>> {
    vec![Some(1), None, Some(3)]
}

/// @noRd
#[miniextendr]
pub fn conv_vec_option_i64_ret_big() -> Vec<Option<i64>> {
    vec![Some(i64::MAX), None, Some(1)]
}

/// @noRd
#[miniextendr]
pub fn conv_vec_option_u32_ret() -> Vec<Option<u32>> {
    vec![Some(1), None, Some(42)]
}

/// @noRd
#[miniextendr]
pub fn conv_vec_option_f32_ret() -> Vec<Option<f32>> {
    vec![Some(1.5), None, Some(3.0)]
}

/// @noRd
#[miniextendr]
pub fn conv_vec_option_i64_roundtrip(x: Vec<Option<i64>>) -> Vec<Option<i64>> {
    x
}

/// @noRd
#[miniextendr]
pub fn conv_vec_option_u64_ret_small() -> Vec<Option<u64>> {
    vec![Some(1), None, Some(3)]
}

/// @noRd
#[miniextendr]
pub fn conv_vec_option_u64_ret_big() -> Vec<Option<u64>> {
    vec![Some(u64::MAX), None, Some(1)]
}

/// @noRd
#[miniextendr]
pub fn conv_vec_option_isize_ret_small() -> Vec<Option<isize>> {
    vec![Some(1), None, Some(3)]
}

/// @noRd
#[miniextendr]
pub fn conv_vec_option_usize_ret_small() -> Vec<Option<usize>> {
    vec![Some(1), None, Some(3)]
}

/// @noRd
#[miniextendr]
pub fn conv_vec_option_i8_ret() -> Vec<Option<i8>> {
    vec![Some(1), None, Some(-1)]
}

/// @noRd
#[miniextendr]
pub fn conv_vec_option_i16_ret() -> Vec<Option<i16>> {
    vec![Some(1), None, Some(-1)]
}

/// @noRd
#[miniextendr]
pub fn conv_vec_option_u16_ret() -> Vec<Option<u16>> {
    vec![Some(1), None, Some(100)]
}

// --- Scalar Option<T> for extended numeric types ---

/// @noRd
#[miniextendr]
pub fn conv_option_i64_some_small() -> Option<i64> {
    Some(42)
}

/// @noRd
#[miniextendr]
pub fn conv_option_i64_some_big() -> Option<i64> {
    Some(i64::MAX)
}

/// @noRd
#[miniextendr]
pub fn conv_option_i64_none() -> Option<i64> {
    None
}

/// @noRd
#[miniextendr]
pub fn conv_option_f32_some() -> Option<f32> {
    Some(1.5)
}

/// @noRd
#[miniextendr]
pub fn conv_option_u32_some() -> Option<u32> {
    Some(100)
}

// =============================================================================
// Named pair wrappers (AsNamedList / AsNamedVector)
// =============================================================================

use miniextendr_api::{AsNamedList, AsNamedListExt, AsNamedVector, AsNamedVectorExt};

/// @noRd
#[miniextendr]
pub fn conv_as_named_list_vec() -> AsNamedList<Vec<(String, i32)>> {
    AsNamedList(vec![
        ("width".into(), 100),
        ("height".into(), 200),
        ("depth".into(), 300),
    ])
}

/// @noRd
#[miniextendr]
pub fn conv_as_named_list_array() -> AsNamedList<[(String, f64); 2]> {
    AsNamedList([("pi".into(), std::f64::consts::PI), ("e".into(), std::f64::consts::E)])
}

/// @noRd
#[miniextendr]
pub fn conv_as_named_list_heterogeneous() -> AsNamedList<Vec<(String, miniextendr_api::ffi::SEXP)>> {
    use miniextendr_api::IntoR;
    AsNamedList(vec![
        ("name".into(), "Alice".to_string().into_sexp()),
        ("age".into(), 30i32.into_sexp()),
        ("score".into(), 95.5f64.into_sexp()),
    ])
}

/// @noRd
#[miniextendr]
pub fn conv_as_named_list_str_keys() -> AsNamedList<Vec<(&'static str, i32)>> {
    AsNamedList(vec![("a", 1), ("b", 2)])
}

/// @noRd
#[miniextendr]
pub fn conv_as_named_list_empty() -> AsNamedList<Vec<(String, i32)>> {
    AsNamedList(vec![])
}

/// @noRd
#[miniextendr]
pub fn conv_as_named_list_duplicate_names() -> AsNamedList<Vec<(&'static str, i32)>> {
    AsNamedList(vec![("x", 1), ("x", 2), ("x", 3)])
}

/// @noRd
#[miniextendr]
pub fn conv_as_named_vector_i32() -> AsNamedVector<Vec<(String, i32)>> {
    AsNamedVector(vec![
        ("alice".into(), 95),
        ("bob".into(), 87),
        ("carol".into(), 92),
    ])
}

/// @noRd
#[miniextendr]
pub fn conv_as_named_vector_f64() -> AsNamedVector<Vec<(&'static str, f64)>> {
    AsNamedVector(vec![("pi", std::f64::consts::PI), ("e", std::f64::consts::E)])
}

/// @noRd
#[miniextendr]
pub fn conv_as_named_vector_string() -> AsNamedVector<Vec<(String, String)>> {
    AsNamedVector(vec![
        ("greeting".into(), "hello".into()),
        ("farewell".into(), "goodbye".into()),
    ])
}

/// @noRd
#[miniextendr]
pub fn conv_as_named_vector_option_i32() -> AsNamedVector<Vec<(&'static str, Option<i32>)>> {
    AsNamedVector(vec![("a", Some(1)), ("b", None), ("c", Some(3))])
}

/// @noRd
#[miniextendr]
pub fn conv_as_named_vector_array() -> AsNamedVector<[(&'static str, f64); 3]> {
    AsNamedVector([("x", 1.0), ("y", 2.0), ("z", 3.0)])
}

/// @noRd
#[miniextendr]
pub fn conv_as_named_vector_empty() -> AsNamedVector<Vec<(String, f64)>> {
    AsNamedVector(vec![])
}

/// @noRd
#[miniextendr]
pub fn conv_as_named_vector_ext_trait() -> AsNamedVector<Vec<(&'static str, i32)>> {
    vec![("one", 1), ("two", 2)].as_named_vector()
}

/// @noRd
#[miniextendr]
pub fn conv_as_named_list_ext_trait() -> AsNamedList<Vec<(&'static str, i32)>> {
    vec![("one", 1), ("two", 2)].as_named_list()
}

/// @noRd
#[miniextendr]
pub fn conv_as_named_list_slice() -> miniextendr_api::ffi::SEXP {
    let pairs: &[(&str, i32)] = &[("x", 10), ("y", 20), ("z", 30)];
    AsNamedList(pairs).into_sexp()
}

/// @noRd
#[miniextendr]
pub fn conv_as_named_vector_slice() -> miniextendr_api::ffi::SEXP {
    let pairs: &[(&str, f64)] = &[("a", 1.5), ("b", 2.5)];
    AsNamedVector(pairs).into_sexp()
}

miniextendr_module! {
    mod conversions;

    fn conv_i32_arg;
    fn conv_i32_ret;
    fn conv_f64_arg;
    fn conv_f64_ret;
    fn conv_u8_arg;
    fn conv_u8_ret;
    fn conv_rbool_arg;
    fn conv_rbool_ret;
    fn conv_rlog_arg;
    fn conv_rlog_ret;
    fn conv_string_arg;
    fn conv_string_ret;
    fn conv_str_ret;
    fn conv_sexp_arg;
    fn conv_sexp_ret;
    fn conv_i64_arg;
    fn conv_i64_ret;
    fn conv_u64_arg;
    fn conv_u64_ret;
    fn conv_isize_arg;
    fn conv_isize_ret;
    fn conv_usize_arg;
    fn conv_usize_ret;
    fn conv_i8_arg;
    fn conv_i8_ret;
    fn conv_i16_arg;
    fn conv_i16_ret;
    fn conv_u16_arg;
    fn conv_u16_ret;
    fn conv_u32_arg;
    fn conv_u32_ret;
    fn conv_f32_arg;
    fn conv_f32_ret;
    fn conv_opt_i32_is_some;
    fn conv_opt_i32_some;
    fn conv_opt_i32_none;
    fn conv_opt_f64_is_some;
    fn conv_opt_f64_some;
    fn conv_opt_f64_none;
    fn conv_opt_bool_is_some;
    fn conv_opt_bool_some;
    fn conv_opt_bool_none;
    fn conv_opt_string_is_some;
    fn conv_opt_string_some;
    fn conv_opt_string_none;
    fn conv_opt_i8_is_some;
    fn conv_opt_i16_is_some;
    fn conv_opt_i64_is_some;
    fn conv_opt_isize_is_some;
    fn conv_opt_u16_is_some;
    fn conv_opt_u32_is_some;
    fn conv_opt_u64_is_some;
    fn conv_opt_usize_is_some;
    fn conv_opt_f32_is_some;
    fn conv_opt_u8_is_some;
    fn conv_opt_rbool_is_some;
    fn conv_opt_rlog_is_some;
    fn conv_slice_i32_len;
    fn conv_slice_f64_len;
    fn conv_slice_u8_len;
    fn conv_slice_rlog_len;
    fn conv_ref_i32_arg;
    fn conv_ref_mut_i32_add_one;
    fn conv_slice_mut_i32_add_one;
    fn conv_slice_mut_u8_add_one;
    fn conv_opt_ref_i32_is_some;
    fn conv_opt_mut_slice_i32_is_some;
    fn conv_vec_ref_i32_len;
    fn conv_vec_slice_i32_total_len;
    fn conv_vec_mut_slice_i32_add_one;
    fn conv_vec_i32_len;
    fn conv_vec_i32_ret;
    fn conv_vec_f64_len;
    fn conv_vec_f64_ret;
    fn conv_vec_u8_len;
    fn conv_vec_u8_ret;
    fn conv_vec_rlog_len;
    fn conv_vec_rlog_ret;
    fn conv_vec_bool_len;
    fn conv_vec_bool_ret;
    fn conv_vec_string_len;
    fn conv_vec_string_ret;
    fn conv_vec_i8_len;
    fn conv_vec_i16_len;
    fn conv_vec_i64_len;
    fn conv_vec_isize_len;
    fn conv_vec_u16_len;
    fn conv_vec_u32_len;
    fn conv_vec_u64_len;
    fn conv_vec_usize_len;
    fn conv_vec_f32_len;
    fn conv_vec_opt_i32_len;
    fn conv_vec_opt_i32_ret;
    fn conv_vec_opt_f64_len;
    fn conv_vec_opt_f64_ret;
    fn conv_vec_opt_bool_len;
    fn conv_vec_opt_bool_ret;
    fn conv_vec_opt_string_len;
    fn conv_vec_opt_string_ret;
    fn conv_vec_opt_rlog_ret;
    fn conv_vec_opt_rbool_ret;
    fn conv_vec_opt_i8_len;
    fn conv_vec_opt_u8_len;
    fn conv_hashset_i32_len;
    fn conv_hashset_i32_ret;
    fn conv_hashset_u8_len;
    fn conv_hashset_u8_ret;
    fn conv_hashset_string_len;
    fn conv_hashset_string_ret;
    fn conv_hashset_rlog_len;
    fn conv_hashset_rlog_ret;
    fn conv_btreeset_i32_len;
    fn conv_btreeset_i32_ret;
    fn conv_btreeset_u8_len;
    fn conv_btreeset_u8_ret;
    fn conv_btreeset_string_len;
    fn conv_btreeset_string_ret;
    fn conv_hashmap_i32_len;
    fn conv_hashmap_i32_ret;
    fn conv_hashmap_f64_len;
    fn conv_hashmap_f64_ret;
    fn conv_hashmap_string_len;
    fn conv_hashmap_string_ret;
    fn conv_hashmap_rlog_len;
    fn conv_hashmap_rlog_ret;
    fn conv_btreemap_i32_len;
    fn conv_btreemap_i32_ret;
    fn conv_btreemap_f64_len;
    fn conv_btreemap_f64_ret;
    fn conv_btreemap_string_len;
    fn conv_btreemap_string_ret;
    fn conv_btreemap_rlog_len;
    fn conv_btreemap_rlog_ret;
    fn conv_list_mut_set_first;
    fn conv_result_i32_arg;
    fn conv_result_vec_i32_arg;
    fn conv_result_i32_ok;
    fn conv_result_i32_err;
    fn conv_result_f64_ok;
    fn conv_result_f64_err;
    fn conv_result_string_ok;
    fn conv_result_string_err;
    fn conv_result_vec_i32_ok;
    fn conv_result_vec_i32_err;

    // Extended conversions
    fn conv_char_arg;
    fn conv_char_ret;
    fn conv_vec_i8_ret;
    fn conv_vec_i16_ret;
    fn conv_vec_u16_ret;
    fn conv_vec_f32_ret;
    fn conv_hashset_i8_ret;
    fn conv_btreeset_i8_ret;
    fn conv_hashset_i16_ret;
    fn conv_btreeset_i16_ret;
    fn conv_hashset_u16_ret;
    fn conv_btreeset_u16_ret;
    fn conv_opt_ref_i32_some_ret;
    fn conv_opt_ref_i32_none_ret;
    fn conv_opt_vec_i32_arg;
    fn conv_opt_vec_i32_some_ret;
    fn conv_opt_vec_i32_none_ret;
    fn conv_opt_vec_string_arg;
    fn conv_opt_vec_string_some_ret;
    fn conv_opt_vec_string_none_ret;
    fn conv_opt_hashmap_i32_arg;
    fn conv_opt_hashmap_i32_some_ret;
    fn conv_opt_hashmap_i32_none_ret;
    fn conv_opt_hashset_i32_arg;
    fn conv_opt_hashset_i32_some_ret;
    fn conv_opt_hashset_i32_none_ret;
    fn conv_vec_vec_i32_arg;
    fn conv_vec_vec_i32_ret;
    fn conv_vec_vec_string_ret;

    // Vec<Option<T>> extended numeric types
    fn conv_vec_option_i64_ret_small;
    fn conv_vec_option_i64_ret_big;
    fn conv_vec_option_i64_roundtrip;
    fn conv_vec_option_u64_ret_small;
    fn conv_vec_option_u64_ret_big;
    fn conv_vec_option_isize_ret_small;
    fn conv_vec_option_usize_ret_small;
    fn conv_vec_option_u32_ret;
    fn conv_vec_option_f32_ret;
    fn conv_vec_option_i8_ret;
    fn conv_vec_option_i16_ret;
    fn conv_vec_option_u16_ret;

    // Scalar Option<T> extended numeric types
    fn conv_option_i64_some_small;
    fn conv_option_i64_some_big;
    fn conv_option_i64_none;
    fn conv_option_f32_some;
    fn conv_option_u32_some;

    // Named pair wrappers
    fn conv_as_named_list_vec;
    fn conv_as_named_list_array;
    fn conv_as_named_list_heterogeneous;
    fn conv_as_named_list_str_keys;
    fn conv_as_named_list_empty;
    fn conv_as_named_list_duplicate_names;
    fn conv_as_named_vector_i32;
    fn conv_as_named_vector_f64;
    fn conv_as_named_vector_string;
    fn conv_as_named_vector_option_i32;
    fn conv_as_named_vector_array;
    fn conv_as_named_vector_empty;
    fn conv_as_named_vector_ext_trait;
    fn conv_as_named_list_ext_trait;
    fn conv_as_named_list_slice;
    fn conv_as_named_vector_slice;
}
