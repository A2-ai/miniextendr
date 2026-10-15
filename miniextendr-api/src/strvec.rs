//! Thin wrapper around R character vector (`STRSXP`).
//!
//! Provides safe construction and element insertion for string vectors.

use std::borrow::Cow;

use crate::SEXPTYPE::STRSXP;
use crate::from_r::{SexpError, SexpTypeError, TryFromSexp, charsxp_to_cow, charsxp_to_str};
use crate::gc_protect::{ChildRoots, OwnedProtect, ProtectScope};
use crate::into_r::IntoR;
use crate::{Preserved, SEXP, SexpExt};

/// Borrowed view over an R character vector (`STRSXP`).
///
/// Strings returned by the view carry `'a`. Constructors must tie that lifetime
/// to a root that keeps the source reachable. [`ProtectedStrVec::as_strvec`]
/// does so through a borrow of its owner.
///
/// [`try_from_root`](Self::try_from_root) borrows a lifetime-carrying root.
/// Generated wrappers use this path for `StrVec<'_>` parameters. Raw
/// `TryFromSexp` is unavailable: a copyable SEXP cannot constrain the view's
/// lifetime. Use [`ProtectedStrVec`] when the result must own its root.
///
/// `Copy` and cheap (a `SEXP` and a retention reference); covariant in `'a`.
/// Uncached ALTREP elements read through a rooted view stay alive for `'a`.
#[derive(Clone, Copy)]
pub struct StrVec<'a>(SEXP, Option<&'a ChildRoots>);

impl std::fmt::Debug for StrVec<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StrVec").field(&self.0).finish()
    }
}

impl<'a> StrVec<'a> {
    /// Strings do not retain a pointer to a native numeric vector.
    #[doc(hidden)]
    pub const NATIVE_BORROW: Option<crate::from_r::NativeBorrow> = None;

    /// Wrap an existing `STRSXP` without additional checks.
    ///
    /// # Safety
    ///
    /// Caller must ensure `sexp` is a valid character vector (`STRSXP`)
    /// that stays GC-reachable for the inferred lifetime `'a`. An ALTREP class
    /// must also retain every CHARSXP read through this raw view for `'a`;
    /// otherwise use `try_from_root` or `ProtectedStrVec`.
    #[inline]
    pub const unsafe fn from_raw(sexp: SEXP) -> Self {
        StrVec(sexp, None)
    }

    /// Get the underlying `SEXP`.
    #[inline]
    pub const fn as_sexp(self) -> SEXP {
        self.0
    }

    /// Length of the character vector (number of elements).
    #[inline]
    pub fn len(self) -> isize {
        self.0.xlength()
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
        let charsxp = self.0.string_elt(idx);
        Some(match self.1 {
            // The root/owner behind this reference lives for 'a.
            Some(children) => unsafe { children.protect(self.0, charsxp) },
            None => charsxp, // from_raw's caller guarantees element retention
        })
    }

