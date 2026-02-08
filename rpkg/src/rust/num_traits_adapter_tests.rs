//! num-traits adapter tests
//!
//! Tests RNum, RSigned, and RFloat blanket impls by wrapping f64 operations.
use miniextendr_api::num_traits_impl::{RFloat, RNum, RSigned};
use miniextendr_api::{miniextendr, miniextendr_module};

/// @noRd
#[miniextendr]
pub fn num_is_zero(x: f64) -> bool {
    RNum::is_zero(&x)
}

/// @noRd
#[miniextendr]
pub fn num_is_one(x: f64) -> bool {
    RNum::is_one(&x)
}

/// @noRd
#[miniextendr]
pub fn signed_abs(x: f64) -> f64 {
    RSigned::abs(&x)
}

/// @noRd
#[miniextendr]
pub fn signed_signum(x: f64) -> f64 {
    RSigned::signum(&x)
}

/// @noRd
#[miniextendr]
pub fn signed_is_positive(x: f64) -> bool {
    RSigned::is_positive(&x)
}

/// @noRd
#[miniextendr]
pub fn signed_is_negative(x: f64) -> bool {
    RSigned::is_negative(&x)
}

/// @noRd
#[miniextendr]
pub fn float_floor(x: f64) -> f64 {
    RFloat::floor(&x)
}

/// @noRd
#[miniextendr]
pub fn float_ceil(x: f64) -> f64 {
    RFloat::ceil(&x)
}

/// @noRd
#[miniextendr]
pub fn float_sqrt(x: f64) -> f64 {
    RFloat::sqrt(&x)
}

/// @noRd
#[miniextendr]
pub fn float_is_finite(x: f64) -> bool {
    RFloat::is_finite(&x)
}

/// @noRd
#[miniextendr]
pub fn float_is_nan(x: f64) -> bool {
    RFloat::is_nan(&x)
}

/// @noRd
#[miniextendr]
pub fn float_powi(x: f64, n: i32) -> f64 {
    RFloat::powi(&x, n)
}

miniextendr_module! {
    mod num_traits_adapter_tests;
    fn num_is_zero;
    fn num_is_one;
    fn signed_abs;
    fn signed_signum;
    fn signed_is_positive;
    fn signed_is_negative;
    fn float_floor;
    fn float_ceil;
    fn float_sqrt;
    fn float_is_finite;
    fn float_is_nan;
    fn float_powi;
}
