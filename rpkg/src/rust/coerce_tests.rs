//! Tests for [`Coerce`], [`TryCoerce`], and [`RNativeType`] traits.

use miniextendr_api::{Coerce, CoerceError, RNativeType, TryCoerce, miniextendr};

// Test 6: `RNativeType` derive macro - newtype wrappers (both tuple and named field)
#[derive(Clone, Copy, RNativeType)]
struct UserId(i32); // Tuple struct

#[allow(dead_code)] // Demonstrates RNativeType derive on pub tuple struct
#[derive(Clone, Copy, RNativeType)]
pub struct Score(pub f64); // Tuple struct

#[derive(Clone, Copy, RNativeType)]
struct Temperature {
    celsius: f64,
} // Named single-field struct

// Verify the derived `RNativeType` works with `Coerce`
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
/// Test RNativeType derive on a tuple struct newtype (i32 -> UserId -> i32).
/// @param id Integer input coerced to UserId.
#[miniextendr]
pub fn test_rnative_newtype(id: i32) -> i32 {
    let user_id: UserId = id.coerce();
    user_id.0 // Extract inner value
}

// Test function using the named-field newtype
/// Test RNativeType derive on a named-field struct (f64 -> Temperature -> f64).
/// @param temp Numeric input coerced to Temperature.
#[miniextendr]
pub fn test_rnative_named_field(temp: f64) -> f64 {
    let t: Temperature = temp.coerce();
    t.celsius // Extract inner value
}

// NOTE: Generic functions like `fn foo<T: Coerce<i32>>(x: T)` DON'T work with miniextendr
// because the macro generates `TryFromSexp::try_from_sexp(x)` which needs to know the
// concrete type `T` at compile time, but T can't be inferred from just the trait bound.
//
// What DOES work:
// 1. Concrete functions that use Coerce internally
// 2. Helper functions with generics that are called with concrete types

// Test 1: Concrete function using Coerce internally (identity)
#[miniextendr]
/// @title Coercion Tests
/// @name rpkg_coercion_tests
/// @description Coercion and RNativeType tests
/// @examples
/// test_coerce_identity(1L)
/// test_coerce_widen(1L)
/// test_try_coerce_f64_to_i32(1.2)
/// test_coerce_attr_u16(10L)
/// test_per_arg_coerce_first(10L, 5L)
/// @aliases test_coerce_identity test_coerce_widen test_coerce_bool_to_int test_coerce_via_helper
///   test_try_coerce_f64_to_i32 test_rnative_newtype test_rnative_named_field test_coerce_attr_u16
///   test_coerce_attr_i16 test_coerce_attr_vec_u16 test_coerce_attr_f32
///   test_coerce_attr_with_invisible test_per_arg_coerce_first test_per_arg_coerce_second
///   test_per_arg_coerce_both test_per_arg_coerce_vec
pub fn test_coerce_identity(x: i32) -> i32 {
    Coerce::<i32>::coerce(x)
}

// Test 2: Widening coercion (i32 -> f64, always succeeds)
/// Test widening coercion from i32 to f64 via Coerce trait.
/// @param x Integer scalar input.
#[miniextendr]
pub fn test_coerce_widen(x: i32) -> f64 {
    x.coerce()
}

// Test 3: bool -> i32 coercion
/// Test coercion from Rboolean to i32 via Coerce trait.
/// @param x Logical scalar input.
#[miniextendr]
pub fn test_coerce_bool_to_int(x: miniextendr_api::ffi::Rboolean) -> i32 {
    x.coerce()
}

// Test 4: Helper using trait bound - called with concrete types
fn helper_accepts_integer<T: Coerce<i32>>(x: T) -> i32 {
    x.coerce()
}

/// Test that a generic helper with Coerce trait bound works with a concrete i32.
/// @param x Integer scalar input.
#[miniextendr]
pub fn test_coerce_via_helper(x: i32) -> i32 {
    // The generic helper works because x is concrete i32 at call site
    helper_accepts_integer(x)
}

