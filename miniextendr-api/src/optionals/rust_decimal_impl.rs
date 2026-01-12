//! Integration with the `rust_decimal` crate.
//!
//! Provides conversions between R values and `Decimal`.
//!
//! # Conversion Paths
//!
//! `Decimal` can be converted from R in two ways:
//!
//! 1. **Character (lossless)**: Parse from R `character` - preserves full precision
//! 2. **Numeric (fast path)**: Convert from R `numeric` - may lose precision for
//!    values that don't fit exactly in IEEE 754 double precision
//!
//! When converting FROM R, the input type determines the path:
//! - `character` input: Uses lossless string parsing
//! - `numeric` input: Uses fast f64 conversion (precision warning below)
//!
//! When converting TO R, `Decimal` always produces `character` to preserve precision.
//!
//! # Precision Warning
//!
//! R's numeric type is IEEE 754 double precision (f64), which can represent:
//! - Integers exactly up to 2^53 (about 9 quadrillion)
//! - Decimals with ~15-17 significant digits
//!
//! `rust_decimal::Decimal` supports 28-29 significant digits. Values outside f64's
//! exact representation range will lose precision when converted from R numeric:
//!
//! ```r
//! # These will lose precision when passed as numeric:
//! decimal_from_r(12345678901234567890)  # f64 can't represent this exactly
//! decimal_from_r(0.1 + 0.2)             # f64 rounding error: 0.30000000000000004
//!
//! # Use character for full precision:
//! decimal_from_r("12345678901234567890")
//! decimal_from_r("0.3")
//! ```
//!
//! # Features
//!
//! Enable this module with the `rust_decimal` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["rust_decimal"] }
//! ```

pub use rust_decimal::Decimal;

use crate::coerce::{Coerce, CoerceError, TryCoerce};
use crate::ffi::{SEXP, SEXPTYPE, TYPEOF};
use crate::from_r::{SexpError, SexpNaError, TryFromSexp};
use crate::into_r::IntoR;
use std::str::FromStr;

// =============================================================================
// Coerce/TryCoerce impls for Decimal
// =============================================================================

/// `i32` → `Decimal`: lossless, i32 fits exactly in Decimal
impl Coerce<Decimal> for i32 {
    #[inline(always)]
    fn coerce(self) -> Decimal {
        Decimal::from(self)
    }
}

/// `i64` → `Decimal`: lossless, i64 fits exactly in Decimal (28-29 digits)
impl Coerce<Decimal> for i64 {
    #[inline(always)]
    fn coerce(self) -> Decimal {
        Decimal::from(self)
    }
}

/// `f64` → `Decimal`: fallible, may have NaN/Inf or representation issues
impl TryCoerce<Decimal> for f64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<Decimal, CoerceError> {
        if self.is_nan() {
            return Err(CoerceError::NaN);
        }
        if self.is_infinite() {
            return Err(CoerceError::Overflow);
        }
        // Decimal::try_from(f64) may fail for values outside its range
        Decimal::try_from(self).map_err(|_| CoerceError::Overflow)
    }
}

/// `Decimal` → `f64`: may lose precision (Decimal has 28-29 digits, f64 ~15-17)
impl TryCoerce<f64> for Decimal {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<f64, CoerceError> {
        // Convert to f64 using to_string parsing for best precision
        // Note: Decimal can represent values that f64 cannot exactly
        use rust_decimal::prelude::ToPrimitive;
        self.to_f64().ok_or(CoerceError::Overflow)
    }
}

/// `Decimal` → `i64`: fallible, may not fit or may have fractional part
impl TryCoerce<i64> for Decimal {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<i64, CoerceError> {
        use rust_decimal::prelude::ToPrimitive;
        // Check for fractional part
        if self.fract() != Decimal::ZERO {
            return Err(CoerceError::PrecisionLoss);
        }
        self.to_i64().ok_or(CoerceError::Overflow)
    }
}

