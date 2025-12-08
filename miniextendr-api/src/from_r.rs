//! Conversions from R SEXP to Rust types.
//!
//! This module provides `TryFromSexp` implementations for converting R values to Rust types:
//!
//! | R Type | Rust Type | Access Method |
//! |--------|-----------|---------------|
//! | INTSXP | `i32`, `&[i32]` | `INTEGER()` / `DATAPTR_RO` |
//! | REALSXP | `f64`, `&[f64]` | `REAL()` / `DATAPTR_RO` |
//! | LGLSXP | `Rboolean`, `&[Rboolean]` | `LOGICAL()` / `DATAPTR_RO` |
//! | RAWSXP | `u8`, `&[u8]` | `RAW()` / `DATAPTR_RO` |
//! | STRSXP | `&str`, `String` | `STRING_ELT()` + `R_CHAR()` |

use crate::ffi::{RNativeType, SEXP, SEXPTYPE, SexpExt};

#[derive(Debug, Clone, Copy)]
pub struct SexpTypeError {
    pub expected: SEXPTYPE,
    pub actual: SEXPTYPE,
}

#[derive(Debug, Clone, Copy)]
pub struct SexpLengthError {
    pub expected: usize,
    pub actual: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum SexpError {
    Type(SexpTypeError),
    Length(SexpLengthError),
}

impl From<SexpTypeError> for SexpError {
    fn from(e: SexpTypeError) -> Self {
        SexpError::Type(e)
    }
}

impl From<SexpLengthError> for SexpError {
    fn from(e: SexpLengthError) -> Self {
        SexpError::Length(e)
    }
}

/// TryFrom-style trait for converting SEXP to Rust types.
pub trait TryFromSexp: Sized {
    type Error;
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error>;
}

// Blanket implementation for scalar R native types
impl<T: RNativeType> TryFromSexp for T {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            }
            .into());
        }
        let len = sexp.len();
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }
        sexp.as_slice::<T>().first().cloned().ok_or_else(|| {
            SexpLengthError {
                expected: 1,
                actual: 0,
            }
            .into()
        })
    }
}

// Blanket implementation for slices of R native types
impl<T: RNativeType> TryFromSexp for &'static [T] {
    type Error = SexpTypeError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            });
        }
        Ok(sexp.as_slice::<T>())
    }
}

// =============================================================================
// String conversions - STRSXP requires special handling via STRING_ELT
// =============================================================================

/// Convert R character vector (STRSXP) to Rust &str.
///
/// Extracts the first element of the character vector and returns it as a UTF-8 string.
/// The returned string has static lifetime because it points to R's internal string pool.
///
/// # Safety
/// The returned &str is only valid as long as R doesn't garbage collect the CHARSXP.
/// In practice, this is safe within a single .Call invocation.
impl TryFromSexp for &'static str {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::{R_CHAR, STRING_ELT};

        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }

        // Get the CHARSXP at index 0
        let charsxp = unsafe { STRING_ELT(sexp, 0) };

        // Check for NA_STRING
        if charsxp == unsafe { crate::ffi::R_NaString } {
            // Return empty string for NA (or we could return an error)
            return Ok("");
        }

        // Get the C string pointer - R_CHAR returns UTF-8 for ASCII/UTF-8 strings
        let c_str = unsafe { R_CHAR(charsxp) };
        if c_str.is_null() {
            return Ok("");
        }

        // Convert to Rust str
        let rust_str = unsafe { std::ffi::CStr::from_ptr(c_str) };
        rust_str.to_str().map_err(|_| {
            // If not valid UTF-8, this is an error
            // In practice, R strings should be valid after R_CHAR
            SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual: SEXPTYPE::STRSXP,
            }
            .into()
        })
    }
}

/// Convert R character vector (STRSXP) to owned Rust String.
///
/// Extracts the first element and creates an owned copy.
impl TryFromSexp for String {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: &str = TryFromSexp::try_from_sexp(sexp)?;
        Ok(s.to_owned())
    }
}
