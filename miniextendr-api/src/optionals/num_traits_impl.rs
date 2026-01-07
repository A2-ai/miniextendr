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
//! # Example
//!
//! ```ignore
//! use miniextendr_api::prelude::*;
//! use miniextendr_api::num_traits_impl::{RNum, RFloat};
//!
//! #[derive(ExternalPtr)]
//! struct MyNumber(f64);
//!
//! // RNum and RFloat are automatically available via blanket impls
//! #[miniextendr]
//! impl RNum for MyNumber {}
//!
//! #[miniextendr]
//! impl RFloat for MyNumber {}
//! ```
//!
//! In R:
//! ```r
//! x <- MyNumber$new(3.14)
//! x$r_is_zero()      # FALSE
//! x$r_floor()        # 3.0
//! x$r_ceil()         # 4.0
//! x$r_is_finite()    # TRUE
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
/// ```ignore
/// use num_traits::Num;
///
/// #[derive(ExternalPtr)]
/// struct BigNum(i128);
///
/// // Blanket impl provides all methods automatically
/// #[miniextendr]
/// impl RNum for BigNum {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RNum for BigNum;
/// }
/// ```
pub trait RNum: Clone {
    /// Get the additive identity (zero).
    fn zero() -> Self;

    /// Get the multiplicative identity (one).
    fn one() -> Self;

    /// Check if this value is zero.
    fn is_zero(&self) -> bool;

    /// Check if this value equals one.
    fn is_one(&self) -> bool;
}

