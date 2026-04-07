//! Scalar and slice conversion tests for R ↔ Rust type conversions.

use miniextendr_api::miniextendr;

// region: scalar conversion tests

// i32 tests
#[miniextendr]
/// @title Conversion Tests
/// @name rpkg_conversion_tests
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

/// Test i32 scalar increment by one.
/// @param x Integer scalar input.
#[miniextendr]
pub fn test_i32_add_one(x: i32) -> i32 {
    x + 1
}

/// Test i32 three-argument addition.
/// @param a Integer scalar.
/// @param b Integer scalar.
/// @param c Integer scalar.
#[miniextendr]
pub fn test_i32_sum(a: i32, b: i32, c: i32) -> i32 {
    a + b + c
}

// f64 tests
/// Test f64 scalar identity roundtrip.
/// @param x Numeric scalar input.
#[miniextendr]
pub fn test_f64_identity(x: f64) -> f64 {
    x
}

/// Test f64 scalar increment by one.
/// @param x Numeric scalar input.
#[miniextendr]
pub fn test_f64_add_one(x: f64) -> f64 {
    x + 1.0
}

/// Test f64 scalar multiplication.
/// @param a Numeric scalar.
/// @param b Numeric scalar.
#[miniextendr]
pub fn test_f64_multiply(a: f64, b: f64) -> f64 {
    a * b
}

// u8 (raw) tests
/// Test u8 (raw byte) scalar identity roundtrip.
/// @param x Raw scalar input.
#[miniextendr]
pub fn test_u8_identity(x: u8) -> u8 {
    x
}

/// Test u8 scalar wrapping increment by one.
/// @param x Raw scalar input.
#[miniextendr]
pub fn test_u8_add_one(x: u8) -> u8 {
    x.wrapping_add(1)
}

// Rboolean tests
/// Test logical scalar identity roundtrip.
/// @param x Logical scalar input.
#[miniextendr]
pub fn test_logical_identity(x: miniextendr_api::ffi::Rboolean) -> miniextendr_api::ffi::Rboolean {
    x
}

/// Test logical negation (TRUE becomes FALSE and vice versa).
/// @param x Logical scalar input.
#[miniextendr]
pub fn test_logical_not(x: miniextendr_api::ffi::Rboolean) -> miniextendr_api::ffi::Rboolean {
    use miniextendr_api::ffi::Rboolean;
    match x {
        Rboolean::TRUE => Rboolean::FALSE,
        _ => Rboolean::TRUE,
    }
}

/// Test logical AND of two Rboolean scalars.
/// @param a Logical scalar.
/// @param b Logical scalar.
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
/// Test widening conversion from i32 to f64.
/// @param x Integer scalar input.
#[miniextendr]
pub fn test_i32_to_f64(x: i32) -> f64 {
    x as f64
}

/// Test truncating conversion from f64 to i32.
/// @param x Numeric scalar input.
#[miniextendr]
pub fn test_f64_to_i32(x: f64) -> i32 {
    x as i32
}

// endregion

// region: slice tests

// Slice tests - i32
/// Test getting the length of an i32 slice from R.
/// @param x Integer vector input.
#[miniextendr]
pub fn test_i32_slice_len(x: &'static [i32]) -> i32 {
    x.len() as i32
}

/// Test summing elements of an i32 slice.
/// @param x Integer vector input.
#[miniextendr]
pub fn test_i32_slice_sum(x: &'static [i32]) -> i32 {
    x.iter().sum()
}

/// Test extracting the first element of an i32 slice (0 if empty).
/// @param x Integer vector input.
#[miniextendr]
pub fn test_i32_slice_first(x: &'static [i32]) -> i32 {
    x.first().copied().unwrap_or(0)
}

/// Test extracting the last element of an i32 slice (0 if empty).
/// @param x Integer vector input.
#[miniextendr]
pub fn test_i32_slice_last(x: &'static [i32]) -> i32 {
    x.last().copied().unwrap_or(0)
}

// Slice tests - f64
/// Test getting the length of an f64 slice from R.
/// @param x Numeric vector input.
#[miniextendr]
pub fn test_f64_slice_len(x: &'static [f64]) -> i32 {
    x.len() as i32
}

/// Test summing elements of an f64 slice.
/// @param x Numeric vector input.
#[miniextendr]
pub fn test_f64_slice_sum(x: &'static [f64]) -> f64 {
    x.iter().sum()
}

/// Test computing the arithmetic mean of an f64 slice (0.0 if empty).
/// @param x Numeric vector input.
#[miniextendr]
pub fn test_f64_slice_mean(x: &'static [f64]) -> f64 {
    if x.is_empty() {
        0.0
    } else {
        x.iter().sum::<f64>() / x.len() as f64
    }
}

// Slice tests - u8 (raw)
/// Test getting the length of a raw byte slice.
/// @param x Raw vector input.
#[miniextendr]
pub fn test_u8_slice_len(x: &'static [u8]) -> i32 {
    x.len() as i32
}

/// Test summing elements of a raw byte slice as i32.
/// @param x Raw vector input.
#[miniextendr]
pub fn test_u8_slice_sum(x: &'static [u8]) -> i32 {
    x.iter().map(|&b| b as i32).sum()
}

// Slice tests - logical
/// Test getting the length of an RLogical slice.
/// @param x Logical vector input.
#[miniextendr]
pub fn test_logical_slice_len(x: &'static [miniextendr_api::ffi::RLogical]) -> i32 {
    x.len() as i32
}

/// Test whether any element of a logical slice is TRUE.
/// @param x Logical vector input.
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

/// Test whether all elements of a logical slice are TRUE.
/// @param x Logical vector input.
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

// region: strict conversion tests

/// Test strict-mode i64 scalar identity roundtrip.
/// @param x Integer or numeric scalar (must be INTSXP or REALSXP in strict mode).
#[miniextendr(strict)]
pub fn strict_echo_i64(x: i64) -> i64 {
    x
}

/// Test strict-mode Vec<i64> identity roundtrip.
/// @param x Integer or numeric vector (must be INTSXP or REALSXP in strict mode).
#[miniextendr(strict)]
pub fn strict_echo_vec_i64(x: Vec<i64>) -> Vec<i64> {
    x
}

/// Test strict-mode Vec<Option<i64>> roundtrip with NA preservation.
/// @param x Integer or numeric vector with possible NA values.
#[miniextendr(strict)]
pub fn strict_echo_vec_option_i64(x: Vec<Option<i64>>) -> Vec<Option<i64>> {
    x
}

/// R6 class for testing strict conversion on impl methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct StrictCounter {
    value: i64,
}

/// @param value Integer or numeric scalar for the initial counter value.
#[miniextendr(r6, strict)]
impl StrictCounter {
    pub fn new(value: i64) -> Self {
        StrictCounter { value }
    }

    pub fn get_value(&self) -> i64 {
        self.value
    }

    pub fn add(&mut self, x: i64) -> i64 {
        self.value += x;
        self.value
    }
}

// endregion
