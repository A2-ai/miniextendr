//! Integration with the `num-complex` crate.
//!
//! Provides conversions between R complex vectors (`CPLXSXP`) and `Complex<f64>` types.
//!
//! | R Type | Rust Type | Notes |
//! |--------|-----------|-------|
//! | `complex(1)` | `Complex<f64>` | Single complex number |
//! | `complex` | `Vec<Complex<f64>>` | Vector of complex numbers |
//! | `NA_complex_` | `Option<Complex<f64>>` | NA maps to None |
//!
//! # Features
//!
//! Enable this module with the `num-complex` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["num-complex"] }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use num_complex::Complex;
//!
//! #[miniextendr]
//! fn add_complex(a: Complex<f64>, b: Complex<f64>) -> Complex<f64> {
//!     a + b
//! }
//!
//! #[miniextendr]
//! fn complex_magnitude(c: Complex<f64>) -> f64 {
//!     c.norm()
//! }
//! ```
//!
//! # NA Handling
//!
//! R's `NA_complex_` has both real and imaginary parts set to `NA_REAL` (a specific NaN).
//! This module treats a complex number as NA if **either** part is R's NA, decided by
//! R's `R_IsNA` rule (a NaN whose low word is 1954, so a computed NA such as
//! `NA_real_ * 1` counts). Other NaN values are preserved as valid values.

pub use num_complex::Complex;

use crate::altrep_traits::NA_REAL;
use crate::from_r::{SexpError, SexpLengthError, SexpNaError, SexpTypeError, TryFromSexp};
use crate::into_r::into_r_infallible;
use crate::{IsNA, Rcomplex, SEXP, SEXPTYPE, SexpExt};

// region: Helper functions

/// Convert `Complex<f64>` to R's `Rcomplex`.
#[inline]
pub fn to_rcomplex(c: Complex<f64>) -> Rcomplex {
    Rcomplex { r: c.re, i: c.im }
}

/// Convert R's `Rcomplex` to `Complex<f64>`.
#[inline]
pub fn from_rcomplex(r: Rcomplex) -> Complex<f64> {
    Complex::new(r.r, r.i)
}

/// Create an NA complex value (both parts are `NA_REAL`).
#[inline]
pub fn na_rcomplex() -> Rcomplex {
    Rcomplex {
        r: NA_REAL,
        i: NA_REAL,
    }
}

/// Check if a `Complex<f64>` value is NA (either part is R's NA, by `R_IsNA`).
#[inline]
pub fn is_na_complex(c: &Complex<f64>) -> bool {
    c.re.is_na() || c.im.is_na()
}
// endregion

// region: Scalar conversions

impl TryFromSexp for Complex<f64> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::CPLXSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::CPLXSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        if len != 1 {
            return Err(SexpError::Length(SexpLengthError {
                expected: 1,
                actual: len,
            }));
        }

        let rcomplex = sexp.complex_elt(0);
        if IsNA::is_na(&rcomplex) {
            return Err(SexpError::Na(SexpNaError {
                sexp_type: SEXPTYPE::CPLXSXP,
            }));
        }

        Ok(from_rcomplex(rcomplex))
    }
}

into_r_infallible!(Complex<f64>, |this| SEXP::scalar_complex(to_rcomplex(this)));
// endregion

// region: Option conversions (NA support)

impl TryFromSexp for Option<Complex<f64>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        // NULL -> None
        if actual == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        if actual != SEXPTYPE::CPLXSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::CPLXSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        if len != 1 {
            return Err(SexpError::Length(SexpLengthError {
                expected: 1,
                actual: len,
            }));
        }

        let rcomplex = sexp.complex_elt(0);
        if IsNA::is_na(&rcomplex) {
            Ok(None)
        } else {
            Ok(Some(from_rcomplex(rcomplex)))
        }
    }
}

into_r_infallible!(Option<Complex<f64>>, |this| match this {
    Some(c) => SEXP::scalar_complex(to_rcomplex(c)),
    None => SEXP::scalar_complex(na_rcomplex()),
});
// endregion

// region: Vector conversions

impl TryFromSexp for Vec<Complex<f64>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::CPLXSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::CPLXSXP,
                actual,
            }
            .into());
        }

        let slice: &[Rcomplex] = unsafe { SexpExt::as_slice(&sexp) };
        let mut result = Vec::with_capacity(slice.len());
        let mut errors = crate::from_r::BatchedErrors::default();

        for (i, rcomplex) in slice.iter().enumerate() {
            if IsNA::is_na(rcomplex) {
                errors.push(i, || "NA is not allowed".to_string());
                continue;
            }
            result.push(from_rcomplex(*rcomplex));
        }
        if !errors.is_empty() {
            return Err(errors.into_element_error());
        }

        Ok(result)
    }
}

