//! Storage-directed conversion to R.
//!
//! This module provides [`IntoRAs`], a trait for converting Rust values to R SEXPs
//! with explicit target storage type selection.
//!
//! # Value-Based Semantics
//!
//! Conversions are **runtime-checked**: if the actual value fits the target type,
//! it converts; if not, it errors. There is no "lossy" escape hatch - if you want
//! lossy conversion, cast the values yourself first.
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::IntoRAs;
//!
//! // These succeed (values fit)
//! let x = vec![1_i64, 2, 3];
//! let sexp = x.into_r_as::<i32>()?;           // OK: all values in i32 range
//!
//! let y = vec![1.0_f64, 2.0, 3.0];
//! let sexp = y.into_r_as::<i32>()?;           // OK: all values are integral
//!
//! // These fail (values don't fit)
//! let z = vec![1_i64 << 40];
//! let sexp = z.into_r_as::<i32>()?;           // Error: out of range
//!
//! let w = vec![1.5_f64];
//! let sexp = w.into_r_as::<i32>()?;           // Error: not integral
//!
//! // User wants lossy? Cast first.
//! let lossy: Vec<i32> = vec![1.5_f64, 2.7].iter().map(|&x| x as i32).collect();
//! let sexp = lossy.into_r();                  // [1, 2] - user's responsibility
//! ```

use crate::coerce::{CoerceError, TryCoerce};
use crate::ffi::{RLogical, SEXP};
use crate::into_r::IntoR;
use std::fmt;

// =============================================================================
// Error type
// =============================================================================

/// Error type for storage-directed conversion failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageCoerceError {
    /// Conversion between these types is not supported.
    Unsupported {
        from: &'static str,
        to: &'static str,
    },
    /// Value is out of range for the target type.
    OutOfRange {
        from: &'static str,
        to: &'static str,
        index: Option<usize>,
    },
    /// Value is non-finite (NaN or Inf) but target requires finite.
    NonFinite {
        to: &'static str,
        index: Option<usize>,
    },
    /// Conversion would lose precision.
    PrecisionLoss {
        to: &'static str,
        index: Option<usize>,
    },
    /// Float value is not integral but target is integer type.
    NotIntegral {
        to: &'static str,
        index: Option<usize>,
    },
    /// Missing value (NA) cannot be represented in target type.
    MissingValue {
        to: &'static str,
        index: Option<usize>,
    },
    /// Invalid UTF-8 in string conversion.
    InvalidUtf8 { index: Option<usize> },
}

impl fmt::Display for StorageCoerceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageCoerceError::Unsupported { from, to } => {
                write!(f, "cannot convert {} to {}", from, to)
            }
            StorageCoerceError::OutOfRange { from, to, index } => {
                if let Some(i) = index {
                    write!(f, "value at index {} out of range for {} → {}", i, from, to)
                } else {
                    write!(f, "value out of range for {} → {}", from, to)
                }
            }
            StorageCoerceError::NonFinite { to, index } => {
                if let Some(i) = index {
                    write!(
                        f,
                        "non-finite value at index {} cannot convert to {}",
                        i, to
                    )
                } else {
                    write!(f, "non-finite value cannot convert to {}", to)
                }
            }
            StorageCoerceError::PrecisionLoss { to, index } => {
                if let Some(i) = index {
                    write!(
                        f,
                        "value at index {} would lose precision converting to {}",
                        i, to
                    )
                } else {
                    write!(f, "value would lose precision converting to {}", to)
                }
            }
            StorageCoerceError::NotIntegral { to, index } => {
                if let Some(i) = index {
                    write!(
                        f,
                        "non-integral value at index {} cannot convert to {}",
                        i, to
                    )
                } else {
                    write!(f, "non-integral value cannot convert to {}", to)
                }
            }
            StorageCoerceError::MissingValue { to, index } => {
                if let Some(i) = index {
                    write!(f, "missing value at index {} cannot convert to {}", i, to)
                } else {
                    write!(f, "missing value cannot convert to {}", to)
                }
            }
            StorageCoerceError::InvalidUtf8 { index } => {
                if let Some(i) = index {
                    write!(f, "invalid UTF-8 at index {}", i)
                } else {
                    write!(f, "invalid UTF-8")
                }
            }
        }
    }
}

