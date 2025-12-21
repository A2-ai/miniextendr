//! Tests for Coerce, TryCoerce, and RNativeType traits.

use miniextendr_api::{
    Coerce, CoerceError, RNativeType, TryCoerce, miniextendr, miniextendr_module,
};

// Test 6: RNativeType derive macro - newtype wrappers (both tuple and named field)
#[derive(Clone, Copy, RNativeType)]
struct UserId(i32); // Tuple struct

#[allow(dead_code)] // Demonstrates RNativeType derive on pub tuple struct
#[derive(Clone, Copy, RNativeType)]
pub struct Score(pub f64); // Tuple struct

#[derive(Clone, Copy, RNativeType)]
struct Temperature {
    celsius: f64,
} // Named single-field struct

// Verify the derived RNativeType works with Coerce
impl Coerce<UserId> for i32 {
    fn coerce(self) -> UserId {
        UserId(self)
    }
}

impl Coerce<Temperature> for f64 {
    fn coerce(self) -> Temperature {
        Temperature { celsius: self }
    }
}

// Test function using the tuple newtype
#[miniextendr]
pub fn test_rnative_newtype(id: i32) -> i32 {
    let user_id: UserId = id.coerce();
    user_id.0 // Extract inner value
}

// Test function using the named-field newtype
#[miniextendr]
pub fn test_rnative_named_field(temp: f64) -> f64 {
    let t: Temperature = temp.coerce();
    t.celsius // Extract inner value
}

// NOTE: Generic functions like `fn foo<T: Coerce<i32>>(x: T)` DON'T work with miniextendr
// because the macro generates `TryFromSexp::try_from_sexp(x)` which needs to know the
// concrete type T at compile time, but T can't be inferred from just the trait bound.
//
// What DOES work:
// 1. Concrete functions that use Coerce internally
// 2. Helper functions with generics that are called with concrete types

// Test 1: Concrete function using Coerce internally (identity)
#[miniextendr]
/// @title Coercion Tests
/// @name rpkg_coercion_tests
/// @keywords internal
/// @description Coercion and RNativeType tests
/// @examples
/// test_coerce_identity(1L)
/// test_coerce_widen(1L)
/// test_try_coerce_f64_to_i32(1.2)
/// test_coerce_attr_u16(10L)
/// test_per_arg_coerce_first(10L, 5L)
/// @aliases test_coerce_identity test_coerce_widen test_coerce_bool_to_int
/// @aliases test_coerce_via_helper test_try_coerce_f64_to_i32
/// @aliases test_rnative_newtype test_rnative_named_field
/// @aliases test_coerce_attr_u16 test_coerce_attr_i16 test_coerce_attr_vec_u16
/// @aliases test_coerce_attr_f32 test_coerce_attr_with_invisible
/// @aliases test_per_arg_coerce_first test_per_arg_coerce_second
/// @aliases test_per_arg_coerce_both test_per_arg_coerce_vec
pub fn test_coerce_identity(x: i32) -> i32 {
    Coerce::<i32>::coerce(x)
}

// Test 2: Widening coercion (i32 → f64, always succeeds)
#[miniextendr]
pub fn test_coerce_widen(x: i32) -> f64 {
    x.coerce()
}

// Test 3: bool → i32 coercion
#[miniextendr]
pub fn test_coerce_bool_to_int(x: miniextendr_api::ffi::Rboolean) -> i32 {
    x.coerce()
}

// Test 4: Helper using trait bound - called with concrete types
fn helper_accepts_integer<T: Coerce<i32>>(x: T) -> i32 {
    x.coerce()
}

#[miniextendr]
pub fn test_coerce_via_helper(x: i32) -> i32 {
    // The generic helper works because x is concrete i32 at call site
    helper_accepts_integer(x)
}

// Test 5: TryCoerce - narrowing with potential failure
#[miniextendr]
pub fn test_try_coerce_f64_to_i32(x: f64) -> i32 {
    match TryCoerce::<i32>::try_coerce(x) {
        Ok(v) => v,
        Err(CoerceError::Overflow) => i32::MIN, // NA
        Err(CoerceError::PrecisionLoss) => i32::MIN,
        Err(CoerceError::NaN) => i32::MIN,
        Err(CoerceError::Zero) => i32::MIN,
    }
}

// =============================================================================
// #[miniextendr(coerce)] attribute tests
// =============================================================================

// Test 6: Coerce attribute - scalar i32 → u16
// R: test_coerce_attr_u16(100L) should return 100
// R: test_coerce_attr_u16(-1L) should error (overflow)
#[miniextendr(coerce)]
pub fn test_coerce_attr_u16(x: u16) -> i32 {
    x as i32 // Return as R integer
}

// Test 7: Coerce attribute - scalar i32 → i16
#[miniextendr(coerce)]
pub fn test_coerce_attr_i16(x: i16) -> i32 {
    x as i32
}

// Test 8: Coerce attribute - Vec<i32> → Vec<u16>
#[miniextendr(coerce)]
pub fn test_coerce_attr_vec_u16(x: Vec<u16>) -> i32 {
    x.iter().map(|&v| v as i32).sum()
}

// Test 9: Coerce attribute - scalar f64 → f32
#[miniextendr(coerce)]
pub fn test_coerce_attr_f32(x: f32) -> f64 {
    x as f64
}

// Test 10: Coerce attribute - combined with other attributes
#[miniextendr(coerce, invisible)]
pub fn test_coerce_attr_with_invisible(x: u16) -> i32 {
    x as i32
}

// =============================================================================
// Per-argument #[miniextendr(coerce)] attribute tests
// =============================================================================

// Test 11: Per-argument coerce - only first argument is coerced
#[miniextendr]
pub fn test_per_arg_coerce_first(#[miniextendr(coerce)] x: u16, y: i32) -> i32 {
    x as i32 + y
}

// Test 12: Per-argument coerce - only second argument is coerced
#[miniextendr]
pub fn test_per_arg_coerce_second(x: i32, #[miniextendr(coerce)] y: u16) -> i32 {
    x + y as i32
}

// Test 13: Per-argument coerce - both arguments coerced
#[miniextendr]
pub fn test_per_arg_coerce_both(
    #[miniextendr(coerce)] x: u16,
    #[miniextendr(coerce)] y: i16,
) -> i32 {
    x as i32 + y as i32
}

// Test 14: Per-argument coerce - Vec coercion
#[miniextendr]
pub fn test_per_arg_coerce_vec(#[miniextendr(coerce)] x: Vec<u16>, y: i32) -> i32 {
    x.iter().map(|&v| v as i32).sum::<i32>() + y
}

miniextendr_module! {
    mod coerce_tests;

    // Coerce trait tests
    fn test_coerce_identity;
    fn test_coerce_widen;
    fn test_coerce_bool_to_int;
    fn test_coerce_via_helper;
    fn test_try_coerce_f64_to_i32;
    fn test_rnative_newtype;
    fn test_rnative_named_field;

    // Coerce attribute tests
    fn test_coerce_attr_u16;
    fn test_coerce_attr_i16;
    fn test_coerce_attr_vec_u16;
    fn test_coerce_attr_f32;
    fn test_coerce_attr_with_invisible;

    // Per-argument coerce tests
    fn test_per_arg_coerce_first;
    fn test_per_arg_coerce_second;
    fn test_per_arg_coerce_both;
    fn test_per_arg_coerce_vec;
}
