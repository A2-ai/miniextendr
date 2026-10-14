//! Integration with the `num-traits` crate for generic numeric operations.
//!
//! This module provides adapter traits that expose `num-traits` functionality to R:
//!
//! - [`RNum`] - For types implementing `num_traits::Num` (basic numeric operations)
//! - [`RFloat`] - For types implementing `num_traits::Float` (floating-point operations)
//! - [`RSigned`] - For types implementing `num_traits::Signed` (signed number operations)
//!
//! These traits have blanket implementations, so any type implementing the underlying
//! `num-traits` trait automatically gets the adapter methods.
//!
//! The `r_` prefix keeps these blanket methods separate from ordinary numeric
//! methods and the source crate's traits. These are Rust helpers; trait-ABI
//! registration is tracked in #1120.
//!
//! # Example
//!
//! ```
//! use miniextendr_api::{RFloat, RNum};
//!
//! assert!(0i32.r_is_zero());
//! assert_eq!(3.5f64.r_floor(), 3.0);
//! ```

use num_traits::{Float, Num, Signed, Zero};

/// Adapter trait for [`num_traits::Num`].
///
/// Provides basic numeric operations for any type implementing `Num`.
/// Automatically implemented via blanket impl.
///
/// # Methods
///
/// - `r_zero()` - Returns the additive identity (zero)
/// - `r_one()` - Returns the multiplicative identity (one)
/// - `r_is_zero()` - Check if value equals zero
/// - `r_is_one()` - Check if value equals one
///
/// # Example
///
/// ```
/// use miniextendr_api::RNum;
/// assert_eq!(<i128 as RNum>::r_zero(), 0);
/// ```
pub trait RNum: Clone {
    /// Get the additive identity (zero).
    fn r_zero() -> Self;

    /// Get the multiplicative identity (one).
    fn r_one() -> Self;

    /// Check if this value is zero.
    fn r_is_zero(&self) -> bool;

    /// Check if this value equals one.
    fn r_is_one(&self) -> bool;
}

impl<T> RNum for T
where
    T: Num + Clone,
{
    fn r_zero() -> Self {
        T::zero()
    }

    fn r_one() -> Self {
        T::one()
    }

    fn r_is_zero(&self) -> bool {
        Zero::is_zero(self)
    }

    fn r_is_one(&self) -> bool {
        self.clone() == T::one()
    }
}

/// Adapter trait for [`num_traits::Signed`].
///
/// Provides signed number operations for any type implementing `Signed`.
/// Automatically implemented via blanket impl.
///
/// # Methods
///
/// - `r_abs()` - Absolute value
/// - `r_signum()` - Sign of the number (-1, 0, or 1)
/// - `r_is_positive()` - Check if positive
/// - `r_is_negative()` - Check if negative
///
/// # Example
///
/// ```
/// use miniextendr_api::RSigned;
/// assert_eq!(RSigned::r_abs(&-5i64), 5);
/// ```
pub trait RSigned: Clone {
    /// Get the absolute value.
    fn r_abs(&self) -> Self;

    /// Get the sign of the number.
    ///
    /// Returns:
    /// - `-1` if negative
    /// - `0` if zero
    /// - `1` if positive
    fn r_signum(&self) -> Self;

    /// Check if the value is positive.
    fn r_is_positive(&self) -> bool;

    /// Check if the value is negative.
    fn r_is_negative(&self) -> bool;
}

impl<T> RSigned for T
where
    T: Signed + Clone,
{
    fn r_abs(&self) -> Self {
        Signed::abs(self)
    }

    fn r_signum(&self) -> Self {
        Signed::signum(self)
    }

    fn r_is_positive(&self) -> bool {
        Signed::is_positive(self)
    }

    fn r_is_negative(&self) -> bool {
        Signed::is_negative(self)
    }
}

