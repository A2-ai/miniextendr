//! num-traits adapter tests
//!
//! Tests RNum, RSigned, and RFloat blanket impls by wrapping f64 operations.
use miniextendr_api::miniextendr;
use miniextendr_api::num_traits_impl::{RFloat, RNum, RSigned};

/// Test whether a numeric value is zero via RNum trait.
/// @param x Numeric scalar.
#[miniextendr]
pub fn num_is_zero(x: f64) -> bool {
    RNum::is_zero(&x)
}

/// Test whether a numeric value is one via RNum trait.
/// @param x Numeric scalar.
#[miniextendr]
pub fn num_is_one(x: f64) -> bool {
    RNum::is_one(&x)
}

/// Test computing the absolute value via RSigned trait.
/// @param x Numeric scalar.
#[miniextendr]
pub fn signed_abs(x: f64) -> f64 {
    RSigned::abs(&x)
}

/// Test computing the signum (-1, 0, or 1) via RSigned trait.
/// @param x Numeric scalar.
#[miniextendr]
pub fn signed_signum(x: f64) -> f64 {
    RSigned::signum(&x)
}

/// Test whether a numeric value is positive via RSigned trait.
/// @param x Numeric scalar.
#[miniextendr]
pub fn signed_is_positive(x: f64) -> bool {
    RSigned::is_positive(&x)
}

/// Test whether a numeric value is negative via RSigned trait.
/// @param x Numeric scalar.
#[miniextendr]
pub fn signed_is_negative(x: f64) -> bool {
    RSigned::is_negative(&x)
}

/// Test computing the floor of a float via RFloat trait.
/// @param x Numeric scalar.
#[miniextendr]
pub fn float_floor(x: f64) -> f64 {
    RFloat::floor(&x)
}

/// Test computing the ceiling of a float via RFloat trait.
/// @param x Numeric scalar.
#[miniextendr]
pub fn float_ceil(x: f64) -> f64 {
    RFloat::ceil(&x)
}

/// Test computing the square root via RFloat trait.
/// @param x Numeric scalar.
#[miniextendr]
pub fn float_sqrt(x: f64) -> f64 {
    RFloat::sqrt(&x)
}

/// Test whether a float is finite via RFloat trait.
/// @param x Numeric scalar.
#[miniextendr]
pub fn float_is_finite(x: f64) -> bool {
    RFloat::is_finite(&x)
}

/// Test whether a float is NaN via RFloat trait.
/// @param x Numeric scalar.
#[miniextendr]
pub fn float_is_nan(x: f64) -> bool {
    RFloat::is_nan(&x)
}

/// Test raising a float to an integer power via RFloat trait.
/// @param x Numeric base.
/// @param n Integer exponent.
#[miniextendr]
pub fn float_powi(x: f64, n: i32) -> f64 {
    RFloat::powi(&x, n)
}
