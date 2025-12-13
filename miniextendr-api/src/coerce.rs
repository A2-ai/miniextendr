//! Type coercion traits for converting Rust types to R native types.
//!
//! R has a fixed set of native scalar types:
//! - `i32` (INTSXP) - 32-bit signed integer
//! - `f64` (REALSXP) - 64-bit floating point
//! - `Rboolean` (LGLSXP) - logical (TRUE/FALSE/NA)
//! - `u8` (RAWSXP) - raw bytes
//! - `Rcomplex` (CPLXSXP) - complex numbers
//!
//! # Traits
//!
//! - [`Coerce<R>`] - infallible coercion (identity, widening)
//! - [`TryCoerce<R>`] - fallible coercion (narrowing, overflow-possible)
//!
//! # Trait Bounds
//!
//! - [`CanCoerceToInteger`] - types that implement `Coerce<i32>`
//! - [`CanCoerceToReal`] - types that implement `Coerce<f64>`
//! - [`CanCoerceToLogical`] - types that implement `Coerce<Rboolean>`
//! - [`CanCoerceToRaw`] - types that implement `Coerce<u8>`
//!
//! # Examples
//!
//! ```ignore
//! use miniextendr_api::coerce::{Coerce, CanCoerceToInteger};
//!
//! // Scalar coercion
//! let x: i32 = 42i8.coerce();
//!
//! // Element-wise slice coercion
//! let slice: &[i8] = &[1, 2, 3];
//! let vec: Vec<i32> = slice.coerce();
//!
//! // Trait bounds
//! fn accepts_integer<T: CanCoerceToInteger>(x: T) -> i32 {
//!     x.coerce()
//! }
//! ```

use crate::ffi::{Rboolean, Rcomplex, SEXPTYPE};

// =============================================================================
// Core traits
// =============================================================================

/// Marker trait for R's native scalar types.
pub trait RNative: Copy + 'static {
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

/// Infallible coercion from `Self` to type `R`.
///
/// Implement this trait for types that can always be converted to `R`.
/// Identity and widening conversions should use this trait.
///
/// Works for both scalars and element-wise on slices:
/// - `i8::coerce() -> i32` (scalar widening)
/// - `&[i8]::coerce() -> Vec<i32>` (element-wise)
///
/// # Example
///
/// ```ignore
/// impl Coerce<i32> for MyType {
///     fn coerce(self) -> i32 { ... }
/// }
/// ```
pub trait Coerce<R> {
    fn coerce(self) -> R;
}

/// Fallible coercion from `Self` to type `R`.
///
/// Implement this trait for narrowing conversions that may overflow or lose precision.
pub trait TryCoerce<R> {
    type Error;
    fn try_coerce(self) -> Result<R, Self::Error>;
}

/// Error type for coercion failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoerceError {
    Overflow,
    PrecisionLoss,
    NaN,
}

impl std::fmt::Display for CoerceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoerceError::Overflow => write!(f, "value out of range"),
            CoerceError::PrecisionLoss => write!(f, "precision loss"),
            CoerceError::NaN => write!(f, "NaN cannot be converted"),
        }
    }
}

impl std::error::Error for CoerceError {}

// =============================================================================
// Trait bounds - for use in where clauses
// =============================================================================

/// Trait bound: `Coerce<i32>`.
pub trait CanCoerceToInteger: Coerce<i32> {}
impl<T: Coerce<i32>> CanCoerceToInteger for T {}

/// Trait bound: `Coerce<f64>`.
pub trait CanCoerceToReal: Coerce<f64> {}
impl<T: Coerce<f64>> CanCoerceToReal for T {}

/// Trait bound: `Coerce<Rboolean>`.
pub trait CanCoerceToLogical: Coerce<Rboolean> {}
impl<T: Coerce<Rboolean>> CanCoerceToLogical for T {}

/// Trait bound: `Coerce<u8>`.
pub trait CanCoerceToRaw: Coerce<u8> {}
impl<T: Coerce<u8>> CanCoerceToRaw for T {}

// =============================================================================
// Blanket: Coerce implies TryCoerce
// =============================================================================

impl<T, R> TryCoerce<R> for T
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
// Identity coercions
// =============================================================================

macro_rules! impl_identity {
    ($t:ty) => {
        impl Coerce<$t> for $t {
            #[inline(always)]
            fn coerce(self) -> $t {
                self
            }
        }
    };
}

