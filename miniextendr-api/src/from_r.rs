//! Conversions from R SEXP to Rust types.
//!
//! This module provides [`TryFromSexp`] implementations for converting R values to Rust types:
//!
//! | R Type | Rust Type | Access Method |
//! |--------|-----------|---------------|
//! | INTSXP | `i32`, `&[i32]` | `INTEGER()` / `DATAPTR_RO` |
//! | REALSXP | `f64`, `&[f64]` | `REAL()` / `DATAPTR_RO` |
//! | LGLSXP | `RLogical`, `&[RLogical]` | `LOGICAL()` / `DATAPTR_RO` |
//! | RAWSXP | `u8`, `&[u8]` | `RAW()` / `DATAPTR_RO` |
//! | STRSXP | `&str`, `String` | `STRING_ELT()` + `R_CHAR()` / `Rf_translateCharUTF8()` |
//!
//! # Thread Safety
//!
//! The trait provides two methods:
//! - [`TryFromSexp::try_from_sexp`] - checked version with debug thread assertions
//! - [`TryFromSexp::try_from_sexp_unchecked`] - unchecked version for performance-critical paths
//!
//! Use `try_from_sexp_unchecked` when you're certain you're on the main thread:
//! - Inside ALTREP callbacks
//! - Inside `#[miniextendr(unsafe(main_thread))]` functions
//! - Inside `extern "C-unwind"` functions called directly by R

use crate::ffi::{RLogical, RNativeType, Rboolean, SEXP, SEXPTYPE, SexpExt};

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
pub struct SexpNaError {
    pub sexp_type: SEXPTYPE,
}

#[derive(Debug, Clone, Copy)]
pub enum SexpError {
    Type(SexpTypeError),
    Length(SexpLengthError),
    Na(SexpNaError),
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

impl From<SexpNaError> for SexpError {
    fn from(e: SexpNaError) -> Self {
        SexpError::Na(e)
    }
}

/// TryFrom-style trait for converting SEXP to Rust types.
pub trait TryFromSexp: Sized {
    /// The error type returned when conversion fails.
    type Error;

    /// Attempt to convert an R SEXP to this Rust type.
    ///
    /// In debug builds, may assert that we're on R's main thread.
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error>;

    /// Convert from SEXP without thread safety checks.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread. In debug builds, this still
    /// calls the checked version by default, but implementations may
    /// skip thread assertions for performance.
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        // Default: just call the checked version
        Self::try_from_sexp(sexp)
    }
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

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            }
            .into());
        }
        let len = unsafe { sexp.len_unchecked() };
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }
        unsafe { sexp.as_slice_unchecked::<T>() }
            .first()
            .cloned()
            .ok_or_else(|| {
                SexpLengthError {
                    expected: 1,
                    actual: 0,
                }
                .into()
            })
    }
}

// =============================================================================
// Logical conversions
// =============================================================================

impl TryFromSexp for Rboolean {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let raw: RLogical = TryFromSexp::try_from_sexp(sexp)?;
        match raw.to_option_bool() {
            Some(false) => Ok(Rboolean::FALSE),
            Some(true) => Ok(Rboolean::TRUE),
            None => Err(SexpNaError {
                sexp_type: SEXPTYPE::LGLSXP,
            }
            .into()),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let raw: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        match raw.to_option_bool() {
            Some(false) => Ok(Rboolean::FALSE),
            Some(true) => Ok(Rboolean::TRUE),
            None => Err(SexpNaError {
                sexp_type: SEXPTYPE::LGLSXP,
            }
            .into()),
        }
    }
}

impl TryFromSexp for bool {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let raw: RLogical = TryFromSexp::try_from_sexp(sexp)?;
        raw.to_option_bool().ok_or_else(|| {
            SexpNaError {
                sexp_type: SEXPTYPE::LGLSXP,
            }
            .into()
        })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let raw: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        raw.to_option_bool().ok_or_else(|| {
            SexpNaError {
                sexp_type: SEXPTYPE::LGLSXP,
            }
            .into()
        })
    }
}

impl TryFromSexp for Option<bool> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let raw: RLogical = TryFromSexp::try_from_sexp(sexp)?;
        Ok(raw.to_option_bool())
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let raw: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(raw.to_option_bool())
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

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            });
        }
        Ok(unsafe { sexp.as_slice_unchecked::<T>() })
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

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::{R_CHAR_unchecked, STRING_ELT_unchecked};

        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = unsafe { sexp.len_unchecked() };
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }

        // Get the CHARSXP at index 0
        let charsxp = unsafe { STRING_ELT_unchecked(sexp, 0) };

        // Check for NA_STRING
        if charsxp == unsafe { crate::ffi::R_NaString } {
            return Ok("");
        }

        // Get the C string pointer
        let c_str = unsafe { R_CHAR_unchecked(charsxp) };
        if c_str.is_null() {
            return Ok("");
        }

        // Convert to Rust str
        let rust_str = unsafe { std::ffi::CStr::from_ptr(c_str) };
        rust_str.to_str().map_err(|_| {
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
        use crate::ffi::{Rf_translateCharUTF8, STRING_ELT};

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
            return Ok(String::new());
        }

        // Translate to UTF-8 in an R-managed buffer, then copy to an owned Rust String.
        let c_str = unsafe { Rf_translateCharUTF8(charsxp) };
        if c_str.is_null() {
            return Ok(String::new());
        }

        let rust_str = unsafe { std::ffi::CStr::from_ptr(c_str) };
        rust_str.to_str().map(|s| s.to_owned()).map_err(|_| {
            SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual: SEXPTYPE::STRSXP,
            }
            .into()
        })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::{Rf_translateCharUTF8_unchecked, STRING_ELT_unchecked};

        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = unsafe { sexp.len_unchecked() };
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }

        // Get the CHARSXP at index 0
        let charsxp = unsafe { STRING_ELT_unchecked(sexp, 0) };

        // Check for NA_STRING
        if charsxp == unsafe { crate::ffi::R_NaString } {
            return Ok(String::new());
        }

        // Translate to UTF-8 in an R-managed buffer, then copy to an owned Rust String.
        let c_str = unsafe { Rf_translateCharUTF8_unchecked(charsxp) };
        if c_str.is_null() {
            return Ok(String::new());
        }

        let rust_str = unsafe { std::ffi::CStr::from_ptr(c_str) };
        rust_str.to_str().map(|s| s.to_owned()).map_err(|_| {
            SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual: SEXPTYPE::STRSXP,
            }
            .into()
        })
    }
}