/// `Decimal` → `i32`: fallible, may not fit or may have fractional part
impl TryCoerce<i32> for Decimal {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<i32, CoerceError> {
        use rust_decimal::prelude::ToPrimitive;
        // Check for fractional part
        if self.fract() != Decimal::ZERO {
            return Err(CoerceError::PrecisionLoss);
        }
        self.to_i32().ok_or(CoerceError::Overflow)
    }
}

fn parse_decimal(s: &str) -> Result<Decimal, SexpError> {
    Decimal::from_str(s).map_err(|e| SexpError::InvalidValue(e.to_string()))
}

fn decimal_from_f64(f: f64) -> Result<Decimal, SexpError> {
    Decimal::try_from(f).map_err(|e| SexpError::InvalidValue(e.to_string()))
}

/// Get the SEXP type (safe wrapper)
fn sexp_type(sexp: SEXP) -> SEXPTYPE {
    unsafe { TYPEOF(sexp) }
}

impl TryFromSexp for Decimal {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        match sexp_type(sexp) {
            SEXPTYPE::REALSXP => {
                // Numeric fast path (may lose precision for large values)
                let f: Option<f64> = TryFromSexp::try_from_sexp(sexp)?;
                let f = f.ok_or(SexpError::Na(SexpNaError {
                    sexp_type: SEXPTYPE::REALSXP,
                }))?;
                decimal_from_f64(f)
            }
            SEXPTYPE::INTSXP => {
                // Integer path (lossless for i32 range)
                let i: Option<i32> = TryFromSexp::try_from_sexp(sexp)?;
                let i = i.ok_or(SexpError::Na(SexpNaError {
                    sexp_type: SEXPTYPE::INTSXP,
                }))?;
                Ok(Decimal::from(i))
            }
            _ => {
                // Character path (lossless)
                let s: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
                let s = s.ok_or(SexpError::Na(SexpNaError {
                    sexp_type: SEXPTYPE::STRSXP,
                }))?;
                parse_decimal(&s)
            }
        }
    }
}

impl TryFromSexp for Option<Decimal> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        match sexp_type(sexp) {
            SEXPTYPE::REALSXP => {
                let f: Option<f64> = TryFromSexp::try_from_sexp(sexp)?;
                match f {
                    Some(f) => decimal_from_f64(f).map(Some),
                    None => Ok(None),
                }
            }
            SEXPTYPE::INTSXP => {
                let i: Option<i32> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(i.map(Decimal::from))
            }
            _ => {
                let s: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
                match s {
                    Some(s) => parse_decimal(&s).map(Some),
                    None => Ok(None),
                }
            }
        }
    }
}

impl TryFromSexp for Vec<Decimal> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        match sexp_type(sexp) {
            SEXPTYPE::REALSXP => {
                let values: Vec<Option<f64>> = TryFromSexp::try_from_sexp(sexp)?;
                values
                    .into_iter()
                    .map(|opt| {
                        let f = opt.ok_or(SexpError::Na(SexpNaError {
                            sexp_type: SEXPTYPE::REALSXP,
                        }))?;
                        decimal_from_f64(f)
                    })
                    .collect()
            }
            SEXPTYPE::INTSXP => {
                let values: Vec<Option<i32>> = TryFromSexp::try_from_sexp(sexp)?;
                values
                    .into_iter()
                    .map(|opt| {
                        let i = opt.ok_or(SexpError::Na(SexpNaError {
                            sexp_type: SEXPTYPE::INTSXP,
                        }))?;
                        Ok(Decimal::from(i))
                    })
                    .collect()
            }
            _ => {
                let values: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
                values
                    .into_iter()
                    .map(|opt| {
                        let s = opt.ok_or(SexpError::Na(SexpNaError {
                            sexp_type: SEXPTYPE::STRSXP,
                        }))?;
                        parse_decimal(&s)
                    })
                    .collect()
            }
        }
    }
}

