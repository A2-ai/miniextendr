//! Thin wrapper around R character vector (`STRSXP`).
//!
//! Provides safe construction and element insertion for string vectors.

use std::borrow::Cow;

use crate::ffi::SEXPTYPE::STRSXP;
use crate::ffi::{self, SEXP, SexpExt};
use crate::from_r::{SexpError, SexpTypeError, TryFromSexp, charsxp_to_cow, charsxp_to_str};
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
        Some(self.0.string_elt(idx))
    }

    /// Get the string at the given index (zero-copy).
    ///
    /// Returns `None` if out of bounds or if the element is `NA_character_`.
    /// Panics if the CHARSXP is not valid UTF-8 (should not happen in a UTF-8 locale).
    #[inline]
    pub fn get_str(self, idx: isize) -> Option<&'static str> {
        let charsxp = self.get_charsxp(idx)?;
        unsafe {
            if charsxp == ffi::R_NaString {
                return None;
            }
            Some(charsxp_to_str(charsxp))
        }
    }

    /// Get the string at the given index as `Cow<str>` (encoding-safe).
    ///
    /// Returns `Cow::Borrowed` for UTF-8 strings (zero-copy), `Cow::Owned` for
    /// non-UTF-8 strings (translated via `Rf_translateCharUTF8`).
    /// Returns `None` if out of bounds or `NA_character_`.
    #[inline]
    pub fn get_cow(self, idx: isize) -> Option<Cow<'static, str>> {
        let charsxp = self.get_charsxp(idx)?;
        unsafe {
            if charsxp == ffi::R_NaString {
                return None;
            }
            Some(charsxp_to_cow(charsxp))
        }
    }

    /// Iterate over elements as `Option<&str>`.
    ///
    /// `NA_character_` elements yield `None`, valid strings yield `Some(&str)`.
    /// Zero-copy — each `&str` borrows directly from R's CHARSXP.
    #[inline]
    pub fn iter(self) -> StrVecIter {
        StrVecIter {
            vec: self,
            idx: 0,
            len: self.len(),
        }
    }

    /// Iterate over elements as `Option<Cow<str>>` (encoding-safe).
    ///
    /// Like [`iter`](Self::iter) but handles non-UTF-8 CHARSXPs gracefully.
    #[inline]
    pub fn iter_cow(self) -> StrVecCowIter {
        StrVecCowIter {
            vec: self,
            idx: 0,
            len: self.len(),
        }
    }

    // region: Safe element insertion

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
            self.0.set_string_elt(idx, charsxp);
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
        self.0.set_string_elt(idx, charsxp);
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
            let charsxp = SEXP::charsxp(s);
            // CHARSXP may be cached, but protect anyway for safety
            let _guard = OwnedProtect::new(charsxp);
            self.0.set_string_elt(idx, charsxp);
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
        unsafe {
            assert!(idx >= 0 && idx < self.len(), "index out of bounds");
            // R_NaString is a global constant, no protection needed
            self.0.set_string_elt(idx, ffi::R_NaString);
        }
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
    // endregion
}

// region: StrVec iterators

/// Iterator over `StrVec` elements as `Option<&str>`.
///
/// Yields `None` for `NA_character_`, `Some(&str)` for valid strings.
/// Zero-copy — each `&str` borrows directly from R's CHARSXP.
pub struct StrVecIter {
    vec: StrVec,
    idx: isize,
    len: isize,
}

