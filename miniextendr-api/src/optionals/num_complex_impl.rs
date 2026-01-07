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
//! This module treats a complex number as NA if **either** part equals `NA_REAL`.
//! Regular NaN values (not `NA_REAL`) are preserved as valid values.

pub use num_complex::Complex;

use crate::altrep_traits::NA_REAL;
use crate::ffi::{Rcomplex, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpLengthError, SexpNaError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;

// =============================================================================
// Helper functions
// =============================================================================

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

/// Check if an `Rcomplex` value is NA.
///
/// A complex is NA if either the real or imaginary part is `NA_REAL`.
/// We use bit comparison for reliable detection since `NA_REAL` is a specific NaN payload.
#[inline]
pub fn is_na_rcomplex(r: &Rcomplex) -> bool {
    let na_bits = NA_REAL.to_bits();
    r.r.to_bits() == na_bits || r.i.to_bits() == na_bits
}

/// Check if a `Complex<f64>` value is NA (either part is `NA_REAL`).
#[inline]
pub fn is_na_complex(c: &Complex<f64>) -> bool {
    let na_bits = NA_REAL.to_bits();
    c.re.to_bits() == na_bits || c.im.to_bits() == na_bits
}

// =============================================================================
// Scalar conversions
// =============================================================================

impl TryFromSexp for Complex<f64> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::COMPLEX_ELT;

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

        let rcomplex = unsafe { COMPLEX_ELT(sexp, 0) };
        if is_na_rcomplex(&rcomplex) {
            return Err(SexpError::Na(SexpNaError {
                sexp_type: SEXPTYPE::CPLXSXP,
            }));
        }

        Ok(from_rcomplex(rcomplex))
    }
}

impl IntoR for Complex<f64> {
    fn into_sexp(self) -> SEXP {
        use crate::ffi::Rf_ScalarComplex;
        unsafe { Rf_ScalarComplex(to_rcomplex(self)) }
    }
}

// =============================================================================
// Option conversions (NA support)
// =============================================================================

impl TryFromSexp for Option<Complex<f64>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::COMPLEX_ELT;

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

        let rcomplex = unsafe { COMPLEX_ELT(sexp, 0) };
        if is_na_rcomplex(&rcomplex) {
            Ok(None)
        } else {
            Ok(Some(from_rcomplex(rcomplex)))
        }
    }
}

impl IntoR for Option<Complex<f64>> {
    fn into_sexp(self) -> SEXP {
        use crate::ffi::Rf_ScalarComplex;
        match self {
            Some(c) => unsafe { Rf_ScalarComplex(to_rcomplex(c)) },
            None => unsafe { Rf_ScalarComplex(na_rcomplex()) },
        }
    }
}

// =============================================================================
// Vector conversions
// =============================================================================

impl TryFromSexp for Vec<Complex<f64>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::COMPLEX_ELT;

        let actual = sexp.type_of();
        if actual != SEXPTYPE::CPLXSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::CPLXSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let mut result = Vec::with_capacity(len);

        for i in 0..len {
            let rcomplex = unsafe { COMPLEX_ELT(sexp, i as crate::ffi::R_xlen_t) };
            if is_na_rcomplex(&rcomplex) {
                return Err(SexpError::InvalidValue(format!(
                    "NA at index {} not allowed for Vec<Complex<f64>>",
                    i
                )));
            }
            result.push(from_rcomplex(rcomplex));
        }

        Ok(result)
    }
}

impl IntoR for Vec<Complex<f64>> {
    fn into_sexp(self) -> SEXP {
        use crate::ffi::{Rf_allocVector, SET_COMPLEX_ELT};

        let len = self.len();
        let sexp = unsafe { Rf_allocVector(SEXPTYPE::CPLXSXP, len as crate::ffi::R_xlen_t) };

        for (i, c) in self.into_iter().enumerate() {
            unsafe { SET_COMPLEX_ELT(sexp, i as crate::ffi::R_xlen_t, to_rcomplex(c)) };
        }

        sexp
    }
}

// =============================================================================
// Vec<Option<Complex>> conversions (NA-aware vectors)
// =============================================================================

