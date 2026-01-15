//! Thin wrapper around R character vector (`STRSXP`).
//!
//! Provides safe construction and element insertion for string vectors.

use crate::ffi::SEXPTYPE::STRSXP;
use crate::ffi::{self, SEXP};
use crate::from_r::{SexpError, SexpTypeError, TryFromSexp};
use crate::gc_protect::{OwnedProtect, ProtectScope};
use crate::into_r::IntoR;

/// Owned handle to an R character vector (`STRSXP`).
///
/// This wrapper provides safe methods for building character vectors
/// element-by-element with proper GC protection.
#[derive(Clone, Copy, Debug)]
pub struct StrVec(SEXP);

impl StrVec {
    /// Wrap an existing `STRSXP` without additional checks.
    ///
    /// # Safety
    ///
    /// Caller must ensure `sexp` is a valid character vector (`STRSXP`)
    /// whose lifetime remains managed by R.
    #[inline]
    pub const unsafe fn from_raw(sexp: SEXP) -> Self {
        StrVec(sexp)
    }

    /// Get the underlying `SEXP`.
    #[inline]
    pub const fn as_sexp(self) -> SEXP {
        self.0
    }

    /// Length of the character vector (number of elements).
    #[inline]
    pub fn len(self) -> isize {
        unsafe { ffi::Rf_xlength(self.0) }
    }

    /// Returns true if the vector is empty.
    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Get the CHARSXP at the given index.
    ///
    /// Returns `None` if out of bounds.
    #[inline]
    pub fn get_charsxp(self, idx: isize) -> Option<SEXP> {
        if idx < 0 || idx >= self.len() {
            return None;
        }
        Some(unsafe { ffi::STRING_ELT(self.0, idx) })
    }

    /// Get the string at the given index.
    ///
    /// Returns `None` if out of bounds or if the element is `NA_character_`.
    #[inline]
    pub fn get_str(self, idx: isize) -> Option<&'static str> {
        let charsxp = self.get_charsxp(idx)?;
        unsafe {
            if charsxp == ffi::R_NaString {
                return None;
            }
            let ptr = ffi::R_CHAR(charsxp);
            let len = ffi::Rf_xlength(charsxp) as usize;
            let bytes = std::slice::from_raw_parts(ptr as *const u8, len);
            // R stores strings as UTF-8 (or native encoding), assume valid UTF-8
            std::str::from_utf8(bytes).ok()
        }
    }

    // =========================================================================
    // Safe element insertion
    // =========================================================================

    /// Set a CHARSXP at the given index, protecting it during insertion.
    ///
    /// This is the safe way to insert a freshly allocated CHARSXP into a string vector.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `charsxp` must be a valid CHARSXP (from `Rf_mkChar*` or `STRING_ELT`)
    /// - `self` must be a valid, protected STRSXP
    ///
    /// # Panics
    ///
    /// Panics if `idx` is out of bounds.
    #[inline]
    pub unsafe fn set_charsxp(self, idx: isize, charsxp: SEXP) {
        assert!(idx >= 0 && idx < self.len(), "index out of bounds");
        // SAFETY: caller guarantees R main thread and valid SEXPs
        unsafe {
            // Protect CHARSXP during SET_STRING_ELT.
            // Note: Rf_mkCharLenCE returns a CHARSXP that may be from the global
            // CHARSXP cache, but protection is still needed for newly allocated ones.
            let _guard = OwnedProtect::new(charsxp);
            ffi::SET_STRING_ELT(self.0, idx, charsxp);
        }
    }

    /// Set a CHARSXP without protecting it.
    ///
    /// # Safety
    ///
    /// In addition to the safety requirements of [`set_charsxp`](Self::set_charsxp):
    /// - The caller must ensure `charsxp` is already protected or from the
    ///   global CHARSXP cache.
    #[inline]
    pub unsafe fn set_charsxp_unchecked(self, idx: isize, charsxp: SEXP) {
        debug_assert!(idx >= 0 && idx < self.len(), "index out of bounds");
        // SAFETY: caller guarantees charsxp is protected/cached
        unsafe { ffi::SET_STRING_ELT(self.0, idx, charsxp) };
    }

    /// Set an element from a Rust string.
    ///
    /// Creates a CHARSXP from the string and inserts it safely.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `self` must be a valid, protected STRSXP
    ///
    /// # Panics
    ///
    /// Panics if `idx` is out of bounds.
    #[inline]
    pub unsafe fn set_str(self, idx: isize, s: &str) {
        assert!(idx >= 0 && idx < self.len(), "index out of bounds");
        // SAFETY: caller guarantees R main thread
        unsafe {
            let charsxp = ffi::Rf_mkCharLenCE(s.as_ptr().cast(), s.len() as i32, ffi::CE_UTF8);
            // CHARSXP may be cached, but protect anyway for safety
            let _guard = OwnedProtect::new(charsxp);
            ffi::SET_STRING_ELT(self.0, idx, charsxp);
        }
    }

    /// Set an element to `NA_character_`.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `self` must be a valid, protected STRSXP
    ///
    /// # Panics
    ///
    /// Panics if `idx` is out of bounds.
    #[inline]
    pub unsafe fn set_na(self, idx: isize) {
        assert!(idx >= 0 && idx < self.len(), "index out of bounds");
        // R_NaString is a global constant, no protection needed
        unsafe { ffi::SET_STRING_ELT(self.0, idx, ffi::R_NaString) };
    }

    /// Set an element from an optional string.
    ///
    /// `None` becomes `NA_character_`.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `self` must be a valid, protected STRSXP
    ///
    /// # Panics
    ///
    /// Panics if `idx` is out of bounds.
    #[inline]
    pub unsafe fn set_opt_str(self, idx: isize, s: Option<&str>) {
        match s {
            Some(s) => unsafe { self.set_str(idx, s) },
            None => unsafe { self.set_na(idx) },
        }
    }
}