impl std::error::Error for StorageCoerceError {}

impl StorageCoerceError {
    /// Add index information to the error.
    #[inline]
    pub fn at_index(self, idx: usize) -> Self {
        match self {
            StorageCoerceError::OutOfRange { from, to, .. } => StorageCoerceError::OutOfRange {
                from,
                to,
                index: Some(idx),
            },
            StorageCoerceError::NonFinite { to, .. } => StorageCoerceError::NonFinite {
                to,
                index: Some(idx),
            },
            StorageCoerceError::PrecisionLoss { to, .. } => StorageCoerceError::PrecisionLoss {
                to,
                index: Some(idx),
            },
            StorageCoerceError::NotIntegral { to, .. } => StorageCoerceError::NotIntegral {
                to,
                index: Some(idx),
            },
            StorageCoerceError::MissingValue { to, .. } => StorageCoerceError::MissingValue {
                to,
                index: Some(idx),
            },
            StorageCoerceError::InvalidUtf8 { .. } => {
                StorageCoerceError::InvalidUtf8 { index: Some(idx) }
            }
            other => other,
        }
    }
}

// =============================================================================
// Trait definition
// =============================================================================

/// Storage-directed conversion to R SEXP.
///
/// This trait allows converting Rust values to R with an explicit target storage
/// type. The conversion is value-based: it succeeds if all values fit the target
/// type, and fails otherwise.
///
/// # Target Types
///
/// - `i32` → R integer (INTSXP)
/// - `f64` → R numeric (REALSXP)
/// - `RLogical` → R logical (LGLSXP)
/// - `u8` → R raw (RAWSXP)
/// - `String` → R character (STRSXP)
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::IntoRAs;
///
/// // Convert i64 to R integer (if values fit)
/// let x: Vec<i64> = vec![1, 2, 3];
/// let sexp = x.into_r_as::<i32>()?;
///
/// // Convert f64 to R integer (if values are integral)
/// let y: Vec<f64> = vec![1.0, 2.0, 3.0];
/// let sexp = y.into_r_as::<i32>()?;
/// ```
pub trait IntoRAs<Target> {
    /// Convert to R SEXP with the specified target storage type.
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError>;
}

// =============================================================================
// Helper: try_coerce_scalar with error mapping
// =============================================================================

/// Try to coerce a scalar value, mapping CoerceError to StorageCoerceError.
#[inline]
fn try_coerce_scalar<T, R>(
    value: T,
    from: &'static str,
    to: &'static str,
) -> Result<R, StorageCoerceError>
where
    T: TryCoerce<R>,
    T::Error: Into<CoerceErrorKind>,
{
    value
        .try_coerce()
        .map_err(|e| map_coerce_error(e.into(), from, to))
}

/// Internal enum to unify different coerce error types.
#[derive(Debug)]
enum CoerceErrorKind {
    Overflow,
    PrecisionLoss,
    NaN,
    Infallible,
}

impl From<CoerceError> for CoerceErrorKind {
    fn from(e: CoerceError) -> Self {
        match e {
            CoerceError::Overflow => CoerceErrorKind::Overflow,
            CoerceError::PrecisionLoss => CoerceErrorKind::PrecisionLoss,
            CoerceError::NaN => CoerceErrorKind::NaN,
            CoerceError::Zero => CoerceErrorKind::Overflow, // Treat zero error as overflow
        }
    }
}

impl From<std::convert::Infallible> for CoerceErrorKind {
    fn from(_: std::convert::Infallible) -> Self {
        CoerceErrorKind::Infallible
    }
}