impl<T> RNum for T
where
    T: Num + Clone,
{
    fn zero() -> Self {
        T::zero()
    }

    fn one() -> Self {
        T::one()
    }

    fn is_zero(&self) -> bool {
        Zero::is_zero(self)
    }

    fn is_one(&self) -> bool {
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
/// ```ignore
/// #[derive(ExternalPtr)]
/// struct SignedInt(i64);
///
/// #[miniextendr]
/// impl RSigned for SignedInt {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RSigned for SignedInt;
/// }
/// ```
pub trait RSigned: Clone {
    /// Get the absolute value.
    fn abs(&self) -> Self;

    /// Get the sign of the number.
    ///
    /// Returns:
    /// - `-1` if negative
    /// - `0` if zero
    /// - `1` if positive
    fn signum(&self) -> Self;

    /// Check if the value is positive.
    fn is_positive(&self) -> bool;

    /// Check if the value is negative.
    fn is_negative(&self) -> bool;
}

impl<T> RSigned for T
where
    T: Signed + Clone,
{
    fn abs(&self) -> Self {
        Signed::abs(self)
    }

    fn signum(&self) -> Self {
        Signed::signum(self)
    }

    fn is_positive(&self) -> bool {
        Signed::is_positive(self)
    }

    fn is_negative(&self) -> bool {
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
/// ```ignore
/// #[derive(ExternalPtr)]
/// struct MyFloat(f64);
///
/// #[miniextendr]
/// impl RFloat for MyFloat {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RFloat for MyFloat;
/// }
/// ```
///
/// In R:
/// ```r
/// x <- MyFloat$new(3.7)
/// x$r_floor()        # 3.0
/// x$r_ceil()         # 4.0
/// x$r_is_finite()    # TRUE
/// x$r_sqrt()         # 1.923538
/// ```
pub trait RFloat: Clone {
    // Classification
    /// Check if the value is NaN.
    fn is_nan(&self) -> bool;

    /// Check if the value is infinite.
    fn is_infinite(&self) -> bool;

    /// Check if the value is finite (not NaN or infinite).
    fn is_finite(&self) -> bool;

    /// Check if the value is normal (not zero, subnormal, infinite, or NaN).
    fn is_normal(&self) -> bool;

    /// Check if the sign bit is positive.
    fn is_sign_positive(&self) -> bool;

    /// Check if the sign bit is negative.
    fn is_sign_negative(&self) -> bool;

    // Rounding
    /// Round towards negative infinity.
    fn floor(&self) -> Self;

    /// Round towards positive infinity.
    fn ceil(&self) -> Self;

    /// Round to nearest integer.
    fn round(&self) -> Self;

    /// Round towards zero (truncate).
    fn trunc(&self) -> Self;

    /// Get the fractional part.
    fn fract(&self) -> Self;

    // Basic math
    /// Get the absolute value.
    fn abs(&self) -> Self;

    /// Get the sign of the number (1.0, -1.0, or NaN).
    fn signum(&self) -> Self;

    /// Compute the square root.
    fn sqrt(&self) -> Self;

    /// Compute the cube root.
    fn cbrt(&self) -> Self;

    // Exponentials and logarithms
    /// Compute e^x.
    fn exp(&self) -> Self;

    /// Compute 2^x.
    fn exp2(&self) -> Self;

    /// Compute the natural logarithm.
    fn ln(&self) -> Self;

    /// Compute the base-2 logarithm.
    fn log2(&self) -> Self;

    /// Compute the base-10 logarithm.
    fn log10(&self) -> Self;

    // Trigonometric
    /// Compute sine.
    fn sin(&self) -> Self;

    /// Compute cosine.
    fn cos(&self) -> Self;

    /// Compute tangent.
    fn tan(&self) -> Self;

    /// Compute arcsine.
    fn asin(&self) -> Self;

    /// Compute arccosine.
    fn acos(&self) -> Self;

    /// Compute arctangent.
    fn atan(&self) -> Self;

    // Hyperbolic
    /// Compute hyperbolic sine.
    fn sinh(&self) -> Self;

    /// Compute hyperbolic cosine.
    fn cosh(&self) -> Self;

    /// Compute hyperbolic tangent.
    fn tanh(&self) -> Self;

    /// Compute inverse hyperbolic sine.
    fn asinh(&self) -> Self;

    /// Compute inverse hyperbolic cosine.
    fn acosh(&self) -> Self;

    /// Compute inverse hyperbolic tangent.
    fn atanh(&self) -> Self;

    // Special values
    /// Get positive infinity.
    fn infinity() -> Self;

    /// Get negative infinity.
    fn neg_infinity() -> Self;

    /// Get NaN.
    fn nan() -> Self;

    /// Get the smallest finite value.
    fn min_value() -> Self;

    /// Get the largest finite value.
    fn max_value() -> Self;

    /// Get the machine epsilon.
    fn epsilon() -> Self;

    // Power and other operations
    /// Compute x^n for integer n.
    fn powi(&self, n: i32) -> Self;

    /// Compute x^y for float y.
    fn powf(&self, y: &Self) -> Self;

    /// Compute the reciprocal (1/x).
    fn recip(&self) -> Self;
}

impl<T> RFloat for T
where
    T: Float,
{
    fn is_nan(&self) -> bool {
        Float::is_nan(*self)
    }

    fn is_infinite(&self) -> bool {
        Float::is_infinite(*self)
    }

    fn is_finite(&self) -> bool {
        Float::is_finite(*self)
    }

    fn is_normal(&self) -> bool {
        Float::is_normal(*self)
    }

    fn is_sign_positive(&self) -> bool {
        Float::is_sign_positive(*self)
    }

    fn is_sign_negative(&self) -> bool {
        Float::is_sign_negative(*self)
    }

    fn floor(&self) -> Self {
        Float::floor(*self)
    }

    fn ceil(&self) -> Self {
        Float::ceil(*self)
    }

    fn round(&self) -> Self {
        Float::round(*self)
    }

    fn trunc(&self) -> Self {
        Float::trunc(*self)
    }

    fn fract(&self) -> Self {
        Float::fract(*self)
    }

    fn abs(&self) -> Self {
        Float::abs(*self)
    }

    fn signum(&self) -> Self {
        Float::signum(*self)
    }

    fn sqrt(&self) -> Self {
        Float::sqrt(*self)
    }

    fn cbrt(&self) -> Self {
        Float::cbrt(*self)
    }

    fn exp(&self) -> Self {
        Float::exp(*self)
    }

    fn exp2(&self) -> Self {
        Float::exp2(*self)
    }

    fn ln(&self) -> Self {
        Float::ln(*self)
    }

    fn log2(&self) -> Self {
        Float::log2(*self)
    }

    fn log10(&self) -> Self {
        Float::log10(*self)
    }

    fn sin(&self) -> Self {
        Float::sin(*self)
    }

    fn cos(&self) -> Self {
        Float::cos(*self)
    }

    fn tan(&self) -> Self {
        Float::tan(*self)
    }

    fn asin(&self) -> Self {
        Float::asin(*self)
    }

    fn acos(&self) -> Self {
        Float::acos(*self)
    }

    fn atan(&self) -> Self {
        Float::atan(*self)
    }

    fn sinh(&self) -> Self {
        Float::sinh(*self)
    }

    fn cosh(&self) -> Self {
        Float::cosh(*self)
    }

    fn tanh(&self) -> Self {
        Float::tanh(*self)
    }

    fn asinh(&self) -> Self {
        Float::asinh(*self)
    }

    fn acosh(&self) -> Self {
        Float::acosh(*self)
    }

    fn atanh(&self) -> Self {
        Float::atanh(*self)
    }

    fn infinity() -> Self {
        Float::infinity()
    }

    fn neg_infinity() -> Self {
        Float::neg_infinity()
    }

    fn nan() -> Self {
        Float::nan()
    }

    fn min_value() -> Self {
        Float::min_value()
    }

    fn max_value() -> Self {
        Float::max_value()
    }

    fn epsilon() -> Self {
        Float::epsilon()
    }

    fn powi(&self, n: i32) -> Self {
        Float::powi(*self, n)
    }

    fn powf(&self, y: &Self) -> Self {
        Float::powf(*self, *y)
    }

    fn recip(&self) -> Self {
        Float::recip(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rnum_i32() {
        assert_eq!(<i32 as RNum>::zero(), 0);
        assert_eq!(<i32 as RNum>::one(), 1);
        assert!(RNum::is_zero(&0i32));
        assert!(!RNum::is_zero(&1i32));
        assert!(RNum::is_one(&1i32));
        assert!(!RNum::is_one(&2i32));
    }

    #[test]
    fn test_rnum_f64() {
        assert_eq!(<f64 as RNum>::zero(), 0.0);
        assert_eq!(<f64 as RNum>::one(), 1.0);
        assert!(RNum::is_zero(&0.0f64));
        assert!(!RNum::is_zero(&0.1f64));
    }

    #[test]
    fn test_rsigned_i32() {
        assert_eq!((-5i32).abs(), 5);
        assert_eq!(5i32.abs(), 5);
        assert_eq!((-5i32).signum(), -1);
        assert_eq!(0i32.signum(), 0);
        assert_eq!(5i32.signum(), 1);
        assert!(5i32.is_positive());
        assert!(!(-5i32).is_positive());
        assert!((-5i32).is_negative());
        assert!(!5i32.is_negative());
    }

    #[test]
    fn test_rsigned_f64() {
        // Use explicit trait qualification since f64 implements both RSigned and RFloat
        assert_eq!(RSigned::abs(&-std::f64::consts::PI), std::f64::consts::PI);
        assert_eq!(RSigned::signum(&-std::f64::consts::PI), -1.0);
        assert_eq!(RSigned::signum(&std::f64::consts::PI), 1.0);
    }

    #[test]
    fn test_rfloat_classification() {
        assert!(f64::NAN.is_nan());
        assert!(!1.0f64.is_nan());
        assert!(f64::INFINITY.is_infinite());
        assert!(f64::NEG_INFINITY.is_infinite());
        assert!(!1.0f64.is_infinite());
        assert!(1.0f64.is_finite());
        assert!(!f64::INFINITY.is_finite());
        assert!(!f64::NAN.is_finite());
        assert!(1.0f64.is_normal());
        assert!(!0.0f64.is_normal());
    }

    #[test]
    fn test_rfloat_sign() {
        assert!(1.0f64.is_sign_positive());
        assert!(!(-1.0f64).is_sign_positive());
        assert!((-1.0f64).is_sign_negative());
        assert!(!1.0f64.is_sign_negative());
    }

    #[test]
    fn test_rfloat_rounding() {
        assert_eq!(3.7f64.floor(), 3.0);
        assert_eq!((-3.7f64).floor(), -4.0);
        assert_eq!(3.2f64.ceil(), 4.0);
        assert_eq!((-3.2f64).ceil(), -3.0);
        assert_eq!(3.5f64.round(), 4.0);
        assert_eq!(3.4f64.round(), 3.0);
        assert_eq!(3.7f64.trunc(), 3.0);
        assert_eq!((-3.7f64).trunc(), -3.0);
        let fract = 3.7f64.fract();
        assert!((fract - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_rfloat_math() {
        // Use explicit trait qualification for abs/signum since f64 implements both RSigned and RFloat
        assert_eq!(RFloat::abs(&-3.0f64), 3.0);
        assert_eq!(RFloat::signum(&-1.0f64), -1.0);
        assert_eq!(RFloat::signum(&1.0f64), 1.0);
        assert_eq!(4.0f64.sqrt(), 2.0);
        assert_eq!(8.0f64.cbrt(), 2.0);
    }

    #[test]
    fn test_rfloat_exp_log() {
        assert!((1.0f64.exp() - std::f64::consts::E).abs() < 1e-10);
        assert_eq!(3.0f64.exp2(), 8.0);
        assert!((std::f64::consts::E.ln() - 1.0).abs() < 1e-10);
        assert_eq!(8.0f64.log2(), 3.0);
        assert_eq!(100.0f64.log10(), 2.0);
    }

    #[test]
    fn test_rfloat_trig() {
        let pi = std::f64::consts::PI;
        assert!(0.0f64.sin().abs() < 1e-10);
        assert!((1.0f64 - 0.0f64.cos()).abs() < 1e-10);
        assert!(0.0f64.tan().abs() < 1e-10);
        assert!((0.0f64.asin()).abs() < 1e-10);
        assert!((1.0f64.acos()).abs() < 1e-10);
        assert!((pi / 4.0 - 1.0f64.atan()).abs() < 1e-10);
    }

    #[test]
    fn test_rfloat_hyperbolic() {
        assert!(0.0f64.sinh().abs() < 1e-10);
        assert!((1.0 - 0.0f64.cosh()).abs() < 1e-10);
        assert!(0.0f64.tanh().abs() < 1e-10);
        assert!(0.0f64.asinh().abs() < 1e-10);
        assert!(1.0f64.acosh().abs() < 1e-10);
        assert!(0.0f64.atanh().abs() < 1e-10);
    }

    #[test]
    fn test_rfloat_special_values() {
        assert!(<f64 as RFloat>::infinity().is_infinite());
        assert!(<f64 as RFloat>::infinity() > 0.0);
        assert!(<f64 as RFloat>::neg_infinity().is_infinite());
        assert!(<f64 as RFloat>::neg_infinity() < 0.0);
        assert!(<f64 as RFloat>::nan().is_nan());
        assert!(<f64 as RFloat>::min_value() < 0.0);
        assert!(<f64 as RFloat>::max_value() > 0.0);
        assert!(<f64 as RFloat>::epsilon() > 0.0);
        assert!(<f64 as RFloat>::epsilon() < 1e-10);
    }

    #[test]
    fn test_rfloat_power() {
        assert_eq!(2.0f64.powi(3), 8.0);
        assert_eq!(2.0f64.powi(-1), 0.5);
        assert_eq!(<f64 as RFloat>::powf(&2.0f64, &3.0), 8.0);
        assert_eq!(<f64 as RFloat>::powf(&4.0f64, &0.5), 2.0);
        assert_eq!(<f64 as RFloat>::recip(&2.0f64), 0.5);
    }

    #[test]
    fn test_rfloat_f32() {
        // Verify f32 also works
        assert_eq!(<f32 as RNum>::zero(), 0.0f32);
        assert_eq!(<f32 as RNum>::one(), 1.0f32);
        assert!(f32::NAN.is_nan());
        assert_eq!(4.0f32.sqrt(), 2.0f32);
    }
}