impl Iterator for StrVecIter {
    type Item = Option<&'static str>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }
        let charsxp = self.vec.0.string_elt(self.idx);
        self.idx += 1;
        if charsxp == unsafe { ffi::R_NaString } {
            Some(None)
        } else {
            Some(Some(unsafe { charsxp_to_str(charsxp) }))
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.len - self.idx) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for StrVecIter {}

/// Iterator over `StrVec` elements as `Option<Cow<'static, str>>`.
///
/// Like [`StrVecIter`] but handles non-UTF-8 CHARSXPs via `Rf_translateCharUTF8`.
pub struct StrVecCowIter {
    vec: StrVec,
    idx: isize,
    len: isize,
}

impl Iterator for StrVecCowIter {
    type Item = Option<Cow<'static, str>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }
        let charsxp = self.vec.0.string_elt(self.idx);
        self.idx += 1;
        if charsxp == unsafe { ffi::R_NaString } {
            Some(None)
        } else {
            Some(Some(unsafe { charsxp_to_cow(charsxp) }))
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.len - self.idx) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for StrVecCowIter {}

impl IntoIterator for StrVec {
    type Item = Option<&'static str>;
    type IntoIter = StrVecIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// endregion

// region: StrVecBuilder - efficient batch string vector construction

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
    pub unsafe fn new(scope: &'a ProtectScope, len: usize) -> Self {
        // SAFETY: caller guarantees R main thread
        let vec = unsafe { scope.alloc_character(len).into_raw() };
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
        let charsxp = SEXP::charsxp(s);
        self.vec.set_string_elt(idx, charsxp);
    }

    /// Set an element to `NA_character_`.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn set_na(&self, idx: isize) {
        debug_assert!(idx >= 0 && idx < unsafe { ffi::Rf_xlength(self.vec) });
        self.vec.set_string_elt(idx, SEXP::na_string());
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
// endregion

// region: Trait implementations

impl IntoR for StrVec {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.0
    }
}

impl TryFromSexp for StrVec {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
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
// endregion

// region: ProtectedStrVec — GC-protected string vector with proper lifetimes

/// GC-protected view over an R character vector (`STRSXP`).
///
/// Unlike [`StrVec`] (which is `Copy` and trusts the caller for GC protection),
/// `ProtectedStrVec` owns an [`OwnedProtect`] guard that keeps the STRSXP alive.
/// All borrowed data (`&str`, iterators) has its lifetime tied to `&self`,
/// not `'static` — preventing use-after-GC bugs at compile time.
///
/// # When to use
///
/// - **`StrVec`**: for SEXP arguments to `.Call` (R protects them), or when you
///   manage protection yourself. Lightweight, `Copy`.
/// - **`ProtectedStrVec`**: when you allocate or receive an STRSXP and need to
///   keep it alive beyond the immediate scope. Not `Copy`.
///
/// # Example
///
/// ```ignore
/// #[miniextendr]
/// pub fn count_unique(strings: ProtectedStrVec) -> i32 {
///     let unique: HashSet<&str> = strings.iter()
///         .filter_map(|s| s)
///         .collect();
///     unique.len() as i32
/// }
/// ```
pub struct ProtectedStrVec {
    inner: StrVec,
    len: isize,
    _protect: Option<OwnedProtect>,
}

impl ProtectedStrVec {
    /// Create a protected view over an STRSXP.
    ///
    /// Calls `Rf_protect` on the SEXP. Use [`from_sexp_trusted`](Self::from_sexp_trusted)
    /// when the SEXP is already protected (e.g., `.Call` arguments) to avoid
    /// double-protecting.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid STRSXP.
    /// - Must be called from the R main thread.
    #[inline]
    pub unsafe fn new(sexp: SEXP) -> Self {
        let guard = unsafe { OwnedProtect::new(sexp) };
        let inner = unsafe { StrVec::from_raw(guard.get()) };
        let len = inner.len();
        Self {
            inner,
            len,
            _protect: Some(guard),
        }
    }

    /// Create a view without adding GC protection.
    ///
    /// Use this when the SEXP is already protected by R (e.g., a `.Call`
    /// argument, or in a `ProtectScope`). Avoids the redundant
    /// `Rf_protect`/`Rf_unprotect` pair.
    ///
    /// The lifetime-bound `&str` borrows are still enforced — this only
    /// skips the protect stack push, not the safety guarantees.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid STRSXP.
    /// - `sexp` must remain GC-protected for the lifetime of this struct.
    /// - Must be called from the R main thread.
    #[inline]
    pub unsafe fn from_sexp_trusted(sexp: SEXP) -> Self {
        let inner = unsafe { StrVec::from_raw(sexp) };
        let len = inner.len();
        Self {
            inner,
            len,
            _protect: None,
        }
    }

