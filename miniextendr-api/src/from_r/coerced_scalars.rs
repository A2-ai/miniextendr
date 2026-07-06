//! Coerced scalar conversions (multi-source numeric) and large integer scalars.
//!
//! **The `SEXPTYPE` literals here are the source of truth (#882).** They are
//! *runtime* match arms on `sexp.type_of()`, deliberately accepting several
//! source types and coercing into one target Rust type — there is no single `T`
//! whose `T::SEXP_TYPE` they could be folded into (the whole point is the 1:N
//! input fan-in). Leave them.
//!
//! These types accept multiple R source types (INTSXP, REALSXP, RAWSXP, LGLSXP)
//! and coerce to the target Rust type via [`TryCoerce`].
//!
//! Covers: `i8`, `i16`, `u16`, `u32`, `f32` (sub-native scalars) and
//! `i64`, `u64`, `isize`, `usize` (large integers via f64 intermediary).
//!
//! # Tradeoff
//!
//! This is the **looser** inbound path. The strict alternative is the bare
//! [`TryFromSexp`] impl on the matching R native
//! type (`i32`, `f64`, `&[i32]`, …) — those reject any mismatched
//! [`SEXPTYPE`] outright instead of coercing. Failure mode of preferring the
//! coerced path when you wanted strict: an R caller silently passes `1.7`
//! (REALSXP) into a Rust `i32` argument and gets a truncated `1`.
//!
//! Outbound counterparts for the large-integer types in this module live in
//! `crate::into_r::large_integers` (lax, default) and [`crate::strict`]
//! (`#[miniextendr(strict)]` opt-in).

use crate::altrep_traits::NA_INTEGER;
use crate::coerce::TryCoerce;
use crate::from_r::{SexpError, SexpNaError, TryFromSexp, is_na_real};
use crate::{RLogical, SEXP, SEXPTYPE, SexpExt};

/// NA-rejecting error for a logical (`LGLSXP`) scalar that holds `NA`.
///
/// The bare integer/float scalar paths must reject `NA_logical_` rather than
/// silently coercing R's `NA_LOGICAL` sentinel (`i32::MIN`) into a finite
/// number. Mirrors the guard the bare `i32` (`INTSXP`) impl already applies.
#[inline]
fn lglsxp_na_error() -> SexpError {
    SexpNaError {
        sexp_type: SEXPTYPE::LGLSXP,
    }
    .into()
}

#[inline]
pub(crate) fn coerce_value<R, T>(value: R) -> Result<T, SexpError>
where
    R: TryCoerce<T>,
    <R as TryCoerce<T>>::Error: std::fmt::Debug,
{
    value
        .try_coerce()
        .map_err(|e| SexpError::InvalidValue(format!("{e:?}")))
}

#[inline]
fn try_from_sexp_numeric_scalar<T>(sexp: SEXP) -> Result<T, SexpError>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    u8: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Debug,
    <f64 as TryCoerce<T>>::Error: std::fmt::Debug,
    <u8 as TryCoerce<T>>::Error: std::fmt::Debug,
{
    let actual = sexp.type_of();
    match actual {
        SEXPTYPE::INTSXP => {
            let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
            coerce_value(value)
        }
        SEXPTYPE::REALSXP => {
            let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
            coerce_value(value)
        }
        SEXPTYPE::RAWSXP => {
            let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
            coerce_value(value)
        }
        SEXPTYPE::LGLSXP => {
            let value: RLogical = TryFromSexp::try_from_sexp(sexp)?;
            if value.is_na() {
                return Err(lglsxp_na_error());
            }
            coerce_value(value.to_i32())
        }
        _ => Err(SexpError::InvalidValue(format!(
            "expected integer, numeric, logical, or raw; got {:?}",
            actual
        ))),
    }
}

