//! Type coercion traits for converting Rust types to R native types.
//!
//! R has a fixed set of native scalar types:
//! - `i32` (INTSXP) - 32-bit signed integer
//! - `f64` (REALSXP) - 64-bit floating point
//! - `Rboolean` (LGLSXP) - logical (TRUE/FALSE/NA)
//! - `u8` (RAWSXP) - raw bytes
//! - `Rcomplex` (CPLXSXP) - complex numbers
//!
//! This module provides traits for coercing Rust types to these R native types:
//! - [`Coerce<R>`] - infallible coercion (identity, widening)
//! - [`TryCoerce<R>`] - fallible coercion (narrowing, overflow-possible)
//!
//! # String Types
//!
//! Strings are handled separately because R has two string representations:
//! - **CHARSXP** - single string (≈ `&str`, `String`)
//! - **STRSXP** - character vector (≈ `&[&str]`, `Vec<String>`)
//!
//! See [`IntoR`](crate::IntoR) for string-to-SEXP conversions.
//!
//! # Examples
//!
//! ```ignore
//! use miniextendr_api::coerce::{Coerce, TryCoerce, CoerceError};
//!
//! // Identity coercion (no-op)
//! let x: i32 = 42i32.coerce();
//!
//! // Widening coercion (infallible)
//! let y: i32 = 42i8.coerce();
//! let z: f64 = 3.14f32.coerce();
//!
//! // Narrowing coercion (fallible)
//! let a: Result<i32, _> = 42u32.try_coerce();  // Ok(42)
//! let b: Result<i32, _> = u32::MAX.try_coerce(); // Err(overflow)
//! ```

use crate::ffi::{Rboolean, Rcomplex, SEXPTYPE};

// =============================================================================
// Core traits
// =============================================================================

/// Marker trait for R's native scalar types.
///
/// These are the element types that can be stored in R vectors:
/// - `i32` → INTSXP (integer vector)
/// - `f64` → REALSXP (numeric/double vector)
/// - `Rboolean` → LGLSXP (logical vector)
/// - `u8` → RAWSXP (raw vector)
/// - `Rcomplex` → CPLXSXP (complex vector)
pub trait RNative: Copy + 'static {
    /// The SEXPTYPE for vectors containing this element type.
    const SEXP_TYPE: SEXPTYPE;
}

impl RNative for i32 {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::INTSXP;
}

impl RNative for f64 {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::REALSXP;
}

impl RNative for Rboolean {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::LGLSXP;
}

impl RNative for u8 {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::RAWSXP;
}

impl RNative for Rcomplex {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::CPLXSXP;
}

/// Infallible coercion from type `Self` to R native type `R`.
///
/// Use this for:
/// - Identity conversions (`i32` → `i32`)
/// - Widening conversions (`i8` → `i32`, `f32` → `f64`)
///
/// Identity implementations compile to no-ops.
pub trait Coerce<R: RNative> {
    /// Coerce this value to the target R native type.
    fn coerce(self) -> R;
}

/// Fallible coercion from type `Self` to R native type `R`.
///
/// Use this for:
/// - Narrowing conversions that may overflow (`u32` → `i32`)
/// - Conversions that may lose precision (`f64` → `i32`)
pub trait TryCoerce<R: RNative> {
    /// The error type returned when coercion fails.
    type Error;

    /// Attempt to coerce this value to the target R native type.
    fn try_coerce(self) -> Result<R, Self::Error>;
}

/// Error type for coercion failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoerceError {
    /// Value is outside the representable range of the target type.
    Overflow,
    /// Value cannot be represented exactly (precision loss).
    PrecisionLoss,
    /// Value is NaN and target type doesn't support NaN.
    NaN,
}

impl std::fmt::Display for CoerceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoerceError::Overflow => write!(f, "value out of range for target type"),
            CoerceError::PrecisionLoss => write!(f, "precision loss in conversion"),
            CoerceError::NaN => write!(f, "NaN cannot be converted to target type"),
        }
    }
}

impl std::error::Error for CoerceError {}

// =============================================================================
// Blanket implementation: Coerce implies TryCoerce
// =============================================================================