fn map_coerce_error(
    kind: CoerceErrorKind,
    from: &'static str,
    to: &'static str,
) -> StorageCoerceError {
    match kind {
        CoerceErrorKind::Overflow => StorageCoerceError::OutOfRange {
            from,
            to,
            index: None,
        },
        CoerceErrorKind::PrecisionLoss => StorageCoerceError::PrecisionLoss { to, index: None },
        CoerceErrorKind::NaN => StorageCoerceError::NonFinite { to, index: None },
        CoerceErrorKind::Infallible => unreachable!(),
    }
}

// =============================================================================
// Scalar implementations: -> i32 (R integer)
// =============================================================================

macro_rules! impl_into_r_as_i32_scalar {
    ($from:ty, $from_name:literal) => {
        impl IntoRAs<i32> for $from {
            #[inline]
            fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
                let v: i32 = try_coerce_scalar(self, $from_name, "i32")?;
                Ok(v.into_sexp())
            }
        }
    };
}

// Identity
impl IntoRAs<i32> for i32 {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.into_sexp())
    }
}

// Widening (infallible)
impl_into_r_as_i32_scalar!(i8, "i8");
impl_into_r_as_i32_scalar!(i16, "i16");
impl_into_r_as_i32_scalar!(u8, "u8");
impl_into_r_as_i32_scalar!(u16, "u16");

// Narrowing (fallible)
impl_into_r_as_i32_scalar!(i64, "i64");
impl_into_r_as_i32_scalar!(isize, "isize");
impl_into_r_as_i32_scalar!(u32, "u32");
impl_into_r_as_i32_scalar!(u64, "u64");
impl_into_r_as_i32_scalar!(usize, "usize");

// Float to int (fallible - must be integral and in range)
impl_into_r_as_i32_scalar!(f32, "f32");
impl_into_r_as_i32_scalar!(f64, "f64");

// Bool to int
impl IntoRAs<i32> for bool {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok((self as i32).into_sexp())
    }
}

// =============================================================================
// Scalar implementations: -> f64 (R numeric)
// =============================================================================

macro_rules! impl_into_r_as_f64_scalar {
    ($from:ty, $from_name:literal) => {
        impl IntoRAs<f64> for $from {
            #[inline]
            fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
                let v: f64 = try_coerce_scalar(self, $from_name, "f64")?;
                Ok(v.into_sexp())
            }
        }
    };
}

// Identity
impl IntoRAs<f64> for f64 {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        if !self.is_finite() {
            return Err(StorageCoerceError::NonFinite {
                to: "f64",
                index: None,
            });
        }
        Ok(self.into_sexp())
    }
}

// Widening from f32 (check finite)
impl IntoRAs<f64> for f32 {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        if !self.is_finite() {
            return Err(StorageCoerceError::NonFinite {
                to: "f64",
                index: None,
            });
        }
        Ok((self as f64).into_sexp())
    }
}

// Widening from integers (infallible for small types)
impl IntoRAs<f64> for i8 {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok((self as f64).into_sexp())
    }
}

impl IntoRAs<f64> for i16 {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok((self as f64).into_sexp())
    }
}

impl IntoRAs<f64> for i32 {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok((self as f64).into_sexp())
    }
}

impl IntoRAs<f64> for u8 {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok((self as f64).into_sexp())
    }
}

impl IntoRAs<f64> for u16 {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok((self as f64).into_sexp())
    }
}

impl IntoRAs<f64> for u32 {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok((self as f64).into_sexp())
    }
}

// Large integers: check precision (> 2^53 loses precision)
impl_into_r_as_f64_scalar!(i64, "i64");
impl_into_r_as_f64_scalar!(u64, "u64");
impl_into_r_as_f64_scalar!(isize, "isize");
impl_into_r_as_f64_scalar!(usize, "usize");