    /// Number of elements.
    #[inline]
    pub fn len(&self) -> isize {
        self.len
    }

    /// Whether the vector is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the string at index (zero-copy, lifetime tied to `&self`).
    ///
    /// Returns `None` for out-of-bounds or `NA_character_`.
    #[inline]
    pub fn get_str(&self, idx: isize) -> Option<&str> {
        // charsxp_to_str returns &'static str, but lifetime elision
        // restricts it to &'_ (tied to &self) — correct: data lives
        // as long as OwnedProtect keeps the STRSXP alive.
        self.inner.get_str(idx)
    }

    /// Get the string at index as `Cow<str>` (encoding-safe, lifetime tied to `&self`).
    #[inline]
    pub fn get_cow(&self, idx: isize) -> Option<Cow<'_, str>> {
        self.inner.get_cow(idx)
    }

    /// Iterate over elements as `Option<&str>` (lifetime tied to `&self`).
    #[inline]
    pub fn iter(&self) -> ProtectedStrVecIter<'_> {
        ProtectedStrVecIter {
            vec: self,
            idx: 0,
            len: self.len,
        }
    }

    /// Iterate over elements as `Option<Cow<str>>` (encoding-safe).
    #[inline]
    pub fn iter_cow(&self) -> ProtectedStrVecCowIter<'_> {
        ProtectedStrVecCowIter {
            vec: self,
            idx: 0,
            len: self.len,
        }
    }

    /// Get the underlying SEXP (still protected by this handle).
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.inner.as_sexp()
    }

    /// Get the inner `StrVec` (unprotected copy — caller assumes protection responsibility).
    #[inline]
    pub fn as_strvec(&self) -> StrVec {
        self.inner
    }
}

/// Iterator over `ProtectedStrVec` with lifetime tied to the protection guard.
pub struct ProtectedStrVecIter<'a> {
    vec: &'a ProtectedStrVec,
    idx: isize,
    len: isize,
}

impl<'a> Iterator for ProtectedStrVecIter<'a> {
    type Item = Option<&'a str>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }
        let result = self.vec.get_str(self.idx);
        self.idx += 1;
        // get_str returns None for NA; we need to distinguish "end of iter" from "NA element"
        // Wrap: Some(None) = NA, Some(Some(&str)) = value, None = end
        Some(result)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.len - self.idx) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for ProtectedStrVecIter<'_> {}

/// Encoding-safe iterator over `ProtectedStrVec`.
pub struct ProtectedStrVecCowIter<'a> {
    vec: &'a ProtectedStrVec,
    idx: isize,
    len: isize,
}

impl<'a> Iterator for ProtectedStrVecCowIter<'a> {
    type Item = Option<Cow<'a, str>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }
        let result = self.vec.get_cow(self.idx);
        self.idx += 1;
        Some(result)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.len - self.idx) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for ProtectedStrVecCowIter<'_> {}

impl<'a> IntoIterator for &'a ProtectedStrVec {
    type Item = Option<&'a str>;
    type IntoIter = ProtectedStrVecIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoR for ProtectedStrVec {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.as_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(self.as_sexp())
    }
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.as_sexp()
    }
}

impl TryFromSexp for ProtectedStrVec {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != STRSXP {
            return Err(SexpTypeError {
                expected: STRSXP,
                actual,
            }
            .into());
        }
        // Use from_sexp_trusted: TryFromSexp is called from generated .Call
        // wrappers where R already protects the argument. No need to double-protect.
        Ok(unsafe { ProtectedStrVec::from_sexp_trusted(sexp) })
    }
}

impl std::fmt::Debug for ProtectedStrVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProtectedStrVec")
            .field("len", &self.len)
            .finish()
    }
}
// endregion
