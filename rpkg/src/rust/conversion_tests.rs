//! Scalar and slice conversion tests for R ↔ Rust type conversions.

use miniextendr_api::{miniextendr, miniextendr_module};

// region: scalar conversion tests

// i32 tests
#[miniextendr]
/// @title Conversion Tests
/// @name rpkg_conversion_tests
/// @noRd
/// @description Scalar and slice conversion tests
/// @examples
/// test_i32_add_one(1L)
/// test_f64_multiply(2, 3)
/// test_u8_identity(as.raw(255))
/// test_i32_slice_sum(1:5)
/// test_f64_slice_mean(c(1, 2, 3))
/// @aliases test_i32_identity test_i32_add_one test_i32_sum test_f64_identity test_f64_add_one
///   test_f64_multiply test_u8_identity test_u8_add_one test_logical_identity test_logical_not
///   test_logical_and test_i32_to_f64 test_f64_to_i32 test_i32_slice_len test_i32_slice_sum
///   test_i32_slice_first test_i32_slice_last test_f64_slice_len test_f64_slice_sum
///   test_f64_slice_mean test_u8_slice_len test_u8_slice_sum test_logical_slice_len
///   test_logical_slice_any_true test_logical_slice_all_true
pub fn test_i32_identity(x: i32) -> i32 {
    x
}

/// @noRd
#[miniextendr]
pub fn test_i32_add_one(x: i32) -> i32 {
    x + 1
}

/// @noRd
#[miniextendr]
pub fn test_i32_sum(a: i32, b: i32, c: i32) -> i32 {
    a + b + c
}

// f64 tests
/// @noRd
#[miniextendr]
pub fn test_f64_identity(x: f64) -> f64 {
    x
}

/// @noRd
#[miniextendr]
pub fn test_f64_add_one(x: f64) -> f64 {
    x + 1.0
}

/// @noRd
#[miniextendr]
pub fn test_f64_multiply(a: f64, b: f64) -> f64 {
    a * b
}

// u8 (raw) tests
/// @noRd
#[miniextendr]
pub fn test_u8_identity(x: u8) -> u8 {
    x
}

/// @noRd
#[miniextendr]
pub fn test_u8_add_one(x: u8) -> u8 {
    x.wrapping_add(1)
}

// Rboolean tests
/// @noRd
#[miniextendr]
pub fn test_logical_identity(x: miniextendr_api::ffi::Rboolean) -> miniextendr_api::ffi::Rboolean {
    x
}

/// @noRd
#[miniextendr]
pub fn test_logical_not(x: miniextendr_api::ffi::Rboolean) -> miniextendr_api::ffi::Rboolean {
    use miniextendr_api::ffi::Rboolean;
    match x {
        Rboolean::TRUE => Rboolean::FALSE,
        _ => Rboolean::TRUE,
    }
}

/// @noRd
#[miniextendr]
pub fn test_logical_and(
    a: miniextendr_api::ffi::Rboolean,
    b: miniextendr_api::ffi::Rboolean,
) -> miniextendr_api::ffi::Rboolean {
    use miniextendr_api::ffi::Rboolean;
    match (a, b) {
        (Rboolean::TRUE, Rboolean::TRUE) => Rboolean::TRUE,
        _ => Rboolean::FALSE,
    }
}

// Mixed type tests
/// @noRd
#[miniextendr]
pub fn test_i32_to_f64(x: i32) -> f64 {
    x as f64
}

/// @noRd
#[miniextendr]
pub fn test_f64_to_i32(x: f64) -> i32 {
    x as i32
}

// endregion

// region: slice tests

// Slice tests - i32
/// @noRd
#[miniextendr]
pub fn test_i32_slice_len(x: &'static [i32]) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn test_i32_slice_sum(x: &'static [i32]) -> i32 {
    x.iter().sum()
}

/// @noRd
#[miniextendr]
pub fn test_i32_slice_first(x: &'static [i32]) -> i32 {
    x.first().copied().unwrap_or(0)
}

/// @noRd
#[miniextendr]
pub fn test_i32_slice_last(x: &'static [i32]) -> i32 {
    x.last().copied().unwrap_or(0)
}

// Slice tests - f64
/// @noRd
#[miniextendr]
pub fn test_f64_slice_len(x: &'static [f64]) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn test_f64_slice_sum(x: &'static [f64]) -> f64 {
    x.iter().sum()
}

/// @noRd
#[miniextendr]
pub fn test_f64_slice_mean(x: &'static [f64]) -> f64 {
    if x.is_empty() {
        0.0
    } else {
        x.iter().sum::<f64>() / x.len() as f64
    }
}

// Slice tests - u8 (raw)
/// @noRd
#[miniextendr]
pub fn test_u8_slice_len(x: &'static [u8]) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn test_u8_slice_sum(x: &'static [u8]) -> i32 {
    x.iter().map(|&b| b as i32).sum()
}

// Slice tests - logical
/// @noRd
#[miniextendr]
pub fn test_logical_slice_len(x: &'static [miniextendr_api::ffi::RLogical]) -> i32 {
    x.len() as i32
}

/// @noRd
#[miniextendr]
pub fn test_logical_slice_any_true(
    x: &'static [miniextendr_api::ffi::RLogical],
) -> miniextendr_api::ffi::Rboolean {
    use miniextendr_api::ffi::Rboolean;
    if x.iter().any(|v| v.to_option_bool() == Some(true)) {
        Rboolean::TRUE
    } else {
        Rboolean::FALSE
    }
}

/// @noRd
#[miniextendr]
pub fn test_logical_slice_all_true(
    x: &'static [miniextendr_api::ffi::RLogical],
) -> miniextendr_api::ffi::Rboolean {
    use miniextendr_api::ffi::Rboolean;
    if x.iter().all(|v| v.to_option_bool() == Some(true)) {
        Rboolean::TRUE
    } else {
        Rboolean::FALSE
    }
}

// endregion

miniextendr_module! {
    mod conversion_tests;

    // Scalar conversion tests
    fn test_i32_identity;
    fn test_i32_add_one;
    fn test_i32_sum;
    fn test_f64_identity;
    fn test_f64_add_one;
    fn test_f64_multiply;
    fn test_u8_identity;
    fn test_u8_add_one;
    fn test_logical_identity;
    fn test_logical_not;
    fn test_logical_and;
    fn test_i32_to_f64;
    fn test_f64_to_i32;

    // Slice conversion tests
    fn test_i32_slice_len;
    fn test_i32_slice_sum;
    fn test_i32_slice_first;
    fn test_i32_slice_last;
    fn test_f64_slice_len;
    fn test_f64_slice_sum;
    fn test_f64_slice_mean;
    fn test_u8_slice_len;
    fn test_u8_slice_sum;
    fn test_logical_slice_len;
    fn test_logical_slice_any_true;
    fn test_logical_slice_all_true;
}