    /// Get the string at the given index (zero-copy), leashed to `'a`.
    ///
    /// Returns `None` if out of bounds or if the element is `NA_character_`.
    /// Panics if the CHARSXP is not valid UTF-8 (should not happen in a UTF-8 locale).
    #[inline]
    pub fn get_str(self, idx: isize) -> Option<&'a str> {
        let charsxp = self.get_charsxp(idx)?;
        unsafe {
            if charsxp == SEXP::na_string() {
                return None;
            }
            // charsxp_to_str fabricates &'static; covariance narrows it to &'a.
            Some(charsxp_to_str(charsxp))
        }
    }

    /// Get the string at the given index as `Cow<str>` (encoding-safe), leashed to `'a`.
    ///
    /// Returns `Cow::Borrowed` for UTF-8 strings (zero-copy), `Cow::Owned` for
    /// non-UTF-8 strings (translated via `Rf_translateCharUTF8`).
    /// Returns `None` if out of bounds or `NA_character_`.
    #[inline]
    pub fn get_cow(self, idx: isize) -> Option<Cow<'a, str>> {
        let charsxp = self.get_charsxp(idx)?;
        unsafe {
            if charsxp == SEXP::na_string() {
                return None;
            }
            Some(charsxp_to_cow(charsxp))
        }
    }

    /// Get the string at the given index as `&'static str` — the courageous escape.
    ///
    /// Unlike [`get_str`](Self::get_str), the returned reference is **not** leashed
    /// to `'a`, so it can be stored in an `ExternalPtr`, a global, or sent across
    /// threads. The string data lives in R's CHARSXP; the CHARSXP must remain
    /// reachable. For an uncached ALTREP element, retain this view's protection
    /// scope or `ProtectedStrVec` owner as well as the source vector.
    ///
    /// # Safety
    ///
    /// The caller must keep the returned CHARSXP reachable for as long as the
    /// reference is used. Rooting an ordinary source STRSXP suffices; an ALTREP
    /// source may need this view's scope/owner or a separate root for the child.
    /// Letting R collect that child is use-after-free.
    #[inline]
    pub unsafe fn get_str_static(self, idx: isize) -> Option<&'static str> {
        let charsxp = self.get_charsxp(idx)?;
        unsafe {
            if charsxp == SEXP::na_string() {
                return None;
            }
            Some(charsxp_to_str(charsxp))
        }
    }

    /// Get the string at the given index as `Cow<'static, str>` — the courageous escape.
    ///
    /// `Cow` analogue of [`get_str_static`](Self::get_str_static); same rooting
    /// obligation for the borrowed (`Cow::Borrowed`) case.
    ///
    /// # Safety
    ///
    /// See [`get_str_static`](Self::get_str_static).
    #[inline]
    pub unsafe fn get_cow_static(self, idx: isize) -> Option<Cow<'static, str>> {
        let charsxp = self.get_charsxp(idx)?;
        unsafe {
            if charsxp == SEXP::na_string() {
                return None;
            }
            Some(charsxp_to_cow(charsxp))
        }
    }

    /// Iterate over elements as `Option<&str>` (leashed to `'a`).
    ///
    /// `NA_character_` elements yield `None`, valid strings yield `Some(&str)`.
    /// Zero-copy — each `&str` borrows directly from R's CHARSXP.
    #[inline]
    pub fn iter(self) -> StrVecIter<'a> {
        StrVecIter {
            vec: self,
            idx: 0,
            len: self.len(),
        }
    }

    /// Iterate over elements as `Option<Cow<str>>` (encoding-safe, leashed to `'a`).
    ///
    /// Like [`iter`](Self::iter) but handles non-UTF-8 CHARSXPs gracefully.
    #[inline]
    pub fn iter_cow(self) -> StrVecCowIter<'a> {
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
        assert!(idx >= 0 && idx < self.len(), "index out of bounds");
        // R_NaString is a global constant, no protection needed
        self.0.set_string_elt(idx, SEXP::na_string());
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

/// Iterator over `StrVec` elements as `Option<&str>` (leashed to `'a`).
///
/// Yields `None` for `NA_character_`, `Some(&str)` for valid strings.
/// Zero-copy — each `&str` borrows directly from R's CHARSXP.
pub struct StrVecIter<'a> {
    vec: StrVec<'a>,
    idx: isize,
    len: isize,
}

impl<'a> Iterator for StrVecIter<'a> {
    type Item = Option<&'a str>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }
        let value = self.vec.get_str(self.idx);
        self.idx += 1;
        Some(value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.len - self.idx) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for StrVecIter<'_> {}

/// Iterator over `StrVec` elements as `Option<Cow<'a, str>>`.
///
/// Like [`StrVecIter`] but handles non-UTF-8 CHARSXPs via `Rf_translateCharUTF8`.
pub struct StrVecCowIter<'a> {
    vec: StrVec<'a>,
    idx: isize,
    len: isize,
}

impl<'a> Iterator for StrVecCowIter<'a> {
    type Item = Option<Cow<'a, str>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }
        let value = self.vec.get_cow(self.idx);
        self.idx += 1;
        Some(value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.len - self.idx) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for StrVecCowIter<'_> {}

impl<'a> IntoIterator for StrVec<'a> {
    type Item = Option<&'a str>;
    type IntoIter = StrVecIter<'a>;

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

    /// Create a new string vector builder via the **unchecked** FFI allocation path.
    ///
    /// `_unchecked` twin of [`new`](Self::new): the STRSXP is allocated with
    /// `Rf_allocVector_unchecked` (see [`ProtectScope::alloc_character_unchecked`]),
    /// bypassing the main-thread assertion. Use inside ALTREP callbacks,
    /// `with_r_unwind_protect`, or `with_r_thread` bodies, and pair element
    /// insertion with the `_unchecked` string-element setters.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread, in a context where the checked-FFI
    /// assertion is intentionally bypassed (see CLAUDE.md "FFI thread checking").
    #[inline]
    pub unsafe fn new_unchecked(scope: &'a ProtectScope, len: usize) -> Self {
        // SAFETY: caller guarantees R main thread in a checked-bypass context.
        let vec = unsafe { scope.alloc_character_unchecked(len).into_raw() };
        Self { vec, _scope: scope }
    }

    /// Set an element from a Rust string.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn set_str(&self, idx: isize, s: &str) {
        debug_assert!(idx >= 0 && idx < self.vec.xlength());
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
        debug_assert!(idx >= 0 && idx < self.vec.xlength());
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

    /// Convert to a `StrVec` view, leashed to the builder's protect scope `'a`.
    #[inline]
    pub fn into_strvec(self) -> StrVec<'a> {
        // The builder allocates an ordinary STRSXP, whose elements are traced
        // directly. Its scope already roots that vector for 'a.
        StrVec(self.vec, None)
    }

