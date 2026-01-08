//! Type coercion traits for converting Rust types to R native types.
//!
//! R has a fixed set of native scalar types:
//! - `i32` (INTSXP) - 32-bit signed integer
//! - `f64` (REALSXP) - 64-bit floating point
//! - `RLogical` (LGLSXP) - logical (TRUE/FALSE/NA)
//! - `u8` (RAWSXP) - raw bytes
//! - `Rcomplex` (CPLXSXP) - complex numbers
//!
//! # Traits
//!
//! - [`Coerce<R>`] - infallible coercion (identity, widening)
//! - [`TryCoerce<R>`] - fallible coercion (narrowing, overflow-possible)
//!
//! # Examples
//!
//! ```ignore
//! use miniextendr_api::coerce::Coerce;
//!
//! // Scalar coercion
//! let x: i32 = 42i8.coerce();
//!
//! // Element-wise slice coercion
//! let slice: &[i8] = &[1, 2, 3];
//! let vec: Vec<i32> = slice.coerce();
//! ```

use crate::altrep_traits::{NA_INTEGER, NA_LOGICAL, NA_REAL};
use crate::ffi::{Rboolean, Rcomplex};

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
    Zero,
}

impl std::fmt::Display for CoerceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoerceError::Overflow => write!(f, "value out of range"),
            CoerceError::PrecisionLoss => write!(f, "precision loss"),
            CoerceError::NaN => write!(f, "NaN cannot be converted"),
            CoerceError::Zero => write!(f, "zero not allowed"),
        }
    }
}

impl std::error::Error for CoerceError {}

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
// Widening from u8 to larger integer/float types
// =============================================================================

impl Coerce<i64> for u8 {
    #[inline(always)]
    fn coerce(self) -> i64 {
        self.into()
    }
}

impl Coerce<isize> for u8 {
    #[inline(always)]
    fn coerce(self) -> isize {
        self.into()
    }
}

impl Coerce<u64> for u8 {
    #[inline(always)]
    fn coerce(self) -> u64 {
        self.into()
    }
}

impl Coerce<usize> for u8 {
    #[inline(always)]
    fn coerce(self) -> usize {
        self.into()
    }
}

impl Coerce<f32> for u8 {
    #[inline(always)]
    fn coerce(self) -> f32 {
        self.into()
    }
}

impl Coerce<f32> for i32 {
    #[inline(always)]
    fn coerce(self) -> f32 {
        self as f32
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
        if self { 1 } else { 0 }
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
// Option<T> to R-native with None → NA
// =============================================================================

/// `Option<f64>` → `f64` with `None` → `NA_real_`.
impl Coerce<f64> for Option<f64> {
    #[inline(always)]
    fn coerce(self) -> f64 {
        self.unwrap_or(NA_REAL)
    }
}

/// `Option<i32>` → `i32` with `None` → `NA_integer_`.
impl Coerce<i32> for Option<i32> {
    #[inline(always)]
    fn coerce(self) -> i32 {
        self.unwrap_or(NA_INTEGER)
    }
}

/// `Option<bool>` → `i32` with `None` → `NA_LOGICAL`.
impl Coerce<i32> for Option<bool> {
    #[inline(always)]
    fn coerce(self) -> i32 {
        match self {
            Some(true) => 1,
            Some(false) => 0,
            None => NA_LOGICAL,
        }
    }
}

/// `Option<Rboolean>` → `i32` with `None` → `NA_LOGICAL`.
impl Coerce<i32> for Option<Rboolean> {
    #[inline(always)]
    fn coerce(self) -> i32 {
        match self {
            Some(v) => v as i32,
            None => NA_LOGICAL,
        }
    }
}

// =============================================================================
// i32 to larger/unsigned types (for argument coercion from R integers)
// =============================================================================

/// i32 -> i64: widening, always safe
impl Coerce<i64> for i32 {
    #[inline(always)]
    fn coerce(self) -> i64 {
        self.into()
    }
}

/// i32 -> isize: always safe (isize is at least 32 bits)
impl Coerce<isize> for i32 {
    #[inline(always)]
    fn coerce(self) -> isize {
        self as isize
    }
}

/// i32 -> u32: can fail if negative
impl TryCoerce<u32> for i32 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<u32, CoerceError> {
        self.try_into().map_err(|_| CoerceError::Overflow)
    }
}

/// i32 -> u64: can fail if negative
impl TryCoerce<u64> for i32 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<u64, CoerceError> {
        self.try_into().map_err(|_| CoerceError::Overflow)
    }
}