impl_identity!(i32);
impl_identity!(f64);
impl_identity!(Rboolean);
impl_identity!(u8);
impl_identity!(Rcomplex);

// =============================================================================
// Widening to i32
// =============================================================================

macro_rules! impl_widen_i32 {
    ($t:ty) => {
        impl Coerce<i32> for $t {
            #[inline(always)]
            fn coerce(self) -> i32 {
                self.into()
            }
        }
    };
}

impl_widen_i32!(i8);
impl_widen_i32!(i16);
impl_widen_i32!(u8);
impl_widen_i32!(u16);

// =============================================================================
// Widening to f64
// =============================================================================

macro_rules! impl_widen_f64 {
    ($t:ty) => {
        impl Coerce<f64> for $t {
            #[inline(always)]
            fn coerce(self) -> f64 {
                self.into()
            }
        }
    };
}

impl_widen_f64!(f32);
impl_widen_f64!(i8);
impl_widen_f64!(i16);
impl_widen_f64!(i32);
impl_widen_f64!(u8);
impl_widen_f64!(u16);
impl_widen_f64!(u32);

// =============================================================================
// bool coercions
// =============================================================================

impl Coerce<Rboolean> for bool {
    #[inline(always)]
    fn coerce(self) -> Rboolean {
        if self { Rboolean::TRUE } else { Rboolean::FALSE }
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

impl Coerce<i32> for Rboolean {
    #[inline(always)]
    fn coerce(self) -> i32 {
        self as i32
    }
}

// =============================================================================
// Narrowing to i32 (fallible)
// =============================================================================

macro_rules! impl_try_i32 {
    ($t:ty) => {
        impl TryCoerce<i32> for $t {
            type Error = CoerceError;
            #[inline]
            fn try_coerce(self) -> Result<i32, CoerceError> {
                self.try_into().map_err(|_| CoerceError::Overflow)
            }
        }
    };
}

impl_try_i32!(u32);
impl_try_i32!(u64);
impl_try_i32!(usize);
impl_try_i32!(i64);
impl_try_i32!(isize);

// =============================================================================
// Narrowing to u8 (fallible)
// =============================================================================

macro_rules! impl_try_u8 {
    ($t:ty) => {
        impl TryCoerce<u8> for $t {
            type Error = CoerceError;
            #[inline]
            fn try_coerce(self) -> Result<u8, CoerceError> {
                self.try_into().map_err(|_| CoerceError::Overflow)
            }
        }
    };
}

impl_try_u8!(i8);
impl_try_u8!(i16);
impl_try_u8!(i32);
impl_try_u8!(i64);
impl_try_u8!(u16);
impl_try_u8!(u32);
impl_try_u8!(u64);
impl_try_u8!(usize);
impl_try_u8!(isize);

// =============================================================================
// Widening to u16/i16/u32 (infallible)
// =============================================================================

impl Coerce<u16> for u8 {
    #[inline(always)]
    fn coerce(self) -> u16 {
        self.into()
    }
}

impl Coerce<i16> for i8 {
    #[inline(always)]
    fn coerce(self) -> i16 {
        self.into()
    }
}

impl Coerce<i16> for u8 {
    #[inline(always)]
    fn coerce(self) -> i16 {
        self.into()
    }
}

impl Coerce<u32> for u8 {
    #[inline(always)]
    fn coerce(self) -> u32 {
        self.into()
    }
}

impl Coerce<u32> for u16 {
    #[inline(always)]
    fn coerce(self) -> u32 {
        self.into()
    }
}

// =============================================================================
// Narrowing to u16 (fallible)
// =============================================================================

macro_rules! impl_try_u16 {
    ($t:ty) => {
        impl TryCoerce<u16> for $t {
            type Error = CoerceError;
            #[inline]
            fn try_coerce(self) -> Result<u16, CoerceError> {
                self.try_into().map_err(|_| CoerceError::Overflow)
            }
        }
    };
}

impl_try_u16!(i8);
impl_try_u16!(i16);
impl_try_u16!(i32);
impl_try_u16!(i64);
impl_try_u16!(u32);
impl_try_u16!(u64);
impl_try_u16!(usize);
impl_try_u16!(isize);

// =============================================================================
// Narrowing to i16 (fallible)
// =============================================================================

macro_rules! impl_try_i16 {
    ($t:ty) => {
        impl TryCoerce<i16> for $t {
            type Error = CoerceError;
            #[inline]
            fn try_coerce(self) -> Result<i16, CoerceError> {
                self.try_into().map_err(|_| CoerceError::Overflow)
            }
        }
    };
}

impl_try_i16!(i32);
impl_try_i16!(i64);
impl_try_i16!(u16);
impl_try_i16!(u32);
impl_try_i16!(u64);
impl_try_i16!(usize);
impl_try_i16!(isize);

// =============================================================================
// Narrowing to i8 (fallible)
// =============================================================================

macro_rules! impl_try_i8 {
    ($t:ty) => {
        impl TryCoerce<i8> for $t {
            type Error = CoerceError;
            #[inline]
            fn try_coerce(self) -> Result<i8, CoerceError> {
                self.try_into().map_err(|_| CoerceError::Overflow)
            }
        }
    };
}

impl_try_i8!(i16);
impl_try_i8!(i32);
impl_try_i8!(i64);
impl_try_i8!(u8);
impl_try_i8!(u16);
impl_try_i8!(u32);
impl_try_i8!(u64);
impl_try_i8!(usize);
impl_try_i8!(isize);

// =============================================================================
// Float to smaller integers (fallible)
// =============================================================================

impl TryCoerce<u16> for f64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<u16, CoerceError> {
        if self.is_nan() {
            return Err(CoerceError::NaN);
        }
        if self < 0.0 || self > u16::MAX as f64 {
            return Err(CoerceError::Overflow);
        }
        if self.fract() != 0.0 {
            return Err(CoerceError::PrecisionLoss);
        }
        Ok(self as u16)
    }
}

impl TryCoerce<i16> for f64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<i16, CoerceError> {
        if self.is_nan() {
            return Err(CoerceError::NaN);
        }
        if self < i16::MIN as f64 || self > i16::MAX as f64 {
            return Err(CoerceError::Overflow);
        }
        if self.fract() != 0.0 {
            return Err(CoerceError::PrecisionLoss);
        }
        Ok(self as i16)
    }
}