// Bool to f64
impl IntoRAs<f64> for bool {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok((if self { 1.0 } else { 0.0 }).into_sexp())
    }
}

// =============================================================================
// Scalar implementations: -> u8 (R raw)
// =============================================================================

macro_rules! impl_into_r_as_u8_scalar {
    ($from:ty, $from_name:literal) => {
        impl IntoRAs<u8> for $from {
            #[inline]
            fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
                let v: u8 = try_coerce_scalar(self, $from_name, "u8")?;
                Ok(v.into_sexp())
            }
        }
    };
}

// Identity
impl IntoRAs<u8> for u8 {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.into_sexp())
    }
}

// Narrowing (fallible)
impl_into_r_as_u8_scalar!(i8, "i8");
impl_into_r_as_u8_scalar!(i16, "i16");
impl_into_r_as_u8_scalar!(i32, "i32");
impl_into_r_as_u8_scalar!(i64, "i64");
impl_into_r_as_u8_scalar!(isize, "isize");
impl_into_r_as_u8_scalar!(u16, "u16");
impl_into_r_as_u8_scalar!(u32, "u32");
impl_into_r_as_u8_scalar!(u64, "u64");
impl_into_r_as_u8_scalar!(usize, "usize");
impl_into_r_as_u8_scalar!(f32, "f32");
impl_into_r_as_u8_scalar!(f64, "f64");

// =============================================================================
// Scalar implementations: -> RLogical (R logical)
// =============================================================================

impl IntoRAs<RLogical> for bool {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.into_sexp())
    }
}

impl IntoRAs<RLogical> for RLogical {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.into_sexp())
    }
}

// Integer to logical: only 0, 1, NA_INTEGER allowed
impl IntoRAs<RLogical> for i32 {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        match self {
            0 => Ok(false.into_sexp()),
            1 => Ok(true.into_sexp()),
            crate::altrep_traits::NA_INTEGER => Ok(RLogical::NA.into_sexp()),
            _ => Err(StorageCoerceError::OutOfRange {
                from: "i32",
                to: "RLogical",
                index: None,
            }),
        }
    }
}

// =============================================================================
// Scalar implementations: -> String (R character)
// =============================================================================

impl IntoRAs<String> for String {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.into_sexp())
    }
}

impl IntoRAs<String> for &str {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.into_sexp())
    }
}

// Numeric to String: stringify (including NaN/Inf)
impl IntoRAs<String> for f64 {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        let s = if self.is_nan() {
            "NaN".to_string()
        } else if self.is_infinite() {
            if self.is_sign_positive() {
                "Inf".to_string()
            } else {
                "-Inf".to_string()
            }
        } else {
            self.to_string()
        };
        Ok(s.into_sexp())
    }
}

impl IntoRAs<String> for f32 {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        <f64 as IntoRAs<String>>::into_r_as(self as f64)
    }
}

impl IntoRAs<String> for i32 {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.to_string().into_sexp())
    }
}

impl IntoRAs<String> for i64 {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.to_string().into_sexp())
    }
}

impl IntoRAs<String> for bool {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok((if self { "TRUE" } else { "FALSE" }).into_sexp())
    }
}

impl IntoRAs<String> for RLogical {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        let s = match self.to_option_bool() {
            None => "NA",
            Some(true) => "TRUE",
            Some(false) => "FALSE",
        };
        Ok(s.into_sexp())
    }
}

// =============================================================================
// Vec implementations: -> i32 (R integer vector)
// =============================================================================