impl TryFromSexp for Vec<Option<Complex<f64>>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::COMPLEX_ELT;

        let actual = sexp.type_of();
        if actual != SEXPTYPE::CPLXSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::CPLXSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let mut result = Vec::with_capacity(len);

        for i in 0..len {
            let rcomplex = unsafe { COMPLEX_ELT(sexp, i as crate::ffi::R_xlen_t) };
            if is_na_rcomplex(&rcomplex) {
                result.push(None);
            } else {
                result.push(Some(from_rcomplex(rcomplex)));
            }
        }

        Ok(result)
    }
}

impl IntoR for Vec<Option<Complex<f64>>> {
    fn into_sexp(self) -> SEXP {
        use crate::ffi::{Rf_allocVector, SET_COMPLEX_ELT};

        let len = self.len();
        let sexp = unsafe { Rf_allocVector(SEXPTYPE::CPLXSXP, len as crate::ffi::R_xlen_t) };

        for (i, opt) in self.into_iter().enumerate() {
            let rcomplex = match opt {
                Some(c) => to_rcomplex(c),
                None => na_rcomplex(),
            };
            unsafe { SET_COMPLEX_ELT(sexp, i as crate::ffi::R_xlen_t, rcomplex) };
        }

        sexp
    }
}

// =============================================================================
// RComplexOps adapter trait
// =============================================================================

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
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RComplexOps for MyComplex;
/// }
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

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_from_rcomplex() {
        let c = Complex::new(3.0, 4.0);
        let r = to_rcomplex(c);
        assert_eq!(r.r, 3.0);
        assert_eq!(r.i, 4.0);

        let c2 = from_rcomplex(r);
        assert_eq!(c2.re, 3.0);
        assert_eq!(c2.im, 4.0);
    }

    #[test]
    fn test_na_rcomplex() {
        let na = na_rcomplex();
        assert!(is_na_rcomplex(&na));

        let normal = Rcomplex { r: 1.0, i: 2.0 };
        assert!(!is_na_rcomplex(&normal));

        // NA in real part only
        let partial_na = Rcomplex { r: NA_REAL, i: 2.0 };
        assert!(is_na_rcomplex(&partial_na));

        // NA in imaginary part only
        let partial_na2 = Rcomplex { r: 1.0, i: NA_REAL };
        assert!(is_na_rcomplex(&partial_na2));
    }

    #[test]
    fn test_is_na_complex() {
        let na = Complex::new(NA_REAL, NA_REAL);
        assert!(is_na_complex(&na));

        let normal = Complex::new(1.0, 2.0);
        assert!(!is_na_complex(&normal));

        // Regular NaN is NOT NA
        let nan = Complex::new(f64::NAN, 0.0);
        // NaN bits are different from NA_REAL bits (usually)
        // This test verifies that regular NaN is not treated as NA
        assert!(!is_na_complex(&nan) || nan.re.to_bits() == NA_REAL.to_bits());
    }

    #[test]
    fn test_rcomplexops_basics() {
        let c = Complex::new(3.0, 4.0);

        assert_eq!(RComplexOps::re(&c), 3.0);
        assert_eq!(RComplexOps::im(&c), 4.0);
        assert_eq!(RComplexOps::norm(&c), 5.0);
        assert_eq!(RComplexOps::norm_sqr(&c), 25.0);
        assert!(RComplexOps::is_finite(&c));
        assert!(!RComplexOps::is_infinite(&c));
        assert!(!RComplexOps::is_nan(&c));
    }

    #[test]
    fn test_rcomplexops_conj_inv() {
        let c = Complex::new(3.0, 4.0);

        let conj = RComplexOps::conj(&c);
        assert_eq!(conj.re, 3.0);
        assert_eq!(conj.im, -4.0);

        let inv = RComplexOps::inv(&c);
        // 1/(3+4i) = (3-4i)/25 = 0.12 - 0.16i
        assert!((inv.re - 0.12).abs() < 1e-10);
        assert!((inv.im + 0.16).abs() < 1e-10);
    }

    #[test]
    fn test_rcomplexops_special_values() {
        let inf = Complex::new(f64::INFINITY, 0.0);
        assert!(RComplexOps::is_infinite(&inf));
        assert!(!RComplexOps::is_finite(&inf));

        let nan = Complex::new(f64::NAN, 0.0);
        assert!(RComplexOps::is_nan(&nan));
        assert!(!RComplexOps::is_finite(&nan));
    }
}