    /// Convert to the underlying SEXP.
    #[inline]
    pub fn into_sexp(self) -> SEXP {
        self.vec
    }

    /// Get the length.
    #[inline]
    pub fn len(&self) -> isize {
        self.vec.xlength()
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
// endregion

// region: Trait implementations

impl IntoR for StrVec<'_> {
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

impl<'a> StrVec<'a> {
    /// Borrow a character vector for the lifetime of its root.
    ///
    /// The result cannot acquire a lifetime longer than the supplied root.
    /// Raw `TryFromSexp` is intentionally unavailable for this borrowed view.
    ///
    /// ```compile_fail
    /// use miniextendr_api::{Root, StrVec};
    /// fn escape<'a>(root: Root<'a>) -> StrVec<'static> {
    ///     StrVec::try_from_root(root).unwrap()
    /// }
    /// ```
    pub fn try_from_root(root: crate::Root<'a>) -> Result<Self, SexpError> {
        let sexp = root.get();
        let actual = sexp.type_of();
        if actual != STRSXP {
            return Err(SexpTypeError {
                expected: STRSXP,
                actual,
            }
            .into());
        }
        // The root, not an unconstrained return annotation, fixes `'a`.
        Ok(StrVec(sexp, Some(root.child_roots())))
    }
}
// endregion

// region: ProtectedStrVec — GC-protected string vector with proper lifetimes

/// GC-protected view over an R character vector (`STRSXP`).
///
/// Unlike [`StrVec`] (which is `Copy` and trusts the caller for GC protection),
/// `ProtectedStrVec` holds a [`Preserved`] root on the STRSXP for its Rust
/// lifetime. It can outlive a `.Call` frame and be dropped in any order
/// relative to other handles.
/// Borrowed strings and iterators are tied to `&self`. It is `!Send + !Sync`:
/// reading and releasing the R object must happen on R's main thread.
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
    sexp: SEXP,
    /// `None` for [`from_sexp_trusted`](Self::from_sexp_trusted), whose caller
    /// roots `sexp`. The field also makes the handle `!Send + !Sync`.
    _root: Option<Preserved>,
    children: ChildRoots,
    len: isize,
}

impl ProtectedStrVec {
    /// Create a protected view over an STRSXP.
    ///
    /// Roots the SEXP with a [`Preserved`] handle, released on drop,
    /// independently of R's protection stack. Use
    /// [`from_sexp_trusted`](Self::from_sexp_trusted) only when another owner
    /// guarantees reachability for this handle's entire lifetime.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid STRSXP.
    /// - Must be called from the R main thread.
    #[inline]
    pub unsafe fn new(sexp: SEXP) -> Self {
        // Root before the length read, which can run an ALTREP callback.
        let root = Preserved::new(sexp);
        Self {
            sexp,
            len: sexp.xlength(),
            _root: Some(root),
            children: ChildRoots::default(),
        }
    }

    /// Create a view without adding GC protection.
    ///
    /// Use this when the SEXP is already protected by R (e.g., a `.Call`
    /// argument, or in a `ProtectScope`) for the handle's entire lifetime.
    /// Avoids an additional preserved root.
    ///
    /// The lifetime-bound `&str` borrows are still enforced — this only
    /// skips acquiring a root; the unsafe caller must supply it.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid STRSXP.
    /// - `sexp` must remain GC-protected for the lifetime of this struct.
    /// - Must be called from the R main thread.
    #[inline]
    pub unsafe fn from_sexp_trusted(sexp: SEXP) -> Self {
        Self {
            sexp,
            _root: None,
            children: ChildRoots::default(),
            len: sexp.xlength(),
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
        self.as_strvec().get_str(idx)
    }

    /// Get the string at index as `Cow<str>` (encoding-safe, lifetime tied to `&self`).
    #[inline]
    pub fn get_cow(&self, idx: isize) -> Option<Cow<'_, str>> {
        self.as_strvec().get_cow(idx)
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
        self.sexp
    }

    /// Get the inner `StrVec` view, leashed to `&self` (the protect guard).
    #[inline]
    pub fn as_strvec(&self) -> StrVec<'_> {
        // The owned root (or from_sexp_trusted's caller) keeps the parent
        // reachable; uncached children live in this owner's pool. Both outlive
        // the borrowed view, without materializing or changing the source.
        StrVec(self.sexp, Some(&self.children))
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
    const CHARACTER_ONLY: bool = true;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != STRSXP {
            return Err(SexpTypeError {
                expected: STRSXP,
                actual,
            }
            .into());
        }
        // A raw conversion has no frame lifetime. Its result must own a root
        // even when the input happens to be a .Call argument.
        Ok(unsafe { ProtectedStrVec::new(sexp) })
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