macro_rules! impl_vec_into_r_as_i32 {
    ($from:ty, $from_name:literal) => {
        impl IntoRAs<i32> for Vec<$from> {
            fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
                let mut result = Vec::with_capacity(self.len());
                for (i, val) in self.into_iter().enumerate() {
                    let v: i32 =
                        try_coerce_scalar(val, $from_name, "i32").map_err(|e| e.at_index(i))?;
                    result.push(v);
                }
                Ok(result.into_sexp())
            }
        }

        impl IntoRAs<i32> for &[$from] {
            fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
                let mut result = Vec::with_capacity(self.len());
                for (i, &val) in self.iter().enumerate() {
                    let v: i32 =
                        try_coerce_scalar(val, $from_name, "i32").map_err(|e| e.at_index(i))?;
                    result.push(v);
                }
                Ok(result.into_sexp())
            }
        }
    };
}

// Identity (direct copy)
impl IntoRAs<i32> for Vec<i32> {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.into_sexp())
    }
}

impl IntoRAs<i32> for &[i32] {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.into_sexp())
    }
}

impl_vec_into_r_as_i32!(i8, "i8");
impl_vec_into_r_as_i32!(i16, "i16");
impl_vec_into_r_as_i32!(u8, "u8");
impl_vec_into_r_as_i32!(u16, "u16");
impl_vec_into_r_as_i32!(i64, "i64");
impl_vec_into_r_as_i32!(isize, "isize");
impl_vec_into_r_as_i32!(u32, "u32");
impl_vec_into_r_as_i32!(u64, "u64");
impl_vec_into_r_as_i32!(usize, "usize");
impl_vec_into_r_as_i32!(f32, "f32");
impl_vec_into_r_as_i32!(f64, "f64");

// Vec<bool> -> i32
impl IntoRAs<i32> for Vec<bool> {
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        let result: Vec<i32> = self.into_iter().map(|b| b as i32).collect();
        Ok(result.into_sexp())
    }
}

impl IntoRAs<i32> for &[bool] {
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        let result: Vec<i32> = self.iter().map(|&b| b as i32).collect();
        Ok(result.into_sexp())
    }
}

// =============================================================================
// Vec implementations: -> f64 (R numeric vector)
// =============================================================================

macro_rules! impl_vec_into_r_as_f64 {
    ($from:ty, $from_name:literal) => {
        impl IntoRAs<f64> for Vec<$from> {
            fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
                let mut result = Vec::with_capacity(self.len());
                for (i, val) in self.into_iter().enumerate() {
                    let v: f64 =
                        try_coerce_scalar(val, $from_name, "f64").map_err(|e| e.at_index(i))?;
                    result.push(v);
                }
                Ok(result.into_sexp())
            }
        }

        impl IntoRAs<f64> for &[$from] {
            fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
                let mut result = Vec::with_capacity(self.len());
                for (i, &val) in self.iter().enumerate() {
                    let v: f64 =
                        try_coerce_scalar(val, $from_name, "f64").map_err(|e| e.at_index(i))?;
                    result.push(v);
                }
                Ok(result.into_sexp())
            }
        }
    };
}

// Identity - but check for finite values
impl IntoRAs<f64> for Vec<f64> {
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        for (i, &val) in self.iter().enumerate() {
            if !val.is_finite() {
                return Err(StorageCoerceError::NonFinite {
                    to: "f64",
                    index: Some(i),
                });
            }
        }
        Ok(self.into_sexp())
    }
}

impl IntoRAs<f64> for &[f64] {
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        for (i, &val) in self.iter().enumerate() {
            if !val.is_finite() {
                return Err(StorageCoerceError::NonFinite {
                    to: "f64",
                    index: Some(i),
                });
            }
        }
        Ok(self.into_sexp())
    }
}

// f32 - check finite then widen
impl IntoRAs<f64> for Vec<f32> {
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        let mut result = Vec::with_capacity(self.len());
        for (i, val) in self.into_iter().enumerate() {
            if !val.is_finite() {
                return Err(StorageCoerceError::NonFinite {
                    to: "f64",
                    index: Some(i),
                });
            }
            result.push(val as f64);
        }
        Ok(result.into_sexp())
    }
}

