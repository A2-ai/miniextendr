//! Integration with the `ordered-float` crate.
//!
//! Provides conversions for `OrderedFloat<f64>` and `OrderedFloat<f32>`.
//!
//! # Features
//!
//! Enable this module with the `ordered-float` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["ordered-float"] }
//! ```

pub use ordered_float::OrderedFloat;

use crate::ffi::{SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;

fn parse_f64(sexp: SEXP) -> Result<f64, SexpError> {
    let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
    Ok(value)
}

fn parse_f32(sexp: SEXP) -> Result<f32, SexpError> {
    let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
    Ok(value as f32)
}

impl TryFromSexp for OrderedFloat<f64> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        parse_f64(sexp).map(OrderedFloat)
    }
}

impl TryFromSexp for OrderedFloat<f32> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        parse_f32(sexp).map(OrderedFloat)
    }
}

impl TryFromSexp for Option<OrderedFloat<f64>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let value: Option<f64> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(value.map(OrderedFloat))
    }
}

impl TryFromSexp for Option<OrderedFloat<f32>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let value: Option<f64> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(value.map(|v| OrderedFloat(v as f32)))
    }
}

impl TryFromSexp for Vec<OrderedFloat<f64>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }

        let slice: &[f64] = unsafe { sexp.as_slice::<f64>() };
        Ok(slice.iter().copied().map(OrderedFloat).collect())
    }
}

impl TryFromSexp for Vec<OrderedFloat<f32>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }

        let slice: &[f64] = unsafe { sexp.as_slice::<f64>() };
        Ok(slice.iter().map(|v| OrderedFloat(*v as f32)).collect())
    }
}

impl TryFromSexp for Vec<Option<OrderedFloat<f64>>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let values: Vec<Option<f64>> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(values.into_iter().map(|v| v.map(OrderedFloat)).collect())
    }
}

impl TryFromSexp for Vec<Option<OrderedFloat<f32>>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let values: Vec<Option<f64>> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(values
            .into_iter()
            .map(|v| v.map(|val| OrderedFloat(val as f32)))
            .collect())
    }
}

impl IntoR for OrderedFloat<f64> {
    fn into_sexp(self) -> SEXP {
        self.0.into_sexp()
    }
}

impl IntoR for OrderedFloat<f32> {
    fn into_sexp(self) -> SEXP {
        (self.0 as f64).into_sexp()
    }
}

impl IntoR for Option<OrderedFloat<f64>> {
    fn into_sexp(self) -> SEXP {
        self.map(|v| v.0).into_sexp()
    }
}

impl IntoR for Option<OrderedFloat<f32>> {
    fn into_sexp(self) -> SEXP {
        self.map(|v| v.0 as f64).into_sexp()
    }
}

impl IntoR for Vec<OrderedFloat<f64>> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|v| v.0)
            .collect::<Vec<_>>()
            .into_sexp()
    }
}

impl IntoR for Vec<OrderedFloat<f32>> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|v| v.0 as f64)
            .collect::<Vec<_>>()
            .into_sexp()
    }
}

impl IntoR for Vec<Option<OrderedFloat<f64>>> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|v| v.map(|val| val.0))
            .collect::<Vec<_>>()
            .into_sexp()
    }
}

impl IntoR for Vec<Option<OrderedFloat<f32>>> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|v| v.map(|val| val.0 as f64))
            .collect::<Vec<_>>()
            .into_sexp()
    }
}

// =============================================================================
// ROrderedFloatOps adapter trait
// =============================================================================

use ordered_float::FloatCore;