/// i32 -> usize: can fail if negative
impl TryCoerce<usize> for i32 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<usize, CoerceError> {
        self.try_into().map_err(|_| CoerceError::Overflow)
    }
}

// =============================================================================
// NonZero conversions (fallible - zero check)
// =============================================================================

use core::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroIsize, NonZeroU8, NonZeroU16, NonZeroU32,
    NonZeroU64, NonZeroUsize,
};

macro_rules! impl_nonzero_from_self {
    ($base:ty, $nz:ty) => {
        impl TryCoerce<$nz> for $base {
            type Error = CoerceError;

            #[inline]
            fn try_coerce(self) -> Result<$nz, CoerceError> {
                <$nz>::new(self).ok_or(CoerceError::Zero)
            }
        }
    };
}

// Direct NonZero conversions (same base type)
impl_nonzero_from_self!(i8, NonZeroI8);
impl_nonzero_from_self!(i16, NonZeroI16);
impl_nonzero_from_self!(i32, NonZeroI32);
impl_nonzero_from_self!(i64, NonZeroI64);
impl_nonzero_from_self!(isize, NonZeroIsize);
impl_nonzero_from_self!(u8, NonZeroU8);
impl_nonzero_from_self!(u16, NonZeroU16);
impl_nonzero_from_self!(u32, NonZeroU32);
impl_nonzero_from_self!(u64, NonZeroU64);
impl_nonzero_from_self!(usize, NonZeroUsize);

/// i32 -> NonZeroI64: widen then check zero
impl TryCoerce<NonZeroI64> for i32 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<NonZeroI64, CoerceError> {
        NonZeroI64::new(self.into()).ok_or(CoerceError::Zero)
    }
}

/// i32 -> NonZeroIsize: widen then check zero
impl TryCoerce<NonZeroIsize> for i32 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<NonZeroIsize, CoerceError> {
        NonZeroIsize::new(self as isize).ok_or(CoerceError::Zero)
    }
}

/// i32 -> NonZeroU32: check non-negative and non-zero
impl TryCoerce<NonZeroU32> for i32 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<NonZeroU32, CoerceError> {
        let u: u32 = self.try_into().map_err(|_| CoerceError::Overflow)?;
        NonZeroU32::new(u).ok_or(CoerceError::Zero)
    }
}

/// i32 -> NonZeroU64: check non-negative and non-zero
impl TryCoerce<NonZeroU64> for i32 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<NonZeroU64, CoerceError> {
        let u: u64 = self.try_into().map_err(|_| CoerceError::Overflow)?;
        NonZeroU64::new(u).ok_or(CoerceError::Zero)
    }
}

/// i32 -> NonZeroUsize: check non-negative and non-zero
impl TryCoerce<NonZeroUsize> for i32 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<NonZeroUsize, CoerceError> {
        let u: usize = self.try_into().map_err(|_| CoerceError::Overflow)?;
        NonZeroUsize::new(u).ok_or(CoerceError::Zero)
    }
}

/// i32 -> NonZeroI8: narrow then check zero
impl TryCoerce<NonZeroI8> for i32 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<NonZeroI8, CoerceError> {
        let n: i8 = self.try_into().map_err(|_| CoerceError::Overflow)?;
        NonZeroI8::new(n).ok_or(CoerceError::Zero)
    }
}

/// i32 -> NonZeroI16: narrow then check zero
impl TryCoerce<NonZeroI16> for i32 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<NonZeroI16, CoerceError> {
        let n: i16 = self.try_into().map_err(|_| CoerceError::Overflow)?;
        NonZeroI16::new(n).ok_or(CoerceError::Zero)
    }
}

/// i32 -> NonZeroU8: check non-negative, narrow, then check zero
impl TryCoerce<NonZeroU8> for i32 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<NonZeroU8, CoerceError> {
        let u: u8 = self.try_into().map_err(|_| CoerceError::Overflow)?;
        NonZeroU8::new(u).ok_or(CoerceError::Zero)
    }
}