impl IntoRAs<f64> for &[f32] {
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        let mut result = Vec::with_capacity(self.len());
        for (i, &val) in self.iter().enumerate() {
            if !val.is_finite() {
                return Err(StorageCoerceError::NonFinite {
                    to: "f64",
                    index: Some(i),
                });
            }
            result.push(val as f64);
        }
        Ok(result.into_sexp())
    }
}

// Small integers - infallible widening
macro_rules! impl_vec_into_r_as_f64_infallible {
    ($from:ty) => {
        impl IntoRAs<f64> for Vec<$from> {
            fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
                let result: Vec<f64> = self.into_iter().map(|v| v as f64).collect();
                Ok(result.into_sexp())
            }
        }

        impl IntoRAs<f64> for &[$from] {
            fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
                let result: Vec<f64> = self.iter().map(|&v| v as f64).collect();
                Ok(result.into_sexp())
            }
        }
    };
}

impl_vec_into_r_as_f64_infallible!(i8);
impl_vec_into_r_as_f64_infallible!(i16);
impl_vec_into_r_as_f64_infallible!(i32);
impl_vec_into_r_as_f64_infallible!(u8);
impl_vec_into_r_as_f64_infallible!(u16);
impl_vec_into_r_as_f64_infallible!(u32);

// Large integers - check precision
impl_vec_into_r_as_f64!(i64, "i64");
impl_vec_into_r_as_f64!(u64, "u64");
impl_vec_into_r_as_f64!(isize, "isize");
impl_vec_into_r_as_f64!(usize, "usize");

// Vec<bool> -> f64
impl IntoRAs<f64> for Vec<bool> {
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        let result: Vec<f64> = self
            .into_iter()
            .map(|b| if b { 1.0 } else { 0.0 })
            .collect();
        Ok(result.into_sexp())
    }
}

impl IntoRAs<f64> for &[bool] {
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        let result: Vec<f64> = self.iter().map(|&b| if b { 1.0 } else { 0.0 }).collect();
        Ok(result.into_sexp())
    }
}

// =============================================================================
// Vec implementations: -> u8 (R raw vector)
// =============================================================================

macro_rules! impl_vec_into_r_as_u8 {
    ($from:ty, $from_name:literal) => {
        impl IntoRAs<u8> for Vec<$from> {
            fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
                let mut result = Vec::with_capacity(self.len());
                for (i, val) in self.into_iter().enumerate() {
                    let v: u8 =
                        try_coerce_scalar(val, $from_name, "u8").map_err(|e| e.at_index(i))?;
                    result.push(v);
                }
                Ok(result.into_sexp())
            }
        }

        impl IntoRAs<u8> for &[$from] {
            fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
                let mut result = Vec::with_capacity(self.len());
                for (i, &val) in self.iter().enumerate() {
                    let v: u8 =
                        try_coerce_scalar(val, $from_name, "u8").map_err(|e| e.at_index(i))?;
                    result.push(v);
                }
                Ok(result.into_sexp())
            }
        }
    };
}

// Identity
impl IntoRAs<u8> for Vec<u8> {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.into_sexp())
    }
}

impl IntoRAs<u8> for &[u8] {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.into_sexp())
    }
}

impl_vec_into_r_as_u8!(i8, "i8");
impl_vec_into_r_as_u8!(i16, "i16");
impl_vec_into_r_as_u8!(i32, "i32");
impl_vec_into_r_as_u8!(i64, "i64");
impl_vec_into_r_as_u8!(isize, "isize");
impl_vec_into_r_as_u8!(u16, "u16");
impl_vec_into_r_as_u8!(u32, "u32");
impl_vec_into_r_as_u8!(u64, "u64");
impl_vec_into_r_as_u8!(usize, "usize");
impl_vec_into_r_as_u8!(f32, "f32");
impl_vec_into_r_as_u8!(f64, "f64");

// =============================================================================
// Vec implementations: -> RLogical (R logical vector)
// =============================================================================