impl<T, R: RNative> TryCoerce<R> for T
where
    T: Coerce<R>,
{
    type Error = std::convert::Infallible;

    #[inline(always)]
    fn try_coerce(self) -> Result<R, Self::Error> {
        Ok(self.coerce())
    }
}

// =============================================================================
// Identity coercions (no-op)
// =============================================================================

impl Coerce<i32> for i32 {
    #[inline(always)]
    fn coerce(self) -> i32 {
        self
    }
}

impl Coerce<f64> for f64 {
    #[inline(always)]
    fn coerce(self) -> f64 {
        self
    }
}

impl Coerce<Rboolean> for Rboolean {
    #[inline(always)]
    fn coerce(self) -> Rboolean {
        self
    }
}

impl Coerce<u8> for u8 {
    #[inline(always)]
    fn coerce(self) -> u8 {
        self
    }
}

impl Coerce<Rcomplex> for Rcomplex {
    #[inline(always)]
    fn coerce(self) -> Rcomplex {
        self
    }
}

// =============================================================================
// Widening integer coercions to i32 (infallible)
// =============================================================================

impl Coerce<i32> for i8 {
    #[inline(always)]
    fn coerce(self) -> i32 {
        self.into()
    }
}

impl Coerce<i32> for i16 {
    #[inline(always)]
    fn coerce(self) -> i32 {
        self.into()
    }
}

impl Coerce<i32> for u8 {
    #[inline(always)]
    fn coerce(self) -> i32 {
        self.into()
    }
}

impl Coerce<i32> for u16 {
    #[inline(always)]
    fn coerce(self) -> i32 {
        self.into()
    }
}

// =============================================================================
// Widening float coercions to f64 (infallible)
// =============================================================================

impl Coerce<f64> for f32 {
    #[inline(always)]
    fn coerce(self) -> f64 {
        self.into()
    }
}

// Integer to f64 (widening, always fits)
impl Coerce<f64> for i8 {
    #[inline(always)]
    fn coerce(self) -> f64 {
        self.into()
    }
}

impl Coerce<f64> for i16 {
    #[inline(always)]
    fn coerce(self) -> f64 {
        self.into()
    }
}

impl Coerce<f64> for i32 {
    #[inline(always)]
    fn coerce(self) -> f64 {
        self.into()
    }
}

impl Coerce<f64> for u8 {
    #[inline(always)]
    fn coerce(self) -> f64 {
        self.into()
    }
}

impl Coerce<f64> for u16 {
    #[inline(always)]
    fn coerce(self) -> f64 {
        self.into()
    }
}

impl Coerce<f64> for u32 {
    #[inline(always)]
    fn coerce(self) -> f64 {
        self.into()
    }
}

// =============================================================================
// bool coercions
// =============================================================================

impl Coerce<Rboolean> for bool {
    #[inline(always)]
    fn coerce(self) -> Rboolean {
        if self {
            Rboolean::TRUE
        } else {
            Rboolean::FALSE
        }
    }
}

impl Coerce<i32> for bool {
    #[inline(always)]
    fn coerce(self) -> i32 {
        self as i32
    }
}

impl Coerce<f64> for bool {
    #[inline(always)]
    fn coerce(self) -> f64 {
        if self { 1.0 } else { 0.0 }
    }
}

// Rboolean to i32 (R's internal representation)
impl Coerce<i32> for Rboolean {
    #[inline(always)]
    fn coerce(self) -> i32 {
        self as i32
    }
}

// =============================================================================
// Narrowing coercions to i32 (fallible)
// =============================================================================

// Helper macro for TryCoerce implementations that use try_into
macro_rules! impl_try_coerce_via_try_into {
    ($from:ty => $to:ty) => {
        impl TryCoerce<$to> for $from {
            type Error = CoerceError;

            #[inline]
            fn try_coerce(self) -> Result<$to, Self::Error> {
                self.try_into().map_err(|_| CoerceError::Overflow)
            }
        }
    };
}

impl_try_coerce_via_try_into!(u32 => i32);
impl_try_coerce_via_try_into!(u64 => i32);
impl_try_coerce_via_try_into!(usize => i32);
impl_try_coerce_via_try_into!(i64 => i32);
impl_try_coerce_via_try_into!(isize => i32);