/// i32 -> NonZeroU16: check non-negative, narrow, then check zero
impl TryCoerce<NonZeroU16> for i32 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<NonZeroU16, CoerceError> {
        let u: u16 = self.try_into().map_err(|_| CoerceError::Overflow)?;
        NonZeroU16::new(u).ok_or(CoerceError::Zero)
    }
}

// =============================================================================
// i32/Rboolean to bool (fallible - NA handling)
// =============================================================================

/// Error type for logical coercion failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalCoerceError {
    /// R's NA_LOGICAL cannot be represented as Rust bool
    NAValue,
    /// Value is not 0 or 1
    InvalidValue(i32),
}

impl std::fmt::Display for LogicalCoerceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicalCoerceError::NAValue => write!(f, "NA cannot be converted to bool"),
            LogicalCoerceError::InvalidValue(v) => write!(f, "invalid logical value: {}", v),
        }
    }
}

impl std::error::Error for LogicalCoerceError {}

impl TryCoerce<bool> for i32 {
    type Error = LogicalCoerceError;

    #[inline]
    fn try_coerce(self) -> Result<bool, LogicalCoerceError> {
        match self {
            0 => Ok(false),
            1 => Ok(true),
            // NA_LOGICAL is i32::MIN in R
            i32::MIN => Err(LogicalCoerceError::NAValue),
            other => Err(LogicalCoerceError::InvalidValue(other)),
        }
    }
}

impl TryCoerce<bool> for Rboolean {
    type Error = LogicalCoerceError;

    #[inline]
    fn try_coerce(self) -> Result<bool, LogicalCoerceError> {
        (self as i32).try_coerce()
    }
}

impl TryCoerce<bool> for crate::ffi::RLogical {
    type Error = LogicalCoerceError;

    #[inline]
    fn try_coerce(self) -> Result<bool, LogicalCoerceError> {
        self.to_i32().try_coerce()
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
        if self.is_infinite() {
            return Err(CoerceError::Overflow);
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
        if self.is_infinite() {
            return Err(CoerceError::Overflow);
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
        if self.is_infinite() {
            return Err(CoerceError::Overflow);
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
        if self.is_infinite() {
            return Err(CoerceError::Overflow);
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
        f64::from(self).try_coerce()
    }
}

// f64 → f32 narrowing (always succeeds, may lose precision or become inf)
impl Coerce<f32> for f64 {
    #[inline(always)]
    fn coerce(self) -> f32 {
        self as f32
    }
}

// =============================================================================
// Float to u8 (fallible) - for RAWSXP
// =============================================================================

impl TryCoerce<u8> for f64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<u8, CoerceError> {
        if self.is_nan() {
            return Err(CoerceError::NaN);
        }
        if self.is_infinite() {
            return Err(CoerceError::Overflow);
        }
        if self < 0.0 || self > u8::MAX as f64 {
            return Err(CoerceError::Overflow);
        }
        if self.fract() != 0.0 {
            return Err(CoerceError::PrecisionLoss);
        }
        Ok(self as u8)
    }
}

impl TryCoerce<u8> for f32 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<u8, CoerceError> {
        f64::from(self).try_coerce()
    }
}

// =============================================================================
// Float to u32 (fallible)
// =============================================================================

impl TryCoerce<u32> for f64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<u32, CoerceError> {
        if self.is_nan() {
            return Err(CoerceError::NaN);
        }
        if self.is_infinite() {
            return Err(CoerceError::Overflow);
        }
        if self < 0.0 || self > u32::MAX as f64 {
            return Err(CoerceError::Overflow);
        }
        if self.fract() != 0.0 {
            return Err(CoerceError::PrecisionLoss);
        }
        Ok(self as u32)
    }
}

// =============================================================================
// Float to i64/u64 (fallible)
// =============================================================================

impl TryCoerce<i64> for f64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<i64, CoerceError> {
        if self.is_nan() {
            return Err(CoerceError::NaN);
        }
        if self.is_infinite() {
            return Err(CoerceError::Overflow);
        }
        // i64::MIN/MAX can't be exactly represented in f64, so use safe bounds
        if self < i64::MIN as f64 || self >= i64::MAX as f64 {
            return Err(CoerceError::Overflow);
        }
        if self.fract() != 0.0 {
            return Err(CoerceError::PrecisionLoss);
        }
        Ok(self as i64)
    }
}