/// Adapter trait for [`num_traits::Float`].
///
/// Provides floating-point operations for any type implementing `Float`.
/// Automatically implemented via blanket impl.
///
/// # Methods
///
/// ## Classification
/// - `r_is_nan()` - Check if NaN
/// - `r_is_infinite()` - Check if infinite
/// - `r_is_finite()` - Check if finite (not NaN or infinite)
/// - `r_is_normal()` - Check if normal (not zero, subnormal, infinite, or NaN)
/// - `r_is_sign_positive()` - Check if sign bit is positive
/// - `r_is_sign_negative()` - Check if sign bit is negative
///
/// ## Rounding
/// - `r_floor()` - Round towards negative infinity
/// - `r_ceil()` - Round towards positive infinity
/// - `r_round()` - Round to nearest integer
/// - `r_trunc()` - Round towards zero
/// - `r_fract()` - Fractional part
///
/// ## Mathematical
/// - `r_abs()` - Absolute value
/// - `r_sqrt()` - Square root
/// - `r_cbrt()` - Cube root
/// - `r_exp()` - Exponential (e^x)
/// - `r_exp2()` - 2^x
/// - `r_ln()` - Natural logarithm
/// - `r_log2()` - Base-2 logarithm
/// - `r_log10()` - Base-10 logarithm
/// - `r_sin()`, `r_cos()`, `r_tan()` - Trigonometric functions
/// - `r_asin()`, `r_acos()`, `r_atan()` - Inverse trigonometric
/// - `r_sinh()`, `r_cosh()`, `r_tanh()` - Hyperbolic functions
///
/// ## Special values
/// - `r_infinity()` - Positive infinity
/// - `r_neg_infinity()` - Negative infinity
/// - `r_nan()` - Not a Number
/// - `r_min_value()` - Smallest finite value
/// - `r_max_value()` - Largest finite value
/// - `r_epsilon()` - Machine epsilon
///
/// # Example
///
/// ```
/// use miniextendr_api::RFloat;
/// assert_eq!(RFloat::r_sqrt(&4.0f64), 2.0);
/// ```
pub trait RFloat: Clone {
    // Classification
    /// Check if the value is NaN.
    fn r_is_nan(&self) -> bool;

    /// Check if the value is infinite.
    fn r_is_infinite(&self) -> bool;

    /// Check if the value is finite (not NaN or infinite).
    fn r_is_finite(&self) -> bool;

    /// Check if the value is normal (not zero, subnormal, infinite, or NaN).
    fn r_is_normal(&self) -> bool;

    /// Check if the sign bit is positive.
    fn r_is_sign_positive(&self) -> bool;

    /// Check if the sign bit is negative.
    fn r_is_sign_negative(&self) -> bool;

    // Rounding
    /// Round towards negative infinity.
    fn r_floor(&self) -> Self;

    /// Round towards positive infinity.
    fn r_ceil(&self) -> Self;

    /// Round to nearest integer.
    fn r_round(&self) -> Self;

    /// Round towards zero (truncate).
    fn r_trunc(&self) -> Self;

    /// Get the fractional part.
    fn r_fract(&self) -> Self;

    // Basic math
    /// Get the absolute value.
    fn r_abs(&self) -> Self;

    /// Get the sign of the number (1.0, -1.0, or NaN).
    fn r_signum(&self) -> Self;

    /// Compute the square root.
    fn r_sqrt(&self) -> Self;

    /// Compute the cube root.
    fn r_cbrt(&self) -> Self;

    // Exponentials and logarithms
    /// Compute e^x.
    fn r_exp(&self) -> Self;

    /// Compute 2^x.
    fn r_exp2(&self) -> Self;

    /// Compute the natural logarithm.
    fn r_ln(&self) -> Self;

    /// Compute the base-2 logarithm.
    fn r_log2(&self) -> Self;

    /// Compute the base-10 logarithm.
    fn r_log10(&self) -> Self;

    // Trigonometric
    /// Compute sine.
    fn r_sin(&self) -> Self;

    /// Compute cosine.
    fn r_cos(&self) -> Self;