#[inline]
unsafe fn try_from_sexp_numeric_scalar_unchecked<T>(sexp: SEXP) -> Result<T, SexpError>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    u8: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Debug,
    <f64 as TryCoerce<T>>::Error: std::fmt::Debug,
    <u8 as TryCoerce<T>>::Error: std::fmt::Debug,
{
    let actual = sexp.type_of();
    match actual {
        SEXPTYPE::INTSXP => {
            let value: i32 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
            coerce_value(value)
        }
        SEXPTYPE::REALSXP => {
            let value: f64 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
            coerce_value(value)
        }
        SEXPTYPE::RAWSXP => {
            let value: u8 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
            coerce_value(value)
        }
        SEXPTYPE::LGLSXP => {
            let value: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
            if value.is_na() {
                return Err(lglsxp_na_error());
            }
            coerce_value(value.to_i32())
        }
        _ => Err(SexpError::InvalidValue(format!(
            "expected integer, numeric, logical, or raw; got {:?}",
            actual
        ))),
    }
}

#[inline]
fn try_from_sexp_numeric_option<T>(sexp: SEXP) -> Result<Option<T>, SexpError>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    u8: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Debug,
    <f64 as TryCoerce<T>>::Error: std::fmt::Debug,
    <u8 as TryCoerce<T>>::Error: std::fmt::Debug,
{
    if sexp.type_of() == SEXPTYPE::NILSXP {
        return Ok(None);
    }

    let actual = sexp.type_of();
    match actual {
        SEXPTYPE::INTSXP => {
            // Read raw i32 without the NA guard — we handle NA here by returning None.
            let len = sexp.len();
            if len != 1 {
                return Err(crate::from_r::SexpLengthError {
                    expected: 1,
                    actual: len,
                }
                .into());
            }
            let value = unsafe { sexp.as_slice::<i32>() }
                .first()
                .cloned()
                .ok_or_else(|| {
                    SexpError::from(crate::from_r::SexpLengthError {
                        expected: 1,
                        actual: 0,
                    })
                })?;
            if value == NA_INTEGER {
                Ok(None)
            } else {
                coerce_value(value).map(Some)
            }
        }
        SEXPTYPE::REALSXP => {
            let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
            if is_na_real(value) {
                Ok(None)
            } else {
                coerce_value(value).map(Some)
            }
        }
        SEXPTYPE::RAWSXP => {
            let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
            coerce_value(value).map(Some)
        }
        SEXPTYPE::LGLSXP => {
            let value: RLogical = TryFromSexp::try_from_sexp(sexp)?;
            if value.is_na() {
                Ok(None)
            } else {
                coerce_value(value.to_i32()).map(Some)
            }
        }
        _ => Err(SexpError::InvalidValue(format!(
            "expected integer, numeric, logical, or raw; got {:?}",
            actual
        ))),
    }
}

#[inline]
unsafe fn try_from_sexp_numeric_option_unchecked<T>(sexp: SEXP) -> Result<Option<T>, SexpError>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    u8: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Debug,
    <f64 as TryCoerce<T>>::Error: std::fmt::Debug,
    <u8 as TryCoerce<T>>::Error: std::fmt::Debug,
{
    if sexp.type_of() == SEXPTYPE::NILSXP {
        return Ok(None);
    }

    let actual = sexp.type_of();
    match actual {
        SEXPTYPE::INTSXP => {
            // Read raw i32 without the NA guard — we handle NA here by returning None.
            let len = unsafe { sexp.len_unchecked() };
            if len != 1 {
                return Err(crate::from_r::SexpLengthError {
                    expected: 1,
                    actual: len,
                }
                .into());
            }
            let value = unsafe { sexp.as_slice_unchecked::<i32>() }
                .first()
                .cloned()
                .ok_or_else(|| {
                    SexpError::from(crate::from_r::SexpLengthError {
                        expected: 1,
                        actual: 0,
                    })
                })?;
            if value == NA_INTEGER {
                Ok(None)
            } else {
                coerce_value(value).map(Some)
            }
        }
        SEXPTYPE::REALSXP => {
            let value: f64 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
            if is_na_real(value) {
                Ok(None)
            } else {
                coerce_value(value).map(Some)
            }
        }
        SEXPTYPE::RAWSXP => {
            let value: u8 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
            coerce_value(value).map(Some)
        }
        SEXPTYPE::LGLSXP => {
            let value: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
            if value.is_na() {
                Ok(None)
            } else {
                coerce_value(value.to_i32()).map(Some)
            }
        }
        _ => Err(SexpError::InvalidValue(format!(
            "expected integer, numeric, logical, or raw; got {:?}",
            actual
        ))),
    }
}