impl TryCoerce<u64> for f64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<u64, CoerceError> {
        if self.is_nan() {
            return Err(CoerceError::NaN);
        }
        if self.is_infinite() {
            return Err(CoerceError::Overflow);
        }
        if self < 0.0 || self >= u64::MAX as f64 {
            return Err(CoerceError::Overflow);
        }
        if self.fract() != 0.0 {
            return Err(CoerceError::PrecisionLoss);
        }
        Ok(self as u64)
    }
}

impl TryCoerce<isize> for f64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<isize, CoerceError> {
        if self.is_nan() {
            return Err(CoerceError::NaN);
        }
        if self.is_infinite() {
            return Err(CoerceError::Overflow);
        }
        if self < isize::MIN as f64 || self > isize::MAX as f64 {
            return Err(CoerceError::Overflow);
        }
        if self.fract() != 0.0 {
            return Err(CoerceError::PrecisionLoss);
        }
        Ok(self as isize)
    }
}

impl TryCoerce<usize> for f64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<usize, CoerceError> {
        if self.is_nan() {
            return Err(CoerceError::NaN);
        }
        if self.is_infinite() {
            return Err(CoerceError::Overflow);
        }
        if self < 0.0 || self > usize::MAX as f64 {
            return Err(CoerceError::Overflow);
        }
        if self.fract() != 0.0 {
            return Err(CoerceError::PrecisionLoss);
        }
        Ok(self as usize)
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
// Coerced wrapper type
// =============================================================================

use std::marker::PhantomData;

/// Wrapper for values coerced from an R native type during conversion.
///
/// This enables using non-native Rust types in collections read from R:
///
/// ```ignore
/// // Read a Vec of i64 from R integers (i32)
/// let vec: Vec<Coerced<i64, i32>> = TryFromSexp::try_from_sexp(sexp)?;
///
/// // Extract the values
/// let i64_vec: Vec<i64> = vec.into_iter().map(Coerced::into_inner).collect();
/// ```
///
/// The type parameters are:
/// - `T`: The target Rust type you want
/// - `R`: The R-native type to read and coerce from
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Coerced<T, R> {
    value: T,
    _marker: PhantomData<R>,
}

impl<T, R> Coerced<T, R> {
    /// Create a new Coerced wrapper.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    /// Extract the inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Get a reference to the inner value.
    #[inline]
    pub const fn as_inner(&self) -> &T {
        &self.value
    }

    /// Get a mutable reference to the inner value.
    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T, R> std::ops::Deref for Coerced<T, R> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, R> std::ops::DerefMut for Coerced<T, R> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
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
// Tuple coercions (element-wise)
// =============================================================================

/// Macro to implement element-wise Coerce for tuples.
macro_rules! impl_tuple_coerce {
    (($($T:ident),+), ($($R:ident),+), ($($idx:tt),+)) => {
        impl<$($T,)+ $($R,)+> Coerce<($($R,)+)> for ($($T,)+)
        where
            $($T: Coerce<$R>,)+
        {
            #[inline]
            fn coerce(self) -> ($($R,)+) {
                ($(Coerce::<$R>::coerce(self.$idx),)+)
            }
        }
    };
}

// Implement for tuples of sizes 2-8
impl_tuple_coerce!((A, B), (RA, RB), (0, 1));
impl_tuple_coerce!((A, B, C), (RA, RB, RC), (0, 1, 2));
impl_tuple_coerce!((A, B, C, D), (RA, RB, RC, RD), (0, 1, 2, 3));
impl_tuple_coerce!((A, B, C, D, E), (RA, RB, RC, RD, RE), (0, 1, 2, 3, 4));
impl_tuple_coerce!(
    (A, B, C, D, E, F),
    (RA, RB, RC, RD, RE, RF),
    (0, 1, 2, 3, 4, 5)
);
impl_tuple_coerce!(
    (A, B, C, D, E, F, G),
    (RA, RB, RC, RD, RE, RF, RG),
    (0, 1, 2, 3, 4, 5, 6)
);
impl_tuple_coerce!(
    (A, B, C, D, E, F, G, H),
    (RA, RB, RC, RD, RE, RF, RG, RH),
    (0, 1, 2, 3, 4, 5, 6, 7)
);

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests;