    /// Compute tangent.
    fn r_tan(&self) -> Self;

    /// Compute arcsine.
    fn r_asin(&self) -> Self;

    /// Compute arccosine.
    fn r_acos(&self) -> Self;

    /// Compute arctangent.
    fn r_atan(&self) -> Self;

    // Hyperbolic
    /// Compute hyperbolic sine.
    fn r_sinh(&self) -> Self;

    /// Compute hyperbolic cosine.
    fn r_cosh(&self) -> Self;

    /// Compute hyperbolic tangent.
    fn r_tanh(&self) -> Self;

    /// Compute inverse hyperbolic sine.
    fn r_asinh(&self) -> Self;

    /// Compute inverse hyperbolic cosine.
    fn r_acosh(&self) -> Self;

    /// Compute inverse hyperbolic tangent.
    fn r_atanh(&self) -> Self;

    // Special values
    /// Get positive infinity.
    fn r_infinity() -> Self;

    /// Get negative infinity.
    fn r_neg_infinity() -> Self;

    /// Get NaN.
    fn r_nan() -> Self;

    /// Get the smallest finite value.
    fn r_min_value() -> Self;

    /// Get the largest finite value.
    fn r_max_value() -> Self;

    /// Get the machine epsilon.
    fn r_epsilon() -> Self;

    // Power and other operations
    /// Compute x^n for integer n.
    fn r_powi(&self, n: i32) -> Self;

    /// Compute x^y for float y.
    fn r_powf(&self, y: &Self) -> Self;

    /// Compute the reciprocal (1/x).
    fn r_recip(&self) -> Self;
}

