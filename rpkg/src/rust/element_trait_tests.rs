//! Fixtures for a hand-written `TryFromSexpElement`: a user type that
//! implements the element trait itself (no derive), and so converts as
//! `Vec<T>`, `Option<T>` and `Vec<Option<T>>` arguments.
//!
//! `Celsius` reads `f64`'s containers and wraps each value; its check
//! refuses a vector of class `fahrenheit` before anything is read, with its
//! own condition classes.
//!
//! See `tests/testthat/test-element-trait.R`.

use miniextendr_api::condition::RError;
use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::{SEXP, SexpExt, TryFromSexpElement, miniextendr};

/// A temperature in degrees Celsius.
pub struct Celsius(f64);

impl TryFromSexpElement for Celsius {
    type Inner = f64;

    fn from_inner(inner: f64) -> Self {
        Celsius(inner)
    }

    fn check_sexp(sexp: SEXP) -> Result<(), SexpError> {
        if sexp.inherits_class(c"fahrenheit") {
            return Err(
                RError::new("temperatures in Fahrenheit: convert them to Celsius first")
                    .class(["temperature_unit_error", "temperature_error"])
                    .data("unit", "fahrenheit")
                    .into(),
            );
        }
        Ok(())
    }
}

/// The scalar follows the forwarding rule: the check, then `f64`'s
/// conversion, then `from_inner`. No container calls it.
impl TryFromSexp for Celsius {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        Self::check_sexp(sexp)?;
        f64::try_from_sexp(sexp).map(Self::from_inner)
    }
}

/// The mean of `x`, `NA` for no values.
/// @param x Numeric vector of temperatures in Celsius.
#[miniextendr(noexport)]
pub fn celsius_mean(x: Vec<Celsius>) -> Option<f64> {
    let n = i32::try_from(x.len()).expect("fewer than i32::MAX temperatures");
    (n > 0).then(|| x.iter().map(|c| c.0).sum::<f64>() / f64::from(n))
}

/// `x`, or 20 (room temperature) when `x` is `NULL` or `NA`.
/// @param x A temperature in Celsius, `NULL` or `NA`.
#[miniextendr(noexport)]
pub fn celsius_or_room(x: Option<Celsius>) -> f64 {
    x.map_or(20.0, |c| c.0)
}

/// How many of `x` are known (not `NA`).
/// @param x Numeric vector of temperatures in Celsius, with `NA`s.
#[miniextendr(noexport)]
pub fn celsius_known(x: Vec<Option<Celsius>>) -> i32 {
    i32::try_from(x.iter().flatten().count()).expect("fewer than i32::MAX temperatures")
}

/// A temperature in Celsius, as a scalar argument.
/// @param x A temperature in Celsius.
#[miniextendr(noexport)]
pub fn celsius_scalar(x: Celsius) -> f64 {
    x.0
}