impl TryFromSexp for Vec<Option<Decimal>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        match sexp_type(sexp) {
            SEXPTYPE::REALSXP => {
                let values: Vec<Option<f64>> = TryFromSexp::try_from_sexp(sexp)?;
                values
                    .into_iter()
                    .map(|opt| match opt {
                        Some(f) => decimal_from_f64(f).map(Some),
                        None => Ok(None),
                    })
                    .collect()
            }
            SEXPTYPE::INTSXP => {
                let values: Vec<Option<i32>> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(values
                    .into_iter()
                    .map(|opt| opt.map(Decimal::from))
                    .collect())
            }
            _ => {
                let values: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
                values
                    .into_iter()
                    .map(|opt| match opt {
                        Some(s) => parse_decimal(&s).map(Some),
                        None => Ok(None),
                    })
                    .collect()
            }
        }
    }
}

impl IntoR for Decimal {
    fn into_sexp(self) -> SEXP {
        self.to_string().into_sexp()
    }
}

impl IntoR for Option<Decimal> {
    fn into_sexp(self) -> SEXP {
        self.map(|v| v.to_string()).into_sexp()
    }
}

impl IntoR for Vec<Decimal> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .into_sexp()
    }
}

impl IntoR for Vec<Option<Decimal>> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|v| v.map(|val| val.to_string()))
            .collect::<Vec<_>>()
            .into_sexp()
    }
}

// =============================================================================
// RDecimalOps adapter trait
// =============================================================================

use rust_decimal::prelude::*;

/// Adapter trait for [`Decimal`] operations.
///
/// Provides fixed-precision decimal arithmetic from R.
/// Automatically implemented for `rust_decimal::Decimal`.
///
/// # Example
///
/// ```rust,ignore
/// use rust_decimal::Decimal;
/// use miniextendr_api::rust_decimal_impl::RDecimalOps;
///
/// #[derive(ExternalPtr)]
/// struct MyDecimal(Decimal);
///
/// #[miniextendr]
/// impl RDecimalOps for MyDecimal {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RDecimalOps for MyDecimal;
/// }
/// ```
///
/// In R:
/// ```r
/// x <- MyDecimal$from_str("123.456")
/// x$add_str("0.001")$as_string()  # "123.457"
/// x$round(2)$as_string()          # "123.46"
/// x$scale()                       # 3
/// ```
pub trait RDecimalOps {
    /// Convert to string representation.
    fn as_string(&self) -> String;

    /// Check if zero.
    fn is_zero(&self) -> bool;

    /// Check if positive (> 0).
    fn is_positive(&self) -> bool;

    /// Check if negative (< 0).
    fn is_negative(&self) -> bool;

    /// Get the sign as an integer: -1, 0, or 1.
    fn sign(&self) -> i32;

    /// Get the number of decimal places.
    fn scale(&self) -> i32;

    /// Get the absolute value.
    fn abs(&self) -> Decimal;

    /// Negate the value.
    fn neg(&self) -> Decimal;

    /// Add another Decimal (passed as string).
    fn add_str(&self, other: &str) -> Result<Decimal, String>;

    /// Subtract another Decimal (passed as string).
    fn sub_str(&self, other: &str) -> Result<Decimal, String>;

    /// Multiply by another Decimal (passed as string).
    fn mul_str(&self, other: &str) -> Result<Decimal, String>;

    /// Divide by another Decimal (passed as string).
    fn div_str(&self, other: &str) -> Result<Decimal, String>;

    /// Remainder after division (passed as string).
    fn rem_str(&self, other: &str) -> Result<Decimal, String>;

    /// Round to the specified number of decimal places.
    fn round(&self, dp: i32) -> Decimal;

    /// Round toward negative infinity.
    fn floor(&self) -> Decimal;

    /// Round toward positive infinity.
    fn ceil(&self) -> Decimal;

    /// Truncate toward zero (remove decimal places).
    fn trunc(&self) -> Decimal;

    /// Get the fractional part (value - trunc).
    fn fract(&self) -> Decimal;

    /// Convert to f64 (may lose precision).
    fn as_f64(&self) -> f64;