// =============================================================================
// Narrowing coercions to u8 (fallible)
// =============================================================================

impl_try_coerce_via_try_into!(i32 => u8);
impl_try_coerce_via_try_into!(i64 => u8);
impl_try_coerce_via_try_into!(u16 => u8);
impl_try_coerce_via_try_into!(u32 => u8);
impl_try_coerce_via_try_into!(u64 => u8);
impl_try_coerce_via_try_into!(usize => u8);
impl_try_coerce_via_try_into!(isize => u8);

// =============================================================================
// Float to integer coercions (fallible - may overflow or lose precision)
// =============================================================================

impl TryCoerce<i32> for f64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<i32, Self::Error> {
        if self.is_nan() {
            return Err(CoerceError::NaN);
        }
        if self < i32::MIN as f64 || self > i32::MAX as f64 {
            return Err(CoerceError::Overflow);
        }
        // Check for precision loss (fractional part)
        if self.fract() != 0.0 {
            return Err(CoerceError::PrecisionLoss);
        }
        Ok(self as i32)
    }
}

impl TryCoerce<i32> for f32 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<i32, Self::Error> {
        (self as f64).try_coerce()
    }
}

// =============================================================================
// Large integers to f64 (fallible - may lose precision)
// =============================================================================

impl TryCoerce<f64> for i64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<f64, Self::Error> {
        // f64 has 53 bits of mantissa, so it can exactly represent
        // integers in the range [-(2^53), 2^53]. Outside this range,
        // consecutive integers cannot be distinguished.
        const MAX_SAFE: i64 = 1 << 53; // 9007199254740992
        const MIN_SAFE: i64 = -(1 << 53);

        if !(MIN_SAFE..=MAX_SAFE).contains(&self) {
            return Err(CoerceError::PrecisionLoss);
        }
        Ok(self as f64)
    }
}

impl TryCoerce<f64> for u64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<f64, Self::Error> {
        // f64 can exactly represent integers in [0, 2^53]
        const MAX_SAFE: u64 = 1 << 53;

        if self > MAX_SAFE {
            return Err(CoerceError::PrecisionLoss);
        }
        Ok(self as f64)
    }
}

impl TryCoerce<f64> for isize {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<f64, Self::Error> {
        (self as i64).try_coerce()
    }
}

impl TryCoerce<f64> for usize {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<f64, Self::Error> {
        (self as u64).try_coerce()
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Identity coercions
    // -------------------------------------------------------------------------

    #[test]
    fn test_identity_i32() {
        let x: i32 = 42;
        assert_eq!(Coerce::<i32>::coerce(x), 42);
    }

    #[test]
    fn test_identity_f64() {
        let x: f64 = 3.14;
        assert_eq!(Coerce::<f64>::coerce(x), 3.14);
    }

    // -------------------------------------------------------------------------
    // Widening integer to i32
    // -------------------------------------------------------------------------

    #[test]
    fn test_i8_to_i32() {
        assert_eq!(Coerce::<i32>::coerce(42i8), 42i32);
        assert_eq!(Coerce::<i32>::coerce(-128i8), -128i32);
        assert_eq!(Coerce::<i32>::coerce(127i8), 127i32);
    }

    #[test]
    fn test_i16_to_i32() {
        assert_eq!(Coerce::<i32>::coerce(1000i16), 1000i32);
    }

    #[test]
    fn test_u8_to_i32() {
        assert_eq!(Coerce::<i32>::coerce(255u8), 255i32);
    }

    #[test]
    fn test_u16_to_i32() {
        assert_eq!(Coerce::<i32>::coerce(65535u16), 65535i32);
    }

    // -------------------------------------------------------------------------
    // Widening to f64
    // -------------------------------------------------------------------------