impl TryCoerce<i8> for f64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<i8, CoerceError> {
        if self.is_nan() {
            return Err(CoerceError::NaN);
        }
        if self < i8::MIN as f64 || self > i8::MAX as f64 {
            return Err(CoerceError::Overflow);
        }
        if self.fract() != 0.0 {
            return Err(CoerceError::PrecisionLoss);
        }
        Ok(self as i8)
    }
}

// =============================================================================
// Float to i32 (fallible)
// =============================================================================

impl TryCoerce<i32> for f64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<i32, CoerceError> {
        if self.is_nan() {
            return Err(CoerceError::NaN);
        }
        if self < i32::MIN as f64 || self > i32::MAX as f64 {
            return Err(CoerceError::Overflow);
        }
        if self.fract() != 0.0 {
            return Err(CoerceError::PrecisionLoss);
        }
        Ok(self as i32)
    }
}

impl TryCoerce<i32> for f32 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<i32, CoerceError> {
        (self as f64).try_coerce()
    }
}

// =============================================================================
// Large int to f64 (fallible - precision)
// =============================================================================

impl TryCoerce<f64> for i64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<f64, CoerceError> {
        const MAX_SAFE: i64 = 1 << 53;
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
    fn try_coerce(self) -> Result<f64, CoerceError> {
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
    fn try_coerce(self) -> Result<f64, CoerceError> {
        (self as i64).try_coerce()
    }
}

impl TryCoerce<f64> for usize {
    type Error = CoerceError;
    #[inline]
    fn try_coerce(self) -> Result<f64, CoerceError> {
        (self as u64).try_coerce()
    }
}

// =============================================================================
// Slice coercions (element-wise)
// =============================================================================

/// Coerce a slice element-wise to a Vec.
impl<T: Copy + Coerce<R>, R> Coerce<Vec<R>> for &[T] {
    #[inline]
    fn coerce(self) -> Vec<R> {
        self.iter().copied().map(Coerce::coerce).collect()
    }
}

/// Coerce a Vec element-wise to a new Vec.
impl<T: Coerce<R>, R> Coerce<Vec<R>> for Vec<T> {
    #[inline]
    fn coerce(self) -> Vec<R> {
        self.into_iter().map(Coerce::coerce).collect()
    }
}

// Note: TryCoerce<Vec<R>> is automatically provided by the blanket impl
// when T: Coerce<R>. For types that only implement TryCoerce (not Coerce),
// use: slice.iter().map(|x| x.try_coerce()).collect::<Result<Vec<_>, _>>()

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        assert_eq!(Coerce::<i32>::coerce(42i32), 42i32);
        assert_eq!(Coerce::<f64>::coerce(3.14f64), 3.14f64);
    }

    #[test]
    fn test_widening() {
        let x: i32 = 42i8.coerce();
        assert_eq!(x, 42);

        let y: f64 = 42i32.coerce();
        assert_eq!(y, 42.0);
    }

    #[test]
    fn test_bool() {
        assert_eq!(Coerce::<Rboolean>::coerce(true), Rboolean::TRUE);
        assert_eq!(Coerce::<i32>::coerce(true), 1);
        assert_eq!(Coerce::<f64>::coerce(false), 0.0);
    }

    fn takes_can_coerce<T: CanCoerceToInteger>(x: T) -> i32 {
        x.coerce()
    }

    #[test]
    fn test_trait_bound() {
        assert_eq!(takes_can_coerce(42i8), 42);
        assert_eq!(takes_can_coerce(true), 1);
    }

    #[test]
    fn test_try_coerce() {
        assert_eq!(TryCoerce::<i32>::try_coerce(42u32), Ok(42));
        assert_eq!(TryCoerce::<i32>::try_coerce(u32::MAX), Err(CoerceError::Overflow));
    }

    #[test]
    fn test_f64_to_i32() {
        assert_eq!(TryCoerce::<i32>::try_coerce(42.0f64), Ok(42));
        assert_eq!(TryCoerce::<i32>::try_coerce(42.5f64), Err(CoerceError::PrecisionLoss));
        assert_eq!(TryCoerce::<i32>::try_coerce(f64::NAN), Err(CoerceError::NaN));
    }

    #[test]
    fn test_i64_to_f64() {
        assert_eq!(TryCoerce::<f64>::try_coerce(1000i64), Ok(1000.0));
        assert_eq!(TryCoerce::<f64>::try_coerce(i64::MAX), Err(CoerceError::PrecisionLoss));
    }

    #[test]
    fn test_slice_coerce() {
        let slice: &[i8] = &[1, 2, 3];
        let vec: Vec<i32> = slice.coerce();
        assert_eq!(vec, vec![1i32, 2, 3]);
    }

    #[test]
    fn test_vec_coerce() {
        let v: Vec<i16> = vec![10, 20, 30];
        let result: Vec<f64> = v.coerce();
        assert_eq!(result, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_slice_try_coerce_via_blanket() {
        // When element type has Coerce, TryCoerce is provided via blanket impl
        let slice: &[i8] = &[1, 2, 3];
        let result: Result<Vec<i32>, _> = slice.try_coerce();
        assert_eq!(result, Ok(vec![1, 2, 3]));
    }

    #[test]
    fn test_fallible_slice_coerce_manual() {
        // For types with only TryCoerce (not Coerce), use manual iteration
        let slice: &[u32] = &[1, u32::MAX, 3];
        let result: Result<Vec<i32>, _> = slice
            .iter()
            .copied()
            .map(TryCoerce::try_coerce)
            .collect();
        assert_eq!(result, Err(CoerceError::Overflow));
    }

    #[test]
    fn test_i32_to_u16() {
        // Success case
        assert_eq!(TryCoerce::<u16>::try_coerce(1000i32), Ok(1000u16));
        // Overflow - negative
        assert_eq!(TryCoerce::<u16>::try_coerce(-1i32), Err(CoerceError::Overflow));
        // Overflow - too large
        assert_eq!(TryCoerce::<u16>::try_coerce(70000i32), Err(CoerceError::Overflow));
    }

    #[test]
    fn test_i32_slice_to_u16_vec() {
        // Using manual iteration for fallible element-wise coercion
        let slice: &[i32] = &[1, 100, 1000];
        let result: Result<Vec<u16>, _> = slice
            .iter()
            .copied()
            .map(TryCoerce::try_coerce)
            .collect();
        assert_eq!(result, Ok(vec![1u16, 100, 1000]));

        // Failure case
        let slice2: &[i32] = &[1, -5, 1000];
        let result2: Result<Vec<u16>, _> = slice2
            .iter()
            .copied()
            .map(TryCoerce::try_coerce)
            .collect();
        assert_eq!(result2, Err(CoerceError::Overflow));
    }

    #[test]
    fn test_widening_to_u16() {
        let x: u16 = 42u8.coerce();
        assert_eq!(x, 42u16);
    }
}