    /// Try to convert to i64 (returns None if out of range or has decimals).
    fn as_i64(&self) -> Option<i64>;

    /// Normalize - remove trailing zeros.
    fn normalize(&self) -> Decimal;

    /// Check if the value is an integer (no fractional part).
    fn is_integer(&self) -> bool;
}

impl RDecimalOps for Decimal {
    fn as_string(&self) -> String {
        ToString::to_string(self)
    }

    fn is_zero(&self) -> bool {
        Decimal::is_zero(self)
    }

    fn is_positive(&self) -> bool {
        Decimal::is_sign_positive(self) && !Decimal::is_zero(self)
    }

    fn is_negative(&self) -> bool {
        Decimal::is_sign_negative(self) && !Decimal::is_zero(self)
    }

    fn sign(&self) -> i32 {
        if Decimal::is_zero(self) {
            0
        } else if Decimal::is_sign_negative(self) {
            -1
        } else {
            1
        }
    }

    fn scale(&self) -> i32 {
        Decimal::scale(self) as i32
    }

    fn abs(&self) -> Decimal {
        Decimal::abs(self)
    }

    fn neg(&self) -> Decimal {
        -*self
    }

    fn add_str(&self, other: &str) -> Result<Decimal, String> {
        let other = Decimal::from_str(other).map_err(|e| e.to_string())?;
        self.checked_add(other)
            .ok_or_else(|| "addition overflow".to_string())
    }

    fn sub_str(&self, other: &str) -> Result<Decimal, String> {
        let other = Decimal::from_str(other).map_err(|e| e.to_string())?;
        self.checked_sub(other)
            .ok_or_else(|| "subtraction overflow".to_string())
    }

    fn mul_str(&self, other: &str) -> Result<Decimal, String> {
        let other = Decimal::from_str(other).map_err(|e| e.to_string())?;
        self.checked_mul(other)
            .ok_or_else(|| "multiplication overflow".to_string())
    }

    fn div_str(&self, other: &str) -> Result<Decimal, String> {
        let other = Decimal::from_str(other).map_err(|e| e.to_string())?;
        if other.is_zero() {
            return Err("division by zero".to_string());
        }
        self.checked_div(other)
            .ok_or_else(|| "division overflow".to_string())
    }

    fn rem_str(&self, other: &str) -> Result<Decimal, String> {
        let other = Decimal::from_str(other).map_err(|e| e.to_string())?;
        if other.is_zero() {
            return Err("division by zero".to_string());
        }
        self.checked_rem(other)
            .ok_or_else(|| "remainder overflow".to_string())
    }

    fn round(&self, dp: i32) -> Decimal {
        Decimal::round_dp(self, dp.max(0) as u32)
    }

    fn floor(&self) -> Decimal {
        Decimal::floor(self)
    }

    fn ceil(&self) -> Decimal {
        Decimal::ceil(self)
    }

    fn trunc(&self) -> Decimal {
        Decimal::trunc(self)
    }

    fn fract(&self) -> Decimal {
        Decimal::fract(self)
    }

    fn as_f64(&self) -> f64 {
        ToPrimitive::to_f64(self).unwrap_or(f64::NAN)
    }

    fn as_i64(&self) -> Option<i64> {
        // Only convert if it's an integer (no fractional part)
        if Decimal::fract(self).is_zero() {
            ToPrimitive::to_i64(self)
        } else {
            None
        }
    }

    fn normalize(&self) -> Decimal {
        Decimal::normalize(self)
    }