/// Adapter trait for [`OrderedFloat`] operations.
///
/// Provides NaN-safe numeric operations from R.
/// Automatically implemented for `OrderedFloat<T>` where T: FloatCore.
///
/// # Why OrderedFloat?
///
/// Standard floats in Rust don't implement `Ord` because NaN breaks ordering.
/// `OrderedFloat` wraps floats to provide total ordering where NaN < all values.
/// This is useful for sorting, using floats as map keys, etc.
///
/// # Example
///
/// ```rust,ignore
/// use ordered_float::OrderedFloat;
/// use miniextendr_api::ordered_float_impl::ROrderedFloatOps;
///
/// #[derive(ExternalPtr)]
/// struct MyFloat(OrderedFloat<f64>);
///
/// #[miniextendr]
/// impl ROrderedFloatOps for MyFloat {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl ROrderedFloatOps for MyFloat;
/// }
/// ```
///
/// In R:
/// ```r
/// x <- MyFloat$new(3.14)
/// x$is_nan()       # FALSE
/// x$is_infinite()  # FALSE
/// x$is_finite()    # TRUE
/// x$floor()        # 3.0
/// x$ceil()         # 4.0
/// ```
pub trait ROrderedFloatOps {
    /// Get the inner float value.
    fn into_inner(&self) -> f64;

    /// Check if the value is NaN.
    fn is_nan(&self) -> bool;

    /// Check if the value is infinite (positive or negative).
    fn is_infinite(&self) -> bool;

    /// Check if the value is finite (not NaN or infinite).
    fn is_finite(&self) -> bool;

    /// Check if the value is positive.
    fn is_positive(&self) -> bool;

    /// Check if the value is negative.
    fn is_negative(&self) -> bool;

    /// Get the floor (largest integer <= self).
    fn floor(&self) -> f64;

    /// Get the ceiling (smallest integer >= self).
    fn ceil(&self) -> f64;

    /// Round to nearest integer.
    fn round(&self) -> f64;

    /// Truncate toward zero.
    fn trunc(&self) -> f64;

    /// Get the fractional part.
    fn fract(&self) -> f64;

    /// Get the absolute value.
    fn abs(&self) -> f64;

    /// Get the sign: -1.0, 0.0, or 1.0.
    fn signum(&self) -> f64;

    /// Return the minimum of self and other (NaN-safe).
    fn min_with(&self, other: f64) -> f64;

    /// Return the maximum of self and other (NaN-safe).
    fn max_with(&self, other: f64) -> f64;

    /// Clamp the value to a range.
    fn clamp_to(&self, min: f64, max: f64) -> f64;
}