impl TryFromSexp for i8 {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_scalar(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_scalar_unchecked(sexp) }
    }
}

impl TryFromSexp for i16 {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_scalar(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_scalar_unchecked(sexp) }
    }
}

impl TryFromSexp for u16 {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_scalar(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_scalar_unchecked(sexp) }
    }
}

impl TryFromSexp for u32 {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_scalar(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_scalar_unchecked(sexp) }
    }
}

impl TryFromSexp for f32 {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_scalar(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_scalar_unchecked(sexp) }
    }
}

impl TryFromSexp for Option<i8> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_option(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_option_unchecked(sexp) }
    }
}

impl TryFromSexp for Option<i16> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_option(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_option_unchecked(sexp) }
    }
}

impl TryFromSexp for Option<u16> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_option(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_option_unchecked(sexp) }
    }
}

impl TryFromSexp for Option<u32> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_option(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_option_unchecked(sexp) }
    }
}

impl TryFromSexp for Option<f32> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_option(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_option_unchecked(sexp) }
    }
}
// endregion

// region: Large integer scalar conversions (via f64)
//
// R doesn't have native 64-bit integers, so these read from REALSXP (f64)
// and convert with range/precision checking.
//
// **Round-trip precision:**
// - Values in [-2^53, 2^53] round-trip exactly: i64 → R → i64
// - Values outside this range may not round-trip due to f64 precision loss
//
// **Conversion behavior:**
// - Reads from REALSXP (f64) or INTSXP (i32)
// - Validates the value is a whole number (no fractional part)
// - Validates the value fits in the target type's range
// - Returns error for NA values (use Option<i64> for nullable)

/// Convert R numeric scalar to `i64`.
///
/// # Behavior
///
/// - Reads from REALSXP (f64) or INTSXP (i32)
/// - Validates value is a whole number (no fractional part)
/// - Validates value fits in i64 range
/// - Returns `Err` for NA values
///
/// # Example
///
/// ```ignore
/// // From R integer
/// let val: i64 = TryFromSexp::try_from_sexp(int_sexp)?;
///
/// // From R numeric (must be whole number)
/// let val: i64 = TryFromSexp::try_from_sexp(real_sexp)?;
///
/// // Error: 3.14 is not a whole number
/// let val: Result<i64, _> = TryFromSexp::try_from_sexp(pi_sexp);
/// ```
impl TryFromSexp for i64 {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_scalar(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_scalar_unchecked(sexp) }
    }
}

/// Convert R numeric scalar to `u64`.
///
/// Same behavior as [`i64`](impl TryFromSexp for i64), but also validates
/// the value is non-negative.
impl TryFromSexp for u64 {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_scalar(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_scalar_unchecked(sexp) }
    }
}

/// Convert R numeric scalar to `Option<i64>`, with NA → `None`.
impl TryFromSexp for Option<i64> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_option(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_option_unchecked(sexp) }
    }
}

impl TryFromSexp for Option<u64> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_option(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_option_unchecked(sexp) }
    }
}

impl TryFromSexp for usize {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_scalar(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_scalar_unchecked(sexp) }
    }
}

impl TryFromSexp for Option<usize> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_option(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_option_unchecked(sexp) }
    }
}

impl TryFromSexp for isize {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_scalar(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_scalar_unchecked(sexp) }
    }
}

impl TryFromSexp for Option<isize> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_option(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_option_unchecked(sexp) }
    }
}
// endregion