    #[test]
    fn test_f32_to_f64() {
        let x: f64 = Coerce::<f64>::coerce(3.14f32);
        assert!((x - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_i32_to_f64() {
        assert_eq!(Coerce::<f64>::coerce(42i32), 42.0f64);
    }

    #[test]
    fn test_u32_to_f64() {
        assert_eq!(Coerce::<f64>::coerce(u32::MAX), u32::MAX as f64);
    }

    // -------------------------------------------------------------------------
    // bool coercions
    // -------------------------------------------------------------------------

    #[test]
    fn test_bool_to_rboolean() {
        assert_eq!(Coerce::<Rboolean>::coerce(true), Rboolean::TRUE);
        assert_eq!(Coerce::<Rboolean>::coerce(false), Rboolean::FALSE);
    }

    #[test]
    fn test_bool_to_i32() {
        assert_eq!(Coerce::<i32>::coerce(true), 1);
        assert_eq!(Coerce::<i32>::coerce(false), 0);
    }

    #[test]
    fn test_bool_to_f64() {
        assert_eq!(Coerce::<f64>::coerce(true), 1.0);
        assert_eq!(Coerce::<f64>::coerce(false), 0.0);
    }

    #[test]
    fn test_rboolean_to_i32() {
        assert_eq!(Coerce::<i32>::coerce(Rboolean::TRUE), 1);
        assert_eq!(Coerce::<i32>::coerce(Rboolean::FALSE), 0);
    }

    // -------------------------------------------------------------------------
    // Narrowing to i32 (fallible)
    // -------------------------------------------------------------------------

    #[test]
    fn test_u32_to_i32_ok() {
        let result: Result<i32, _> = TryCoerce::<i32>::try_coerce(42u32);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_u32_to_i32_overflow() {
        let result: Result<i32, _> = TryCoerce::<i32>::try_coerce(u32::MAX);
        assert_eq!(result, Err(CoerceError::Overflow));
    }

    #[test]
    fn test_i64_to_i32_ok() {
        let result: Result<i32, _> = TryCoerce::<i32>::try_coerce(1000i64);
        assert_eq!(result, Ok(1000));
    }

    #[test]
    fn test_i64_to_i32_overflow() {
        let result: Result<i32, _> = TryCoerce::<i32>::try_coerce(i64::MAX);
        assert_eq!(result, Err(CoerceError::Overflow));
    }

    #[test]
    fn test_usize_to_i32() {
        let result: Result<i32, _> = TryCoerce::<i32>::try_coerce(100usize);
        assert_eq!(result, Ok(100));
    }

    // -------------------------------------------------------------------------
    // Float to integer (fallible)
    // -------------------------------------------------------------------------

    #[test]
    fn test_f64_to_i32_ok() {
        let result: Result<i32, _> = TryCoerce::<i32>::try_coerce(42.0f64);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_f64_to_i32_overflow() {
        let result: Result<i32, _> = TryCoerce::<i32>::try_coerce(1e15f64);
        assert_eq!(result, Err(CoerceError::Overflow));
    }

    #[test]
    fn test_f64_to_i32_precision_loss() {
        let result: Result<i32, _> = TryCoerce::<i32>::try_coerce(42.5f64);
        assert_eq!(result, Err(CoerceError::PrecisionLoss));
    }

    #[test]
    fn test_f64_to_i32_nan() {
        let result: Result<i32, _> = TryCoerce::<i32>::try_coerce(f64::NAN);
        assert_eq!(result, Err(CoerceError::NaN));
    }

    // -------------------------------------------------------------------------
    // Large integer to f64 (fallible precision)
    // -------------------------------------------------------------------------

    #[test]
    fn test_i64_to_f64_ok() {
        let result: Result<f64, _> = TryCoerce::<f64>::try_coerce(1000i64);
        assert_eq!(result, Ok(1000.0));
    }

    #[test]
    fn test_i64_to_f64_precision_loss() {
        // i64::MAX cannot be represented exactly in f64
        let result: Result<f64, _> = TryCoerce::<f64>::try_coerce(i64::MAX);
        assert_eq!(result, Err(CoerceError::PrecisionLoss));
    }

    // -------------------------------------------------------------------------
    // TryCoerce blanket for Coerce
    // -------------------------------------------------------------------------

    #[test]
    fn test_coerce_implies_try_coerce() {
        // i8 -> i32 is Coerce, so TryCoerce should always succeed
        let result: Result<i32, std::convert::Infallible> = TryCoerce::<i32>::try_coerce(42i8);
        assert_eq!(result, Ok(42));
    }
}