impl<T> RFloat for T
where
    T: Float,
{
    fn r_is_nan(&self) -> bool {
        Float::is_nan(*self)
    }

    fn r_is_infinite(&self) -> bool {
        Float::is_infinite(*self)
    }

    fn r_is_finite(&self) -> bool {
        Float::is_finite(*self)
    }

    fn r_is_normal(&self) -> bool {
        Float::is_normal(*self)
    }

    fn r_is_sign_positive(&self) -> bool {
        Float::is_sign_positive(*self)
    }

    fn r_is_sign_negative(&self) -> bool {
        Float::is_sign_negative(*self)
    }

    fn r_floor(&self) -> Self {
        Float::floor(*self)
    }

    fn r_ceil(&self) -> Self {
        Float::ceil(*self)
    }

    fn r_round(&self) -> Self {
        Float::round(*self)
    }

    fn r_trunc(&self) -> Self {
        Float::trunc(*self)
    }

    fn r_fract(&self) -> Self {
        Float::fract(*self)
    }

    fn r_abs(&self) -> Self {
        Float::abs(*self)
    }

    fn r_signum(&self) -> Self {
        Float::signum(*self)
    }

    fn r_sqrt(&self) -> Self {
        Float::sqrt(*self)
    }

    fn r_cbrt(&self) -> Self {
        Float::cbrt(*self)
    }

    fn r_exp(&self) -> Self {
        Float::exp(*self)
    }

    fn r_exp2(&self) -> Self {
        Float::exp2(*self)
    }

    fn r_ln(&self) -> Self {
        Float::ln(*self)
    }

    fn r_log2(&self) -> Self {
        Float::log2(*self)
    }

    fn r_log10(&self) -> Self {
        Float::log10(*self)
    }

    fn r_sin(&self) -> Self {
        Float::sin(*self)
    }

    fn r_cos(&self) -> Self {
        Float::cos(*self)
    }

    fn r_tan(&self) -> Self {
        Float::tan(*self)
    }

    fn r_asin(&self) -> Self {
        Float::asin(*self)
    }

    fn r_acos(&self) -> Self {
        Float::acos(*self)
    }

    fn r_atan(&self) -> Self {
        Float::atan(*self)
    }

    fn r_sinh(&self) -> Self {
        Float::sinh(*self)
    }

    fn r_cosh(&self) -> Self {
        Float::cosh(*self)
    }

    fn r_tanh(&self) -> Self {
        Float::tanh(*self)
    }

    fn r_asinh(&self) -> Self {
        Float::asinh(*self)
    }

    fn r_acosh(&self) -> Self {
        Float::acosh(*self)
    }

    fn r_atanh(&self) -> Self {
        Float::atanh(*self)
    }

    fn r_infinity() -> Self {
        Float::infinity()
    }

    fn r_neg_infinity() -> Self {
        Float::neg_infinity()
    }

    fn r_nan() -> Self {
        Float::nan()
    }

    fn r_min_value() -> Self {
        Float::min_value()
    }

    fn r_max_value() -> Self {
        Float::max_value()
    }

    fn r_epsilon() -> Self {
        Float::epsilon()
    }

    fn r_powi(&self, n: i32) -> Self {
        Float::powi(*self, n)
    }

    fn r_powf(&self, y: &Self) -> Self {
        Float::powf(*self, *y)
    }

    fn r_recip(&self) -> Self {
        Float::recip(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rnum_i32() {
        assert_eq!(<i32 as RNum>::r_zero(), 0);
        assert_eq!(<i32 as RNum>::r_one(), 1);
        assert!(RNum::r_is_zero(&0i32));
        assert!(!RNum::r_is_zero(&1i32));
        assert!(RNum::r_is_one(&1i32));
        assert!(!RNum::r_is_one(&2i32));
    }

    #[test]
    fn test_rnum_f64() {
        assert_eq!(<f64 as RNum>::r_zero(), 0.0);
        assert_eq!(<f64 as RNum>::r_one(), 1.0);
        assert!(RNum::r_is_zero(&0.0f64));
        assert!(!RNum::r_is_zero(&0.1f64));
    }

    #[test]
    fn test_rsigned_i32() {
        assert_eq!((-5i32).r_abs(), 5);
        assert_eq!(5i32.r_abs(), 5);
        assert_eq!((-5i32).r_signum(), -1);
        assert_eq!(0i32.r_signum(), 0);
        assert_eq!(5i32.r_signum(), 1);
        assert!(5i32.r_is_positive());
        assert!(!(-5i32).r_is_positive());
        assert!((-5i32).r_is_negative());
        assert!(!5i32.r_is_negative());
    }

    #[test]
    fn test_rsigned_f64() {
        // Use explicit trait qualification since f64 implements both RSigned and RFloat
        assert_eq!(RSigned::r_abs(&-std::f64::consts::PI), std::f64::consts::PI);
        assert_eq!(RSigned::r_signum(&-std::f64::consts::PI), -1.0);
        assert_eq!(RSigned::r_signum(&std::f64::consts::PI), 1.0);
    }

    #[test]
    fn test_rfloat_classification() {
        assert!(f64::NAN.r_is_nan());
        assert!(!1.0f64.r_is_nan());
        assert!(f64::INFINITY.r_is_infinite());
        assert!(f64::NEG_INFINITY.r_is_infinite());
        assert!(!1.0f64.r_is_infinite());
        assert!(1.0f64.r_is_finite());
        assert!(!f64::INFINITY.r_is_finite());
        assert!(!f64::NAN.r_is_finite());
        assert!(1.0f64.r_is_normal());
        assert!(!0.0f64.r_is_normal());
    }

    #[test]
    fn test_rfloat_sign() {
        assert!(1.0f64.r_is_sign_positive());
        assert!(!(-1.0f64).r_is_sign_positive());
        assert!((-1.0f64).r_is_sign_negative());
        assert!(!1.0f64.r_is_sign_negative());
    }

    #[test]
    fn test_rfloat_rounding() {
        assert_eq!(3.7f64.r_floor(), 3.0);
        assert_eq!((-3.7f64).r_floor(), -4.0);
        assert_eq!(3.2f64.r_ceil(), 4.0);
        assert_eq!((-3.2f64).r_ceil(), -3.0);
        assert_eq!(3.5f64.r_round(), 4.0);
        assert_eq!(3.4f64.r_round(), 3.0);
        assert_eq!(3.7f64.r_trunc(), 3.0);
        assert_eq!((-3.7f64).r_trunc(), -3.0);
        let fract = 3.7f64.r_fract();
        assert!((fract - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_rfloat_math() {
        // Use explicit trait qualification for abs/signum since f64 implements both RSigned and RFloat
        assert_eq!(RFloat::r_abs(&-3.0f64), 3.0);
        assert_eq!(RFloat::r_signum(&-1.0f64), -1.0);
        assert_eq!(RFloat::r_signum(&1.0f64), 1.0);
        assert_eq!(4.0f64.r_sqrt(), 2.0);
        assert_eq!(8.0f64.r_cbrt(), 2.0);
    }

    #[test]
    fn test_rfloat_exp_log() {
        assert!((1.0f64.r_exp() - std::f64::consts::E).abs() < 1e-10);
        assert_eq!(3.0f64.r_exp2(), 8.0);
        assert!((std::f64::consts::E.r_ln() - 1.0).abs() < 1e-10);
        assert_eq!(8.0f64.r_log2(), 3.0);
        assert_eq!(100.0f64.r_log10(), 2.0);
    }

    #[test]
    fn test_rfloat_trig() {
        let pi = std::f64::consts::PI;
        assert!(0.0f64.r_sin().abs() < 1e-10);
        assert!((1.0f64 - 0.0f64.r_cos()).abs() < 1e-10);
        assert!(0.0f64.r_tan().abs() < 1e-10);
        assert!((0.0f64.r_asin()).abs() < 1e-10);
        assert!((1.0f64.r_acos()).abs() < 1e-10);
        assert!((pi / 4.0 - 1.0f64.r_atan()).abs() < 1e-10);
    }

    #[test]
    fn test_rfloat_hyperbolic() {
        assert!(0.0f64.r_sinh().abs() < 1e-10);
        assert!((1.0 - 0.0f64.r_cosh()).abs() < 1e-10);
        assert!(0.0f64.r_tanh().abs() < 1e-10);
        assert!(0.0f64.r_asinh().abs() < 1e-10);
        assert!(1.0f64.r_acosh().abs() < 1e-10);
        assert!(0.0f64.r_atanh().abs() < 1e-10);
    }

    #[test]
    fn test_rfloat_special_values() {
        assert!(<f64 as RFloat>::r_infinity().r_is_infinite());
        assert!(<f64 as RFloat>::r_infinity() > 0.0);
        assert!(<f64 as RFloat>::r_neg_infinity().r_is_infinite());
        assert!(<f64 as RFloat>::r_neg_infinity() < 0.0);
        assert!(<f64 as RFloat>::r_nan().r_is_nan());
        assert!(<f64 as RFloat>::r_min_value() < 0.0);
        assert!(<f64 as RFloat>::r_max_value() > 0.0);
        assert!(<f64 as RFloat>::r_epsilon() > 0.0);
        assert!(<f64 as RFloat>::r_epsilon() < 1e-10);
    }

    #[test]
    fn test_rfloat_power() {
        assert_eq!(2.0f64.r_powi(3), 8.0);
        assert_eq!(2.0f64.r_powi(-1), 0.5);
        assert_eq!(<f64 as RFloat>::r_powf(&2.0f64, &3.0), 8.0);
        assert_eq!(<f64 as RFloat>::r_powf(&4.0f64, &0.5), 2.0);
        assert_eq!(<f64 as RFloat>::r_recip(&2.0f64), 0.5);
    }

    #[test]
    fn test_rfloat_f32() {
        // Verify f32 also works
        assert_eq!(<f32 as RNum>::r_zero(), 0.0f32);
        assert_eq!(<f32 as RNum>::r_one(), 1.0f32);
        assert!(f32::NAN.r_is_nan());
        assert_eq!(4.0f32.r_sqrt(), 2.0f32);
    }
}