impl IntoRAs<RLogical> for Vec<bool> {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.into_sexp())
    }
}

impl IntoRAs<RLogical> for &[bool] {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.into_sexp())
    }
}

// =============================================================================
// Vec implementations: -> String (R character vector)
// =============================================================================

impl IntoRAs<String> for Vec<String> {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.into_sexp())
    }
}

impl IntoRAs<String> for &[String] {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.into_sexp())
    }
}

impl IntoRAs<String> for Vec<&str> {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.into_sexp())
    }
}

impl IntoRAs<String> for &[&str] {
    #[inline]
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        Ok(self.into_sexp())
    }
}

// Numeric vectors to String (stringify)
impl IntoRAs<String> for Vec<f64> {
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        let strings: Vec<String> = self
            .into_iter()
            .map(|v| {
                if v.is_nan() {
                    "NaN".to_string()
                } else if v.is_infinite() {
                    if v.is_sign_positive() {
                        "Inf".to_string()
                    } else {
                        "-Inf".to_string()
                    }
                } else {
                    v.to_string()
                }
            })
            .collect();
        Ok(strings.into_sexp())
    }
}

impl IntoRAs<String> for Vec<i32> {
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        let strings: Vec<String> = self.into_iter().map(|v| v.to_string()).collect();
        Ok(strings.into_sexp())
    }
}

impl IntoRAs<String> for Vec<i64> {
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        let strings: Vec<String> = self.into_iter().map(|v| v.to_string()).collect();
        Ok(strings.into_sexp())
    }
}

impl IntoRAs<String> for Vec<bool> {
    fn into_r_as(self) -> Result<SEXP, StorageCoerceError> {
        let strings: Vec<String> = self
            .into_iter()
            .map(|b| if b { "TRUE" } else { "FALSE" }.to_string())
            .collect();
        Ok(strings.into_sexp())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These are compile-time tests to ensure the trait is implemented correctly.
    // Runtime tests require R to be initialized and should go in the integration tests.

    fn _assert_into_r_as<T, Target>()
    where
        T: IntoRAs<Target>,
    {
    }

    #[test]
    fn test_trait_bounds() {
        // Scalars -> i32
        _assert_into_r_as::<i32, i32>();
        _assert_into_r_as::<i64, i32>();
        _assert_into_r_as::<f64, i32>();
        _assert_into_r_as::<bool, i32>();

        // Scalars -> f64
        _assert_into_r_as::<f64, f64>();
        _assert_into_r_as::<i32, f64>();
        _assert_into_r_as::<i64, f64>();
        _assert_into_r_as::<bool, f64>();

        // Scalars -> u8
        _assert_into_r_as::<u8, u8>();
        _assert_into_r_as::<i32, u8>();

        // Scalars -> RLogical
        _assert_into_r_as::<bool, RLogical>();
        _assert_into_r_as::<i32, RLogical>();

        // Scalars -> String
        _assert_into_r_as::<String, String>();
        _assert_into_r_as::<&str, String>();
        _assert_into_r_as::<f64, String>();
        _assert_into_r_as::<i32, String>();
        _assert_into_r_as::<bool, String>();

        // Vecs -> i32
        _assert_into_r_as::<Vec<i32>, i32>();
        _assert_into_r_as::<Vec<i64>, i32>();
        _assert_into_r_as::<Vec<f64>, i32>();

        // Vecs -> f64
        _assert_into_r_as::<Vec<f64>, f64>();
        _assert_into_r_as::<Vec<i32>, f64>();
        _assert_into_r_as::<Vec<i64>, f64>();

        // Vecs -> u8
        _assert_into_r_as::<Vec<u8>, u8>();
        _assert_into_r_as::<Vec<i32>, u8>();

        // Vecs -> String
        _assert_into_r_as::<Vec<String>, String>();
        _assert_into_r_as::<Vec<f64>, String>();
    }
}