into_r_infallible!(Vec<Complex<f64>>, |this| {
    use crate::sys::Rf_allocVector;

    let len = this.len();
    let sexp = unsafe { Rf_allocVector(SEXPTYPE::CPLXSXP, len as crate::R_xlen_t) };
    let dst: &mut [Rcomplex] = unsafe { SexpExt::as_mut_slice(&sexp) };

    for (i, c) in this.into_iter().enumerate() {
        dst[i] = to_rcomplex(c);
    }

    sexp
});
// endregion

// region: Vec<Option<Complex>> conversions (NA-aware vectors)

impl TryFromSexp for Vec<Option<Complex<f64>>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::CPLXSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::CPLXSXP,
                actual,
            }
            .into());
        }

        let slice: &[Rcomplex] = unsafe { SexpExt::as_slice(&sexp) };
        let result = slice
            .iter()
            .map(|rcomplex| {
                if IsNA::is_na(rcomplex) {
                    None
                } else {
                    Some(from_rcomplex(*rcomplex))
                }
            })
            .collect();

        Ok(result)
    }
}

into_r_infallible!(Vec<Option<Complex<f64>>>, |this| {
    use crate::sys::Rf_allocVector;

    let len = this.len();
    let sexp = unsafe { Rf_allocVector(SEXPTYPE::CPLXSXP, len as crate::R_xlen_t) };
    let dst: &mut [Rcomplex] = unsafe { SexpExt::as_mut_slice(&sexp) };

    for (i, opt) in this.into_iter().enumerate() {
        dst[i] = match opt {
            Some(c) => to_rcomplex(c),
            None => na_rcomplex(),
        };
    }

    sexp
});
// endregion

// region: RComplexOps adapter trait

/// Adapter trait for [`Complex<f64>`] operations.
///
/// Provides complex number inspection methods from R.
/// Automatically implemented for `Complex<f64>`.
///
/// # Example
///
/// ```rust,ignore
/// use num_complex::Complex;
/// use miniextendr_api::num_complex_impl::RComplexOps;
///
/// #[derive(ExternalPtr)]
/// struct MyComplex(Complex<f64>);
///
/// #[miniextendr]
/// impl RComplexOps for MyComplex {}
/// ```
///
/// In R:
/// ```r
/// z <- MyComplex$new(3, 4)
/// z$re()         # 3
/// z$im()         # 4
/// z$norm()       # 5 (magnitude)
/// z$arg()        # 0.927... (phase in radians)
/// z$conj()       # MyComplex(3, -4)
/// ```
pub trait RComplexOps {
    /// Get the real part.
    fn re(&self) -> f64;

    /// Get the imaginary part.
    fn im(&self) -> f64;

    /// Get the magnitude (absolute value): sqrt(re² + im²).
    fn norm(&self) -> f64;

    /// Get the squared magnitude: re² + im².
    fn norm_sqr(&self) -> f64;

    /// Get the phase angle in radians.
    fn arg(&self) -> f64;

    /// Check if this is a finite complex number.
    fn is_finite(&self) -> bool;

    /// Check if this is an infinite complex number.
    fn is_infinite(&self) -> bool;

    /// Check if this is NaN (either part).
    fn is_nan(&self) -> bool;

    /// Check if this is normal (not zero, infinite, NaN, or subnormal).
    fn is_normal(&self) -> bool;

    /// Get the complex conjugate.
    fn conj(&self) -> Complex<f64>;

    /// Get the reciprocal (1/z).
    fn inv(&self) -> Complex<f64>;
}

impl RComplexOps for Complex<f64> {
    fn re(&self) -> f64 {
        self.re
    }

    fn im(&self) -> f64 {
        self.im
    }

    fn norm(&self) -> f64 {
        Complex::norm(*self)
    }

    fn norm_sqr(&self) -> f64 {
        Complex::norm_sqr(self)
    }

    fn arg(&self) -> f64 {
        Complex::arg(*self)
    }

    fn is_finite(&self) -> bool {
        Complex::is_finite(*self)
    }

    fn is_infinite(&self) -> bool {
        Complex::is_infinite(*self)
    }

    fn is_nan(&self) -> bool {
        Complex::is_nan(*self)
    }

    fn is_normal(&self) -> bool {
        Complex::is_normal(*self)
    }

    fn conj(&self) -> Complex<f64> {
        Complex::conj(self)
    }

    fn inv(&self) -> Complex<f64> {
        Complex::inv(self)
    }
}
// endregion

// region: Unit tests

#[cfg(test)]
mod tests;
// endregion