// =============================================================================
// StrVecBuilder - efficient batch string vector construction
// =============================================================================

/// Builder for constructing string vectors with efficient protection management.
///
/// # Example
///
/// ```ignore
/// unsafe fn build_strvec(strings: &[&str]) -> SEXP {
///     let scope = ProtectScope::new();
///     let builder = StrVecBuilder::new(&scope, strings.len() as isize);
///
///     for (i, s) in strings.iter().enumerate() {
///         builder.set_str(i as isize, s);
///     }
///
///     builder.into_sexp()
/// }
/// ```
pub struct StrVecBuilder<'a> {
    vec: SEXP,
    _scope: &'a ProtectScope,
}

impl<'a> StrVecBuilder<'a> {
    /// Create a new string vector builder with the given length.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn new(scope: &'a ProtectScope, len: isize) -> Self {
        // SAFETY: caller guarantees R main thread
        let vec = unsafe { scope.protect_raw(ffi::Rf_allocVector(STRSXP, len)) };
        Self { vec, _scope: scope }
    }

    /// Set an element from a Rust string.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn set_str(&self, idx: isize, s: &str) {
        debug_assert!(idx >= 0 && idx < unsafe { ffi::Rf_xlength(self.vec) });
        // SAFETY: caller guarantees R main thread
        unsafe {
            let charsxp = ffi::Rf_mkCharLenCE(s.as_ptr().cast(), s.len() as i32, ffi::CE_UTF8);
            ffi::SET_STRING_ELT(self.vec, idx, charsxp);
        }
    }

    /// Set an element to `NA_character_`.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn set_na(&self, idx: isize) {
        debug_assert!(idx >= 0 && idx < unsafe { ffi::Rf_xlength(self.vec) });
        // SAFETY: R_NaString is a global constant
        unsafe { ffi::SET_STRING_ELT(self.vec, idx, ffi::R_NaString) };
    }

    /// Set an element from an optional string.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn set_opt_str(&self, idx: isize, s: Option<&str>) {
        match s {
            // SAFETY: caller guarantees R main thread
            Some(s) => unsafe { self.set_str(idx, s) },
            None => unsafe { self.set_na(idx) },
        }
    }

    /// Get the underlying SEXP.
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.vec
    }

    /// Convert to a `StrVec` wrapper.
    #[inline]
    pub fn into_strvec(self) -> StrVec {
        StrVec(self.vec)
    }

    /// Convert to the underlying SEXP.
    #[inline]
    pub fn into_sexp(self) -> SEXP {
        self.vec
    }

    /// Get the length.
    #[inline]
    pub fn len(&self) -> isize {
        unsafe { ffi::Rf_xlength(self.vec) }
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// =============================================================================
// Trait implementations
// =============================================================================

impl IntoR for StrVec {
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.0
    }
}

impl TryFromSexp for StrVec {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = unsafe { ffi::TYPEOF(sexp) };
        if actual != STRSXP {
            return Err(SexpTypeError {
                expected: STRSXP,
                actual,
            }
            .into());
        }
        Ok(StrVec(sexp))
    }
}
