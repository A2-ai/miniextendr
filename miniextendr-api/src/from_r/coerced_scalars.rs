//! Coerced scalar conversions (multi-source numeric) and large integer scalars.
//!
//! These types accept multiple R source types (INTSXP, REALSXP, RAWSXP, LGLSXP)
//! and coerce to the target Rust type via [`TryCoerce`](crate::coerce::TryCoerce).
//!
//! Covers: `i8`, `i16`, `u16`, `u32`, `f32` (sub-native scalars) and
//! `i64`, `u64`, `isize`, `usize` (large integers via f64 intermediary).

use crate::coerce::TryCoerce;
use crate::ffi::{RLogical, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, TryFromSexp, is_na_real};

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
            let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
            if value == crate::altrep_traits::NA_INTEGER {
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
            let value: i32 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
            if value == crate::altrep_traits::NA_INTEGER {
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
        let actual = sexp.type_of();
        match actual {
            SEXPTYPE::INTSXP => {
                let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(value as f32)
            }
            SEXPTYPE::REALSXP => {
                let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(value as f32)
            }
            SEXPTYPE::RAWSXP => {
                let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(value as f32)
            }
            SEXPTYPE::LGLSXP => {
                let value: RLogical = TryFromSexp::try_from_sexp(sexp)?;
                Ok(value.to_i32() as f32)
            }
            _ => Err(SexpError::InvalidValue(format!(
                "expected integer, numeric, logical, or raw; got {:?}",
                actual
            ))),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        match actual {
            SEXPTYPE::INTSXP => {
                let value: i32 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(value as f32)
            }
            SEXPTYPE::REALSXP => {
                let value: f64 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(value as f32)
            }
            SEXPTYPE::RAWSXP => {
                let value: u8 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(value as f32)
            }
            SEXPTYPE::LGLSXP => {
                let value: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(value.to_i32() as f32)
            }
            _ => Err(SexpError::InvalidValue(format!(
                "expected integer, numeric, logical, or raw; got {:?}",
                actual
            ))),
        }
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
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let actual = sexp.type_of();
        match actual {
            SEXPTYPE::INTSXP => {
                let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
                if value == crate::altrep_traits::NA_INTEGER {
                    Ok(None)
                } else {
                    Ok(Some(value as f32))
                }
            }
            SEXPTYPE::REALSXP => {
                let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
                if is_na_real(value) {
                    Ok(None)
                } else {
                    Ok(Some(value as f32))
                }
            }
            SEXPTYPE::RAWSXP => {
                let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(Some(value as f32))
            }
            SEXPTYPE::LGLSXP => {
                let value: RLogical = TryFromSexp::try_from_sexp(sexp)?;
                if value.is_na() {
                    Ok(None)
                } else {
                    Ok(Some(value.to_i32() as f32))
                }
            }
            _ => Err(SexpError::InvalidValue(format!(
                "expected integer, numeric, logical, or raw; got {:?}",
                actual
            ))),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let actual = sexp.type_of();
        match actual {
            SEXPTYPE::INTSXP => {
                let value: i32 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                if value == crate::altrep_traits::NA_INTEGER {
                    Ok(None)
                } else {
                    Ok(Some(value as f32))
                }
            }
            SEXPTYPE::REALSXP => {
                let value: f64 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                if is_na_real(value) {
                    Ok(None)
                } else {
                    Ok(Some(value as f32))
                }
            }
            SEXPTYPE::RAWSXP => {
                let value: u8 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(Some(value as f32))
            }
            SEXPTYPE::LGLSXP => {
                let value: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                if value.is_na() {
                    Ok(None)
                } else {
                    Ok(Some(value.to_i32() as f32))
                }
            }
            _ => Err(SexpError::InvalidValue(format!(
                "expected integer, numeric, logical, or raw; got {:?}",
                actual
            ))),
        }
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
        // Try i32 first (more common), fall back to f64 for large values
        let actual = sexp.type_of();
        match actual {
            SEXPTYPE::INTSXP => {
                use crate::coerce::TryCoerce;
                let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
                value
                    .try_coerce()
                    .map_err(|e| SexpError::InvalidValue(format!("{e}")))
            }
            SEXPTYPE::REALSXP => {
                use crate::coerce::TryCoerce;
                let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
                let u: u64 = value
                    .try_coerce()
                    .map_err(|e| SexpError::InvalidValue(format!("{e}")))?;
                u.try_into()
                    .map_err(|_| SexpError::InvalidValue("value out of usize range".into()))
            }
            SEXPTYPE::RAWSXP => {
                use crate::coerce::Coerce;
                let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(value.coerce())
            }
            SEXPTYPE::LGLSXP => {
                use crate::coerce::TryCoerce;
                let value: RLogical = TryFromSexp::try_from_sexp(sexp)?;
                value
                    .to_i32()
                    .try_coerce()
                    .map_err(|e| SexpError::InvalidValue(format!("{e}")))
            }
            _ => Err(SexpError::InvalidValue(format!(
                "expected integer, numeric, logical, or raw; got {:?}",
                actual
            ))),
        }
    }
}

impl TryFromSexp for Option<usize> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let actual = sexp.type_of();
        match actual {
            SEXPTYPE::INTSXP => {
                use crate::coerce::TryCoerce;
                let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
                if value == crate::altrep_traits::NA_INTEGER {
                    Ok(None)
                } else {
                    value
                        .try_coerce()
                        .map(Some)
                        .map_err(|e| SexpError::InvalidValue(format!("{e}")))
                }
            }
            SEXPTYPE::REALSXP => {
                use crate::coerce::TryCoerce;
                let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
                if is_na_real(value) {
                    return Ok(None);
                }
                let u: u64 = value
                    .try_coerce()
                    .map_err(|e| SexpError::InvalidValue(format!("{e}")))?;
                u.try_into()
                    .map(Some)
                    .map_err(|_| SexpError::InvalidValue("value out of usize range".into()))
            }
            SEXPTYPE::RAWSXP => {
                use crate::coerce::Coerce;
                let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(Some(value.coerce()))
            }
            SEXPTYPE::LGLSXP => {
                use crate::coerce::TryCoerce;
                let value: RLogical = TryFromSexp::try_from_sexp(sexp)?;
                if value.is_na() {
                    Ok(None)
                } else {
                    value
                        .to_i32()
                        .try_coerce()
                        .map(Some)
                        .map_err(|e| SexpError::InvalidValue(format!("{e}")))
                }
            }
            _ => Err(SexpError::InvalidValue(format!(
                "expected integer, numeric, logical, or raw; got {:?}",
                actual
            ))),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

impl TryFromSexp for isize {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // Try i32 first (more common), fall back to f64 for large values
        let actual = sexp.type_of();
        match actual {
            SEXPTYPE::INTSXP => {
                use crate::coerce::Coerce;
                let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(value.coerce())
            }
            SEXPTYPE::REALSXP => {
                use crate::coerce::TryCoerce;
                let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
                let i: i64 = value
                    .try_coerce()
                    .map_err(|e| SexpError::InvalidValue(format!("{e}")))?;
                i.try_into()
                    .map_err(|_| SexpError::InvalidValue("value out of isize range".into()))
            }
            SEXPTYPE::RAWSXP => {
                use crate::coerce::Coerce;
                let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(value.coerce())
            }
            SEXPTYPE::LGLSXP => {
                use crate::coerce::Coerce;
                let value: RLogical = TryFromSexp::try_from_sexp(sexp)?;
                Ok(value.to_i32().coerce())
            }
            _ => Err(SexpError::InvalidValue(format!(
                "expected integer, numeric, logical, or raw; got {:?}",
                actual
            ))),
        }
    }
}

impl TryFromSexp for Option<isize> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let actual = sexp.type_of();
        match actual {
            SEXPTYPE::INTSXP => {
                use crate::coerce::Coerce;
                let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
                if value == crate::altrep_traits::NA_INTEGER {
                    Ok(None)
                } else {
                    Ok(Some(value.coerce()))
                }
            }
            SEXPTYPE::REALSXP => {
                use crate::coerce::TryCoerce;
                let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
                if is_na_real(value) {
                    return Ok(None);
                }
                let i: i64 = value
                    .try_coerce()
                    .map_err(|e| SexpError::InvalidValue(format!("{e}")))?;
                i.try_into()
                    .map(Some)
                    .map_err(|_| SexpError::InvalidValue("value out of isize range".into()))
            }
            SEXPTYPE::RAWSXP => {
                use crate::coerce::Coerce;
                let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(Some(value.coerce()))
            }
            SEXPTYPE::LGLSXP => {
                use crate::coerce::Coerce;
                let value: RLogical = TryFromSexp::try_from_sexp(sexp)?;
                if value.is_na() {
                    Ok(None)
                } else {
                    Ok(Some(value.to_i32().coerce()))
                }
            }
            _ => Err(SexpError::InvalidValue(format!(
                "expected integer, numeric, logical, or raw; got {:?}",
                actual
            ))),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}
// endregion