impl<T: FloatCore + Into<f64> + Copy> ROrderedFloatOps for OrderedFloat<T>
where
    f64: From<T>,
{
    fn into_inner(&self) -> f64 {
        f64::from(self.0)
    }

    fn is_nan(&self) -> bool {
        self.0.is_nan()
    }

    fn is_infinite(&self) -> bool {
        self.0.is_infinite()
    }

    fn is_finite(&self) -> bool {
        self.0.is_finite()
    }

    fn is_positive(&self) -> bool {
        self.0.is_sign_positive() && !self.0.is_nan()
    }

    fn is_negative(&self) -> bool {
        self.0.is_sign_negative() && !self.0.is_nan()
    }

    fn floor(&self) -> f64 {
        f64::from(self.0).floor()
    }

    fn ceil(&self) -> f64 {
        f64::from(self.0).ceil()
    }

    fn round(&self) -> f64 {
        f64::from(self.0).round()
    }

    fn trunc(&self) -> f64 {
        f64::from(self.0).trunc()
    }

    fn fract(&self) -> f64 {
        f64::from(self.0).fract()
    }

    fn abs(&self) -> f64 {
        f64::from(self.0).abs()
    }

    fn signum(&self) -> f64 {
        f64::from(self.0).signum()
    }

    fn min_with(&self, other: f64) -> f64 {
        let s = f64::from(self.0);
        Ord::min(OrderedFloat(s), OrderedFloat(other)).0
    }

    fn max_with(&self, other: f64) -> f64 {
        let s = f64::from(self.0);
        Ord::max(OrderedFloat(s), OrderedFloat(other)).0
    }

    fn clamp_to(&self, min: f64, max: f64) -> f64 {
        let s = f64::from(self.0);
        Ord::clamp(OrderedFloat(s), OrderedFloat(min), OrderedFloat(max)).0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_float_basic() {
        let of = OrderedFloat(3.14f64);
        assert!(!of.is_nan());
        assert!(of.is_finite());
        assert!(!of.is_infinite());
    }

    #[test]
    fn rorderedfloatops_predicates() {
        let positive = OrderedFloat(3.14f64);
        let negative = OrderedFloat(-2.5f64);
        let nan = OrderedFloat(f64::NAN);
        let inf = OrderedFloat(f64::INFINITY);

        assert!(ROrderedFloatOps::is_positive(&positive));
        assert!(!ROrderedFloatOps::is_negative(&positive));
        assert!(ROrderedFloatOps::is_negative(&negative));
        assert!(!ROrderedFloatOps::is_positive(&negative));

        assert!(ROrderedFloatOps::is_nan(&nan));
        assert!(!ROrderedFloatOps::is_finite(&nan));

        assert!(ROrderedFloatOps::is_infinite(&inf));
        assert!(!ROrderedFloatOps::is_finite(&inf));
    }

    #[test]
    fn rorderedfloatops_rounding() {
        let of = OrderedFloat(3.7f64);
        assert_eq!(ROrderedFloatOps::floor(&of), 3.0);
        assert_eq!(ROrderedFloatOps::ceil(&of), 4.0);
        assert_eq!(ROrderedFloatOps::round(&of), 4.0);
        assert_eq!(ROrderedFloatOps::trunc(&of), 3.0);
        assert!((ROrderedFloatOps::fract(&of) - 0.7).abs() < 0.001);
    }

    #[test]
    fn rorderedfloatops_abs_signum() {
        let neg = OrderedFloat(-5.0f64);
        assert_eq!(ROrderedFloatOps::abs(&neg), 5.0);
        assert_eq!(ROrderedFloatOps::signum(&neg), -1.0);

        let pos = OrderedFloat(5.0f64);
        assert_eq!(ROrderedFloatOps::signum(&pos), 1.0);
    }

    #[test]
    fn rorderedfloatops_min_max_clamp() {
        let of = OrderedFloat(5.0f64);

        assert_eq!(ROrderedFloatOps::min_with(&of, 3.0), 3.0);
        assert_eq!(ROrderedFloatOps::min_with(&of, 7.0), 5.0);

        assert_eq!(ROrderedFloatOps::max_with(&of, 3.0), 5.0);
        assert_eq!(ROrderedFloatOps::max_with(&of, 7.0), 7.0);

        assert_eq!(ROrderedFloatOps::clamp_to(&of, 0.0, 10.0), 5.0);
        assert_eq!(ROrderedFloatOps::clamp_to(&of, 6.0, 10.0), 6.0);
        assert_eq!(ROrderedFloatOps::clamp_to(&of, 0.0, 3.0), 3.0);
    }

    #[test]
    fn rorderedfloatops_nan_handling() {
        let nan = OrderedFloat(f64::NAN);
        let val = OrderedFloat(5.0f64);

        // In OrderedFloat's Ord impl, NaN is greater than all other values
        // So min(5.0, NaN) = 5.0, and max(NaN, 5.0) = NaN
        assert_eq!(ROrderedFloatOps::min_with(&val, f64::NAN), 5.0);
        assert!(ROrderedFloatOps::max_with(&nan, 5.0).is_nan());
    }

    #[test]
    fn rorderedfloatops_f32() {
        let of = OrderedFloat(3.14f32);
        assert!(!ROrderedFloatOps::is_nan(&of));
        assert!(ROrderedFloatOps::is_finite(&of));
        assert!((ROrderedFloatOps::floor(&of) - 3.0).abs() < 0.001);
    }
}
