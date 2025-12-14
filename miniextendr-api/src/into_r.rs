//! Conversions from Rust types to R SEXP.
//!
//! This module provides the [`IntoR`] trait for converting Rust values to R SEXPs.
//!
//! # Thread Safety
//!
//! The trait provides two methods:
//! - [`IntoR::into_sexp`] - checked version with debug thread assertions
//! - [`IntoR::into_sexp_unchecked`] - unchecked version for performance-critical paths
//!
//! Use `into_sexp_unchecked` when you're certain you're on the main thread:
//! - Inside ALTREP callbacks
//! - Inside `#[miniextendr(unsafe(main_thread))]` functions
//! - Inside `extern "C-unwind"` functions called directly by R

/// Trait for converting Rust types to R SEXP values.
pub trait IntoR {
    /// Convert this value to an R SEXP.
    ///
    /// In debug builds, asserts that we're on R's main thread.
    fn into_sexp(self) -> crate::ffi::SEXP;

    /// Convert to SEXP without thread safety checks.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread. In debug builds, this still
    /// calls the checked version by default, but implementations may
    /// skip thread assertions for performance.
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP
    where
        Self: Sized,
    {
        // Default: just call the checked version
        self.into_sexp()
    }
}

impl IntoR for crate::ffi::SEXP {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self
    }
}

impl IntoR for () {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::R_NilValue }
    }
}

impl IntoR for std::convert::Infallible {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::R_NilValue }
    }
}

impl IntoR for i32 {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarInteger(self) }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarInteger_unchecked(self) }
    }
}

impl IntoR for f64 {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarReal(self) }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarReal_unchecked(self) }
    }
}

impl IntoR for u8 {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarRaw(self) }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarRaw_unchecked(self) }
    }
}

impl IntoR for bool {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarLogical(self as i32) }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarLogical_unchecked(self as i32) }
    }
}

impl IntoR for Option<bool> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::Rf_ScalarLogical(i32::MIN) },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::Rf_ScalarLogical_unchecked(i32::MIN) },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarLogical_unchecked(self as i32) }
    }
}

impl IntoR for crate::ffi::RLogical {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarLogical(self.to_i32()) }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarLogical_unchecked(self.to_i32()) }
    }
}

impl IntoR for Option<bool> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::Rf_ScalarLogical(i32::MIN) },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::Rf_ScalarLogical_unchecked(i32::MIN) },
        }
    }
}

impl<T: crate::externalptr::TypedExternal> IntoR for crate::externalptr::ExternalPtr<T> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_sexp()
    }
}

/// Helper to convert a string slice to R CHARSXP.
/// Uses UTF-8 encoding. Empty strings return R_BlankString equivalent.
#[inline]
fn str_to_charsxp(s: &str) -> crate::ffi::SEXP {
    unsafe {
        if s.is_empty() {
            // For empty string, still use mkCharLenCE with length 0
            crate::ffi::Rf_mkCharLenCE(s.as_ptr().cast(), 0, crate::ffi::CE_UTF8)
        } else {
            crate::ffi::Rf_mkCharLenCE(s.as_ptr().cast(), s.len() as i32, crate::ffi::CE_UTF8)
        }
    }
}

/// Unchecked version of [`str_to_charsxp`].
#[inline]
unsafe fn str_to_charsxp_unchecked(s: &str) -> crate::ffi::SEXP {
    unsafe {
        if s.is_empty() {
            crate::ffi::Rf_mkCharLenCE_unchecked(s.as_ptr().cast(), 0, crate::ffi::CE_UTF8)
        } else {
            crate::ffi::Rf_mkCharLenCE_unchecked(
                s.as_ptr().cast(),
                s.len() as i32,
                crate::ffi::CE_UTF8,
            )
        }
    }
}

impl IntoR for String {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_str().into_sexp()
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { self.as_str().into_sexp_unchecked() }
    }
}

impl IntoR for &str {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let charsxp = str_to_charsxp(self);
            crate::ffi::Rf_ScalarString(charsxp)
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe {
            let charsxp = str_to_charsxp_unchecked(self);
            crate::ffi::Rf_ScalarString_unchecked(charsxp)
        }
    }
}