    fn is_integer(&self) -> bool {
        Decimal::fract(self).is_zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_from_str() {
        let d = Decimal::from_str("123.456").unwrap();
        assert_eq!(RDecimalOps::as_string(&d), "123.456");
    }

    #[test]
    fn rdecimalops_sign() {
        let pos = Decimal::from_str("123.45").unwrap();
        let neg = Decimal::from_str("-67.89").unwrap();
        let zero = Decimal::ZERO;

        assert!(RDecimalOps::is_positive(&pos));
        assert!(!RDecimalOps::is_negative(&pos));
        assert_eq!(RDecimalOps::sign(&pos), 1);

        assert!(!RDecimalOps::is_positive(&neg));
        assert!(RDecimalOps::is_negative(&neg));
        assert_eq!(RDecimalOps::sign(&neg), -1);

        assert!(RDecimalOps::is_zero(&zero));
        assert_eq!(RDecimalOps::sign(&zero), 0);
    }

    #[test]
    fn rdecimalops_arithmetic() {
        let d = Decimal::from_str("100.5").unwrap();

        let sum = RDecimalOps::add_str(&d, "0.5").unwrap();
        assert_eq!(ToString::to_string(&sum), "101.0");

        let diff = RDecimalOps::sub_str(&d, "0.5").unwrap();
        assert_eq!(ToString::to_string(&diff), "100.0");

        let prod = RDecimalOps::mul_str(&d, "2").unwrap();
        assert_eq!(ToString::to_string(&prod), "201.0");

        let quot = RDecimalOps::div_str(&d, "2").unwrap();
        // Note: Decimal preserves scale, so 100.5 / 2 = 50.250
        assert_eq!(ToString::to_string(&RDecimalOps::normalize(&quot)), "50.25");

        let rem = RDecimalOps::rem_str(&d, "7").unwrap();
        assert_eq!(ToString::to_string(&rem), "2.5"); // 100.5 % 7 = 2.5
    }

    #[test]
    fn rdecimalops_rounding() {
        let d = Decimal::from_str("123.456").unwrap();

        assert_eq!(ToString::to_string(&RDecimalOps::round(&d, 2)), "123.46");
        assert_eq!(ToString::to_string(&RDecimalOps::floor(&d)), "123");
        assert_eq!(ToString::to_string(&RDecimalOps::ceil(&d)), "124");
        assert_eq!(ToString::to_string(&RDecimalOps::trunc(&d)), "123");
        assert_eq!(ToString::to_string(&RDecimalOps::fract(&d)), "0.456");
    }

    #[test]
    fn rdecimalops_scale() {
        let d1 = Decimal::from_str("123.456").unwrap();
        assert_eq!(RDecimalOps::scale(&d1), 3);

        let d2 = Decimal::from_str("100").unwrap();
        assert_eq!(RDecimalOps::scale(&d2), 0);
    }

    #[test]
    fn rdecimalops_abs_neg() {
        let neg = Decimal::from_str("-123.45").unwrap();
        assert_eq!(ToString::to_string(&RDecimalOps::abs(&neg)), "123.45");
        assert_eq!(ToString::to_string(&RDecimalOps::neg(&neg)), "123.45");

        let pos = Decimal::from_str("67.89").unwrap();
        assert_eq!(ToString::to_string(&RDecimalOps::abs(&pos)), "67.89");
        assert_eq!(ToString::to_string(&RDecimalOps::neg(&pos)), "-67.89");
    }

    #[test]
    fn rdecimalops_conversions() {
        let d = Decimal::from_str("123.456").unwrap();
        assert!((RDecimalOps::as_f64(&d) - 123.456).abs() < 0.0001);
        assert_eq!(RDecimalOps::as_i64(&d), None); // Has decimal part

        let int_d = Decimal::from_str("100").unwrap();
        assert_eq!(RDecimalOps::as_i64(&int_d), Some(100));
        assert!(RDecimalOps::is_integer(&int_d));
        assert!(!RDecimalOps::is_integer(&d));
    }

    #[test]
    fn rdecimalops_normalize() {
        let d = Decimal::from_str("100.500").unwrap();
        let normalized = RDecimalOps::normalize(&d);
        assert_eq!(ToString::to_string(&normalized), "100.5");
    }

    #[test]
    fn rdecimalops_division_by_zero() {
        let d = Decimal::from_str("100").unwrap();
        assert!(RDecimalOps::div_str(&d, "0").is_err());
        assert!(RDecimalOps::rem_str(&d, "0").is_err());
    }
}