// Test 5: TryCoerce - narrowing with potential failure
/// Test TryCoerce narrowing from f64 to i32 (returns NA on overflow, precision loss, or NaN).
/// @param x Numeric scalar input.
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

// region: #[miniextendr(coerce)] attribute tests

// Test 6: Coerce attribute - scalar i32 → u16
// R: test_coerce_attr_u16(100L) should return 100
// R: test_coerce_attr_u16(-1L) should error (overflow)
/// Test `#[miniextendr(coerce)]` attribute converting i32 to u16 (errors on negative).
/// @param x Integer scalar coerced to u16.
#[miniextendr(coerce)]
pub fn test_coerce_attr_u16(x: u16) -> i32 {
    x as i32 // Return as R integer
}

// Test 7: Coerce attribute - scalar i32 -> i16
/// Test `#[miniextendr(coerce)]` attribute converting i32 to i16.
/// @param x Integer scalar coerced to i16.
#[miniextendr(coerce)]
pub fn test_coerce_attr_i16(x: i16) -> i32 {
    x as i32
}

// Test 8: Coerce attribute - Vec<i32> -> Vec<u16>
/// Test `#[miniextendr(coerce)]` attribute converting `Vec<i32>` to `Vec<u16>` then summing.
/// @param x Integer vector coerced element-wise to u16.
#[miniextendr(coerce)]
pub fn test_coerce_attr_vec_u16(x: Vec<u16>) -> i32 {
    x.iter().map(|&v| v as i32).sum()
}

// Test 9: Coerce attribute - scalar f64 -> f32
/// Test `#[miniextendr(coerce)]` attribute narrowing f64 to f32.
/// @param x Numeric scalar coerced to f32.
#[miniextendr(coerce)]
pub fn test_coerce_attr_f32(x: f32) -> f64 {
    x as f64
}

// Test 10: Coerce attribute - combined with other attributes
/// Test `#[miniextendr(coerce, invisible)]` combining coercion with invisible return.
/// @param x Integer scalar coerced to u16.
#[miniextendr(coerce, invisible)]
pub fn test_coerce_attr_with_invisible(x: u16) -> i32 {
    x as i32
}
// endregion

// region: Per-argument #[miniextendr(coerce)] attribute tests

// Test 11: Per-argument coerce - only first argument is coerced
/// Test per-argument coercion on the first parameter only.
/// @param x Integer scalar coerced to u16.
/// @param y Integer scalar (no coercion).
#[miniextendr]
pub fn test_per_arg_coerce_first(#[miniextendr(coerce)] x: u16, y: i32) -> i32 {
    x as i32 + y
}

// Test 12: Per-argument coerce - only second argument is coerced
/// Test per-argument coercion on the second parameter only.
/// @param x Integer scalar (no coercion).
/// @param y Integer scalar coerced to u16.
#[miniextendr]
pub fn test_per_arg_coerce_second(x: i32, #[miniextendr(coerce)] y: u16) -> i32 {
    x + y as i32
}

// Test 13: Per-argument coerce - both arguments coerced
/// Test per-argument coercion on both parameters simultaneously.
/// @param x Integer scalar coerced to u16.
/// @param y Integer scalar coerced to i16.
#[miniextendr]
pub fn test_per_arg_coerce_both(
    #[miniextendr(coerce)] x: u16,
    #[miniextendr(coerce)] y: i16,
) -> i32 {
    x as i32 + y as i32
}

// Test 14: Per-argument coerce - Vec coercion
/// Test per-argument coercion on a Vec parameter (Vec<i32> coerced to Vec<u16>).
/// @param x Integer vector coerced element-wise to u16.
/// @param y Integer scalar added to the sum.
#[miniextendr]
pub fn test_per_arg_coerce_vec(#[miniextendr(coerce)] x: Vec<u16>, y: i32) -> i32 {
    x.iter().map(|&v| v as i32).sum::<i32>() + y
}
// endregion
