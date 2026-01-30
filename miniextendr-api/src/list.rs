//! Thin wrapper around R list (`VECSXP`).
//! Provides safe construction from Rust values and typed extraction.

use crate::ffi::SEXPTYPE::{LISTSXP, STRSXP, VECSXP};
use crate::ffi::{self, Rboolean, SEXP};
use crate::from_r::{SexpError, SexpLengthError, SexpTypeError, TryFromSexp};
use crate::gc_protect::OwnedProtect;
use crate::into_r::IntoR;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

/// Owned handle to an R list (`VECSXP`).
#[derive(Clone, Copy, Debug)]
pub struct List(SEXP);

/// Mutable view of an R list (`VECSXP`).
///
/// This is a wrapper type instead of `&mut [SEXP]` to avoid exposing a raw slice
/// that could become invalid if list elements are replaced with `NULL`.
#[derive(Debug)]
pub struct ListMut(SEXP);

impl List {
    /// Return true if the underlying SEXP is a list (VECSXP) according to R.
    #[inline]
    pub fn is_list(self) -> bool {
        unsafe { ffi::Rf_isList(self.0) != Rboolean::FALSE }
    }

    /// Wrap an existing `VECSXP` without additional checks.
    ///
    /// # Safety
    ///
    /// Caller must ensure `sexp` is a valid list object (typically a `VECSXP` or
    /// a pairlist coerced to `VECSXP`) whose lifetime remains managed by R.
    #[inline]
    pub const unsafe fn from_raw(sexp: SEXP) -> Self {
        List(sexp)
    }

    /// Get the underlying `SEXP`.
    #[inline]
    pub const fn as_sexp(self) -> SEXP {
        self.0
    }

    /// Length of the list (number of elements).
    #[inline]
    pub fn len(self) -> isize {
        unsafe { ffi::Rf_xlength(self.0) }
    }

    /// Returns true if the list is empty.
    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Get raw SEXP element at 0-based index. Returns `None` if out of bounds.
    #[inline]
    pub fn get(self, idx: isize) -> Option<SEXP> {
        if idx < 0 || idx >= self.len() {
            return None;
        }
        Some(unsafe { ffi::VECTOR_ELT(self.0, idx) })
    }

    /// Get element at 0-based index and convert to type `T`.
    ///
    /// Returns `None` if index is out of bounds or conversion fails.
    #[inline]
    pub fn get_index<T>(self, idx: isize) -> Option<T>
    where
        T: TryFromSexp<Error = SexpError>,
    {
        let sexp = self.get(idx)?;
        T::try_from_sexp(sexp).ok()
    }

    /// Get element by name and convert to type `T`.
    ///
    /// Returns `None` if name not found or conversion fails.
    pub fn get_named<T>(self, name: &str) -> Option<T>
    where
        T: TryFromSexp<Error = SexpError>,
    {
        let names_sexp = self.names()?;
        let n = self.len();

        // Search for matching name
        for i in 0..n {
            let name_sexp = unsafe { ffi::STRING_ELT(names_sexp, i) };
            if name_sexp == unsafe { ffi::R_NaString } {
                continue;
            }
            let name_ptr = unsafe { ffi::R_CHAR(name_sexp) };
            let name_cstr = unsafe { std::ffi::CStr::from_ptr(name_ptr) };
            if let Ok(s) = name_cstr.to_str() {
                if s == name {
                    let elem = unsafe { ffi::VECTOR_ELT(self.0, i) };
                    return T::try_from_sexp(elem).ok();
                }
            }
        }
        None
    }

    // =========================================================================
    // Attribute getters (equivalent to R's GET_* macros)
    // =========================================================================

    /// Get an arbitrary attribute by symbol (unchecked internal helper).
    ///
    /// # Safety
    ///
    /// Caller must ensure `what` is a valid symbol SEXP.
    #[inline]
    unsafe fn get_attr_impl_unchecked(self, what: SEXP) -> Option<SEXP> {
        unsafe {
            let attr = ffi::Rf_getAttrib(self.0, what);
            if attr == ffi::R_NilValue {
                None
            } else {
                Some(attr)
            }
        }
    }

    /// Get the `names` attribute if present.
    ///
    /// Equivalent to R's `GET_NAMES(x)`.
    #[inline]
    pub fn names(self) -> Option<SEXP> {
        // Safety: R_NamesSymbol is a known symbol
        unsafe { self.get_attr_impl_unchecked(ffi::R_NamesSymbol) }
    }

    /// Get the `class` attribute if present.
    ///
    /// Equivalent to R's `GET_CLASS(x)`.
    #[inline]
    pub fn get_class(self) -> Option<SEXP> {
        // Safety: R_ClassSymbol is a known symbol
        unsafe { self.get_attr_impl_unchecked(ffi::R_ClassSymbol) }
    }

    /// Get the `dim` attribute if present.
    ///
    /// Equivalent to R's `GET_DIM(x)`.
    #[inline]
    pub fn get_dim(self) -> Option<SEXP> {
        // Safety: R_DimSymbol is a known symbol
        unsafe { self.get_attr_impl_unchecked(ffi::R_DimSymbol) }
    }

    /// Get the `dimnames` attribute if present.
    ///
    /// Equivalent to R's `GET_DIMNAMES(x)`.
    #[inline]
    pub fn get_dimnames(self) -> Option<SEXP> {
        // Safety: R_DimNamesSymbol is a known symbol
        unsafe { self.get_attr_impl_unchecked(ffi::R_DimNamesSymbol) }
    }

    /// Get row names from the `dimnames` attribute.
    ///
    /// Equivalent to R's `GET_ROWNAMES(x)` / `Rf_GetRowNames(x)`.
    #[inline]
    pub fn get_rownames(self) -> Option<SEXP> {
        unsafe {
            let rownames = ffi::Rf_GetRowNames(self.0);
            if rownames == ffi::R_NilValue {
                None
            } else {
                Some(rownames)
            }
        }
    }

    /// Get column names from the `dimnames` attribute.
    ///
    /// Equivalent to R's `GET_COLNAMES(x)` / `Rf_GetColNames(x)`.
    #[inline]
    pub fn get_colnames(self) -> Option<SEXP> {
        unsafe {
            // Rf_GetColNames takes the dimnames, not the object itself
            let dimnames = ffi::Rf_getAttrib(self.0, ffi::R_DimNamesSymbol);
            if dimnames == ffi::R_NilValue {
                return None;
            }
            let colnames = ffi::Rf_GetColNames(dimnames);
            if colnames == ffi::R_NilValue {
                None
            } else {
                Some(colnames)
            }
        }
    }

    /// Get the `levels` attribute if present (for factors).
    ///
    /// Equivalent to R's `GET_LEVELS(x)`.
    #[inline]
    pub fn get_levels(self) -> Option<SEXP> {
        // Safety: R_LevelsSymbol is a known symbol
        unsafe { self.get_attr_impl_unchecked(ffi::R_LevelsSymbol) }
    }

    /// Get the `tsp` attribute if present (for time series).
    ///
    /// Equivalent to R's `GET_TSP(x)`.
    #[inline]
    pub fn get_tsp(self) -> Option<SEXP> {
        // Safety: R_TspSymbol is a known symbol
        unsafe { self.get_attr_impl_unchecked(ffi::R_TspSymbol) }
    }

    // =========================================================================
    // Attribute setters (equivalent to R's SET_* macros)
    // =========================================================================

    /// Set an arbitrary attribute by symbol (unchecked internal helper).
    ///
    /// # Safety
    ///
    /// Caller must ensure `what` is a valid symbol SEXP.
    #[inline]
    #[allow(dead_code)]
    unsafe fn set_attr_impl_unchecked(self, what: SEXP, value: SEXP) -> Self {
        unsafe { ffi::Rf_setAttrib(self.0, what, value) };
        self
    }

    /// Set the `names` attribute; returns the same list for chaining.
    ///
    /// Equivalent to R's `SET_NAMES(x, n)`.
    #[inline]
    pub fn set_names(self, names: SEXP) -> Self {
        unsafe { ffi::Rf_namesgets(self.0, names) };
        self
    }

    /// Set the `class` attribute; returns the same list for chaining.
    ///
    /// Equivalent to R's `SET_CLASS(x, n)`.
    #[inline]
    pub fn set_class(self, class: SEXP) -> Self {
        unsafe { ffi::Rf_setAttrib(self.0, ffi::R_ClassSymbol, class) };
        self
    }

    /// Set the `dim` attribute; returns the same list for chaining.
    ///
    /// Equivalent to R's `SET_DIM(x, n)`.
    #[inline]
    pub fn set_dim(self, dim: SEXP) -> Self {
        unsafe { ffi::Rf_dimgets(self.0, dim) };
        self
    }

    /// Set the `dimnames` attribute; returns the same list for chaining.
    ///
    /// Equivalent to R's `SET_DIMNAMES(x, n)`.
    #[inline]
    pub fn set_dimnames(self, dimnames: SEXP) -> Self {
        unsafe { ffi::Rf_setAttrib(self.0, ffi::R_DimNamesSymbol, dimnames) };
        self
    }

    /// Set the `levels` attribute; returns the same list for chaining.
    ///
    /// Equivalent to R's `SET_LEVELS(x, l)`.
    #[inline]
    pub fn set_levels(self, levels: SEXP) -> Self {
        unsafe { ffi::Rf_setAttrib(self.0, ffi::R_LevelsSymbol, levels) };
        self
    }

    // =========================================================================
    // Convenience setters (string-based)
    // =========================================================================

    /// Set the `class` attribute from a slice of class names.
    ///
    /// This is a convenience wrapper that creates a character vector from the
    /// provided strings and sets it as the class attribute.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let list = List::from_pairs(vec![("x", vec![1, 2, 3])]);
    /// let df = list.set_class_str(&["data.frame"]);
    /// ```
    #[inline]
    pub fn set_class_str(self, classes: &[&str]) -> Self {
        use crate::ffi::SEXPTYPE::STRSXP;

        let n = classes.len() as isize;
        unsafe {
            let class_vec = OwnedProtect::new(ffi::Rf_allocVector(STRSXP, n));
            for (i, class) in classes.iter().enumerate() {
                let chars =
                    ffi::Rf_mkCharLenCE(class.as_ptr().cast(), class.len() as i32, ffi::CE_UTF8);
                ffi::SET_STRING_ELT(class_vec.get(), i as isize, chars);
            }
            ffi::Rf_setAttrib(self.0, ffi::R_ClassSymbol, class_vec.get());
        }
        self
    }

    /// Set the `names` attribute from a slice of strings.
    ///
    /// This is a convenience wrapper that creates a character vector from the
    /// provided strings and sets it as the names attribute.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let list = List::from_values(vec![1, 2, 3]);
    /// let named = list.set_names_str(&["a", "b", "c"]);
    /// ```
    #[inline]
    pub fn set_names_str(self, names: &[&str]) -> Self {
        use crate::ffi::SEXPTYPE::STRSXP;

        let n = names.len() as isize;
        unsafe {
            let names_vec = OwnedProtect::new(ffi::Rf_allocVector(STRSXP, n));
            for (i, name) in names.iter().enumerate() {
                let chars =
                    ffi::Rf_mkCharLenCE(name.as_ptr().cast(), name.len() as i32, ffi::CE_UTF8);
                ffi::SET_STRING_ELT(names_vec.get(), i as isize, chars);
            }
            ffi::Rf_namesgets(self.0, names_vec.get());
        }
        self
    }

    /// Set `row.names` for a data.frame using compact integer form.
    ///
    /// R internally represents row.names as a compact integer vector
    /// `c(NA_integer_, -n)` when the row names are just `1:n`. This is more
    /// memory-efficient than storing n strings.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let list = List::from_pairs(vec![
    ///     ("x", vec![1, 2, 3]),
    ///     ("y", vec![4, 5, 6]),
    /// ])
    /// .set_class_str(&["data.frame"])
    /// .set_row_names_int(3);  // Row names: "1", "2", "3"
    /// ```
    #[inline]
    pub fn set_row_names_int(self, n: usize) -> Self {
        use crate::ffi::SEXPTYPE::INTSXP;

        unsafe {
            // R's compact row.names: c(NA_integer_, -n)
            let row_names = OwnedProtect::new(ffi::Rf_allocVector(INTSXP, 2));
            let ptr = ffi::INTEGER(row_names.get());
            // NA_INTEGER is i32::MIN in R
            *ptr = i32::MIN;
            *ptr.add(1) = -(n as i32);
            ffi::Rf_setAttrib(self.0, ffi::R_RowNamesSymbol, row_names.get());
        }
        self
    }

    /// Set `row.names` from a vector of strings.
    ///
    /// Use this when you need custom row names. For simple sequential row names
    /// (1, 2, 3, ...), use [`set_row_names_int`](Self::set_row_names_int) instead.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let list = List::from_pairs(vec![
    ///     ("x", vec![1, 2, 3]),
    /// ])
    /// .set_class_str(&["data.frame"])
    /// .set_row_names_str(&["row_a", "row_b", "row_c"]);
    /// ```
    #[inline]
    pub fn set_row_names_str(self, row_names: &[&str]) -> Self {
        use crate::ffi::SEXPTYPE::STRSXP;

        let n = row_names.len() as isize;
        unsafe {
            let names_vec = OwnedProtect::new(ffi::Rf_allocVector(STRSXP, n));
            for (i, name) in row_names.iter().enumerate() {
                let chars =
                    ffi::Rf_mkCharLenCE(name.as_ptr().cast(), name.len() as i32, ffi::CE_UTF8);
                ffi::SET_STRING_ELT(names_vec.get(), i as isize, chars);
            }
            ffi::Rf_setAttrib(self.0, ffi::R_RowNamesSymbol, names_vec.get());
        }
        self
    }

    // =========================================================================
    // Safe element insertion
    // =========================================================================

    /// Set an element at the given index, protecting the child during insertion.
    ///
    /// This is the safe way to insert a freshly allocated SEXP into a list.
    /// The child is protected for the duration of the `SET_VECTOR_ELT` call,
    /// ensuring it cannot be garbage collected.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `child` must be a valid SEXP
    /// - `self` must be a valid, protected VECSXP
    ///
    /// # Panics
    ///
    /// Panics if `idx` is out of bounds.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let scope = ProtectScope::new();
    /// let list = List::from_raw(scope.protect_raw(Rf_allocVector(VECSXP, n)));
    ///
    /// for i in 0..n {
    ///     let child = Rf_allocVector(REALSXP, 10);  // unprotected!
    ///     list.set_elt(i, child);  // safe: protects child during insertion
    /// }
    /// ```
    #[inline]
    pub unsafe fn set_elt(self, idx: isize, child: SEXP) {
        assert!(idx >= 0 && idx < self.len(), "index out of bounds");
        // Protect child for the duration of SET_VECTOR_ELT.
        // Once inserted, the child is protected by the parent container.
        // SAFETY: caller guarantees R main thread and valid SEXPs
        unsafe {
            let _guard = OwnedProtect::new(child);
            ffi::SET_VECTOR_ELT(self.0, idx, child);
        }
    }

    /// Set an element without protecting the child.
    ///
    /// # Safety
    ///
    /// In addition to the safety requirements of [`set_elt`](Self::set_elt):
    /// - The caller must ensure `child` is already protected or that no GC
    ///   can occur between child allocation and this call.
    ///
    /// Use this for performance when you know the child is already protected
    /// (e.g., it's a child of another protected container, or you have an
    /// `OwnedProtect` guard for it).
    #[inline]
    pub unsafe fn set_elt_unchecked(self, idx: isize, child: SEXP) {
        debug_assert!(idx >= 0 && idx < self.len(), "index out of bounds");
        // SAFETY: caller guarantees child is protected and valid
        unsafe { ffi::SET_VECTOR_ELT(self.0, idx, child) };
    }

    /// Set an element using a callback that produces the child.
    ///
    /// The callback is executed within a protection scope, so any allocations
    /// it performs are protected until insertion completes.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `self` must be a valid, protected VECSXP
    ///
    /// # Example
    ///
    /// ```ignore
    /// let list = List::from_raw(scope.protect_raw(Rf_allocVector(VECSXP, n)));
    ///
    /// for i in 0..n {
    ///     list.set_elt_with(i, || {
    ///         let vec = Rf_allocVector(REALSXP, 10);
    ///         fill_vector(vec);  // can allocate internally
    ///         vec
    ///     });
    /// }
    /// ```
    #[inline]
    pub unsafe fn set_elt_with<F>(self, idx: isize, f: F)
    where
        F: FnOnce() -> SEXP,
    {
        assert!(idx >= 0 && idx < self.len(), "index out of bounds");
        // SAFETY: caller guarantees R main thread
        unsafe {
            let child = OwnedProtect::new(f());
            ffi::SET_VECTOR_ELT(self.0, idx, child.get());
        }
    }
}

// =============================================================================
// ListBuilder - efficient batch list construction
// =============================================================================

use crate::gc_protect::ProtectScope;

/// Builder for constructing lists with efficient protection management.
///
/// `ListBuilder` holds a reference to a [`ProtectScope`], allowing multiple
/// elements to be inserted without repeatedly protecting/unprotecting each one.
/// This is more efficient than using [`List::set_elt`] in a loop.
///
/// # Example
///
/// ```ignore
/// unsafe fn build_list(n: isize) -> SEXP {
///     let scope = ProtectScope::new();
///     let builder = ListBuilder::new(&scope, n);
///
///     for i in 0..n {
///         // Allocations inside the loop are protected by the scope
///         let child = scope.protect_raw(Rf_allocVector(REALSXP, 10));
///         builder.set(i, child);
///     }
///
///     builder.into_sexp()
/// }
/// ```
pub struct ListBuilder<'a> {
    list: SEXP,
    _scope: &'a ProtectScope,
}

impl<'a> ListBuilder<'a> {
    /// Create a new list builder with the given length.
    ///
    /// The list is allocated and protected using the provided scope.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn new(scope: &'a ProtectScope, len: isize) -> Self {
        // SAFETY: caller guarantees R main thread
        let list = unsafe { scope.protect_raw(ffi::Rf_allocVector(VECSXP, len)) };
        Self {
            list,
            _scope: scope,
        }
    }

    /// Create a builder wrapping an existing protected list.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `list` must be a valid, protected VECSXP
    #[inline]
    pub unsafe fn from_protected(scope: &'a ProtectScope, list: SEXP) -> Self {
        Self {
            list,
            _scope: scope,
        }
    }

    /// Set an element at the given index.
    ///
    /// The `child` should be protected by the same scope (or a parent scope).
    /// Use `scope.protect_raw(...)` before calling this method.
    ///
    /// # Safety
    ///
    /// - `child` must be a valid SEXP
    /// - `child` should be protected (typically via the same scope)
    #[inline]
    pub unsafe fn set(&self, idx: isize, child: SEXP) {
        // SAFETY: caller guarantees valid and protected child
        unsafe {
            debug_assert!(idx >= 0 && idx < ffi::Rf_xlength(self.list));
            ffi::SET_VECTOR_ELT(self.list, idx, child);
        }
    }

    /// Set an element, protecting the child within the builder's scope.
    ///
    /// This is a convenience method that protects the child and then inserts it.
    ///
    /// # Safety
    ///
    /// - `child` must be a valid SEXP
    #[inline]
    pub unsafe fn set_protected(&self, idx: isize, child: SEXP) {
        // SAFETY: caller guarantees valid child
        unsafe {
            debug_assert!(idx >= 0 && idx < ffi::Rf_xlength(self.list));
            let _guard = OwnedProtect::new(child);
            ffi::SET_VECTOR_ELT(self.list, idx, child);
        }
    }

    /// Get the underlying list SEXP.
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.list
    }

    /// Convert to a `List` wrapper.
    #[inline]
    pub fn into_list(self) -> List {
        List(self.list)
    }

    /// Convert to the underlying SEXP.
    #[inline]
    pub fn into_sexp(self) -> SEXP {
        self.list
    }

    /// Get the length of the list.
    #[inline]
    pub fn len(&self) -> isize {
        unsafe { ffi::Rf_xlength(self.list) }
    }

    /// Check if the list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// =============================================================================
// ListAccumulator - unknown-length list construction with bounded stack usage
// =============================================================================

use crate::gc_protect::{ReprotectSlot, Root};

/// Accumulator for building lists when the length is unknown upfront.
///
/// Unlike [`ListBuilder`] which requires knowing the size at construction,
/// `ListAccumulator` supports dynamic growth via [`push`](Self::push). It uses
/// [`ReprotectSlot`] internally to maintain **O(1) protect stack usage** regardless
/// of how many elements are pushed.
///
/// # When to Use
///
/// | Scenario | Recommended Type |
/// |----------|-----------------|
/// | Known size | [`ListBuilder`] - more efficient, no reallocation |
/// | Unknown size | `ListAccumulator` - bounded stack, dynamic growth |
/// | Streaming/iterators | `ListAccumulator` or [`collect_list`] |
///
/// # Growth Strategy
///
/// The internal list grows exponentially (2x) when capacity is exceeded,
/// achieving amortized O(1) push. Elements are copied during growth.
///
/// # Example
///
/// ```ignore
/// unsafe fn collect_filtered(items: &[i32]) -> SEXP {
///     let scope = ProtectScope::new();
///     let mut acc = ListAccumulator::new(&scope, 4); // initial capacity hint
///
///     for &item in items {
///         if item > 0 {
///             acc.push(item);  // auto-converts via IntoR
///         }
///     }
///
///     acc.into_root().get()
/// }
/// ```
pub struct ListAccumulator<'a> {
    /// The current list container (protected via ReprotectSlot).
    list: ReprotectSlot<'a>,
    /// Temporary slot for element conversion and list growth.
    temp: ReprotectSlot<'a>,
    /// Number of elements currently in the list.
    len: usize,
    /// Current capacity of the list.
    cap: usize,
    /// Reference to the scope for creating the final Root.
    scope: &'a ProtectScope,
}

impl<'a> ListAccumulator<'a> {
    /// Create a new accumulator with the given initial capacity.
    ///
    /// A capacity of 0 is valid; the list will grow on first push.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn new(scope: &'a ProtectScope, initial_cap: usize) -> Self {
        let cap = initial_cap.max(1); // At least 1 to avoid edge cases
        let list_sexp = unsafe { ffi::Rf_allocVector(VECSXP, cap as isize) };
        let list = unsafe { scope.protect_with_index(list_sexp) };
        let temp = unsafe { scope.protect_with_index(ffi::R_NilValue) };

        Self {
            list,
            temp,
            len: 0,
            cap,
            scope,
        }
    }

    /// Push a value onto the accumulator.
    ///
    /// The value is converted to a SEXP via [`IntoR`] and inserted.
    /// If the internal list is full, it grows automatically.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn push<T: IntoR>(&mut self, value: T) {
        // Grow if needed
        if self.len >= self.cap {
            unsafe { self.grow() };
        }

        // Convert value using temp slot for protection during conversion
        let sexp = unsafe { self.temp.set_with(|| value.into_sexp()) };

        // Insert into list (list and temp are both protected)
        unsafe {
            ffi::SET_VECTOR_ELT(self.list.get(), self.len as isize, sexp);
        }

        self.len += 1;
    }

    /// Push a raw SEXP onto the accumulator.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `sexp` must be a valid SEXP (it will be temporarily protected)
    pub unsafe fn push_sexp(&mut self, sexp: SEXP) {
        // Grow if needed
        if self.len >= self.cap {
            unsafe { self.grow() };
        }

        // Protect the sexp during insertion using temp slot
        unsafe {
            self.temp.set(sexp);
            ffi::SET_VECTOR_ELT(self.list.get(), self.len as isize, sexp);
        }

        self.len += 1;
    }

    /// Grow the internal list by 2x.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    unsafe fn grow(&mut self) {
        let new_cap = self.cap.saturating_mul(2).max(4);

        // Allocate new list via temp slot (safe pattern)
        let old_list = self.list.get();
        unsafe {
            self.temp
                .set_with(|| ffi::Rf_allocVector(VECSXP, new_cap as isize));
        }
        let new_list = self.temp.get();

        // Copy existing elements
        for i in 0..self.len {
            let elem = unsafe { ffi::VECTOR_ELT(old_list, i as isize) };
            unsafe { ffi::SET_VECTOR_ELT(new_list, i as isize, elem) };
        }

        // Replace list slot with new list
        unsafe { self.list.set(new_list) };
        self.cap = new_cap;
    }

    /// Get the current number of elements.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the accumulator is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the current capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Finalize the accumulator and return a `Root` pointing to the list.
    ///
    /// The returned list is truncated to the actual length (if smaller than capacity).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn into_root(self) -> Root<'a> {
        // If len < cap, we need to shrink the list
        if self.len < self.cap {
            unsafe {
                let shrunk = ffi::Rf_xlengthgets(self.list.get(), self.len as isize);
                // The shrunk list might be the same or a new allocation
                // Either way, we protect it via the scope
                self.scope.protect(shrunk)
            }
        } else {
            // List is already the right size, create a Root without extra protection
            unsafe { self.scope.rooted(self.list.get()) }
        }
    }

    /// Finalize and return the raw SEXP.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn into_sexp(self) -> SEXP {
        unsafe { self.into_root().get() }
    }
}

/// Collect an iterator into an R list with bounded protect stack usage.
///
/// This is a convenience wrapper around [`ListAccumulator`] for iterator-based
/// collection. Each element is converted via [`IntoR`].
///
/// # Safety
///
/// Must be called from the R main thread.
///
/// # Example
///
/// ```ignore
/// unsafe fn squares(n: usize) -> SEXP {
///     let scope = ProtectScope::new();
///     collect_list(&scope, (0..n).map(|i| (i * i) as i32)).get()
/// }
/// ```
pub unsafe fn collect_list<'a, I, T>(scope: &'a ProtectScope, iter: I) -> Root<'a>
where
    I: IntoIterator<Item = T>,
    T: IntoR,
{
    let iter = iter.into_iter();
    let (lower, upper) = iter.size_hint();
    let initial_cap = upper.unwrap_or(lower).max(4);

    let mut acc = unsafe { ListAccumulator::new(scope, initial_cap) };

    for item in iter {
        unsafe { acc.push(item) };
    }

    unsafe { acc.into_root() }
}

impl ListMut {
    /// Wrap an existing `VECSXP` without additional checks.
    ///
    /// # Safety
    ///
    /// Caller must ensure `sexp` is a valid `VECSXP` and remains managed by R.
    #[inline]
    pub const unsafe fn from_raw(sexp: SEXP) -> Self {
        ListMut(sexp)
    }

    /// Get the underlying `SEXP`.
    #[inline]
    pub const fn as_sexp(&self) -> SEXP {
        self.0
    }

    /// Length of the list (number of elements).
    #[inline]
    pub fn len(&self) -> isize {
        unsafe { ffi::Rf_xlength(self.0) }
    }

    /// Returns true if the list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get raw SEXP element at 0-based index. Returns `None` if out of bounds.
    #[inline]
    pub fn get(&self, idx: isize) -> Option<SEXP> {
        if idx < 0 || idx >= self.len() {
            return None;
        }
        Some(unsafe { ffi::VECTOR_ELT(self.0, idx) })
    }

    /// Set raw SEXP element at 0-based index.
    #[inline]
    pub fn set(&mut self, idx: isize, value: SEXP) -> Result<(), SexpError> {
        if idx < 0 || idx >= self.len() {
            return Err(SexpError::InvalidValue("index out of bounds".into()));
        }
        unsafe { ffi::SET_VECTOR_ELT(self.0, idx, value) };
        Ok(())
    }
}

/// Convert things into an R list.
pub trait IntoList {
    fn into_list(self) -> List;
}

/// Fallible conversion from an R list into a Rust value.
pub trait TryFromList: Sized {
    type Error;

    fn try_from_list(list: List) -> Result<Self, Self::Error>;
}

impl<T: IntoR> IntoList for Vec<T> {
    fn into_list(self) -> List {
        // Convert elements first so any panics happen before touching R heap.
        // This also ensures all R allocations happen before we allocate the list,
        // so we don't need to protect the list during SET_VECTOR_ELT.
        let converted: Vec<SEXP> = self.into_iter().map(|v| v.into_sexp()).collect();
        let n = converted.len() as isize;
        unsafe {
            let list = ffi::Rf_allocVector(VECSXP, n);
            // No protection needed: SET_VECTOR_ELT doesn't allocate, and all
            // child SEXPs were allocated before `list`, so no GC can occur here.
            for (i, val) in converted.into_iter().enumerate() {
                ffi::SET_VECTOR_ELT(list, i as isize, val);
            }
            List(list)
        }
    }
}

impl<T> TryFromList for Vec<T>
where
    T: TryFromSexp<Error = SexpError>,
{
    type Error = SexpError;

    fn try_from_list(list: List) -> Result<Self, Self::Error> {
        let expected = list.len() as usize;
        let mut out = Vec::with_capacity(expected);
        for i in 0..expected {
            let sexp = list.get(i as isize).ok_or_else(|| {
                SexpError::from(SexpLengthError {
                    expected,
                    actual: i,
                })
            })?;
            out.push(TryFromSexp::try_from_sexp(sexp)?);
        }
        Ok(out)
    }
}

// =============================================================================
// HashMap conversions
// =============================================================================

impl<K, V> IntoList for HashMap<K, V>
where
    K: AsRef<str>,
    V: IntoR,
{
    fn into_list(self) -> List {
        let pairs: Vec<(K, V)> = self.into_iter().collect();
        List::from_pairs(pairs)
    }
}

impl<V> TryFromList for HashMap<String, V>
where
    V: TryFromSexp<Error = SexpError>,
{
    type Error = SexpError;

    fn try_from_list(list: List) -> Result<Self, Self::Error> {
        let n = list.len() as usize;
        let names_sexp = list.names();
        let mut map = HashMap::with_capacity(n);

        for i in 0..n {
            let sexp = list.get(i as isize).ok_or_else(|| {
                SexpError::from(SexpLengthError {
                    expected: n,
                    actual: i,
                })
            })?;
            let value: V = TryFromSexp::try_from_sexp(sexp)?;

            let key = if let Some(names) = names_sexp {
                let name_sexp = unsafe { ffi::STRING_ELT(names, i as isize) };
                if name_sexp == unsafe { ffi::R_NaString } {
                    format!("{i}")
                } else {
                    let name_ptr = unsafe { ffi::R_CHAR(name_sexp) };
                    let name_cstr = unsafe { std::ffi::CStr::from_ptr(name_ptr) };
                    name_cstr.to_str().unwrap_or(&format!("{i}")).to_string()
                }
            } else {
                format!("{i}")
            };

            map.insert(key, value);
        }
        Ok(map)
    }
}

// =============================================================================
// BTreeMap conversions
// =============================================================================

impl<K, V> IntoList for BTreeMap<K, V>
where
    K: AsRef<str>,
    V: IntoR,
{
    fn into_list(self) -> List {
        let pairs: Vec<(K, V)> = self.into_iter().collect();
        List::from_pairs(pairs)
    }
}

impl<V> TryFromList for BTreeMap<String, V>
where
    V: TryFromSexp<Error = SexpError>,
{
    type Error = SexpError;

    fn try_from_list(list: List) -> Result<Self, Self::Error> {
        let n = list.len() as usize;
        let names_sexp = list.names();
        let mut map = BTreeMap::new();

        for i in 0..n {
            let sexp = list.get(i as isize).ok_or_else(|| {
                SexpError::from(SexpLengthError {
                    expected: n,
                    actual: i,
                })
            })?;
            let value: V = TryFromSexp::try_from_sexp(sexp)?;

            let key = if let Some(names) = names_sexp {
                let name_sexp = unsafe { ffi::STRING_ELT(names, i as isize) };
                if name_sexp == unsafe { ffi::R_NaString } {
                    format!("{i}")
                } else {
                    let name_ptr = unsafe { ffi::R_CHAR(name_sexp) };
                    let name_cstr = unsafe { std::ffi::CStr::from_ptr(name_ptr) };
                    name_cstr.to_str().unwrap_or(&format!("{i}")).to_string()
                }
            } else {
                format!("{i}")
            };

            map.insert(key, value);
        }
        Ok(map)
    }
}

// =============================================================================
// HashSet conversions (unnamed list <-> set)
// =============================================================================

impl<T> IntoList for HashSet<T>
where
    T: IntoR,
{
    fn into_list(self) -> List {
        let values: Vec<T> = self.into_iter().collect();
        values.into_list()
    }
}

impl<T> TryFromList for HashSet<T>
where
    T: TryFromSexp<Error = SexpError> + Eq + Hash,
{
    type Error = SexpError;

    fn try_from_list(list: List) -> Result<Self, Self::Error> {
        let vec: Vec<T> = TryFromList::try_from_list(list)?;
        Ok(vec.into_iter().collect())
    }
}

// =============================================================================
// BTreeSet conversions (unnamed list <-> set)
// =============================================================================

impl<T> IntoList for BTreeSet<T>
where
    T: IntoR,
{
    fn into_list(self) -> List {
        let values: Vec<T> = self.into_iter().collect();
        values.into_list()
    }
}

impl<T> TryFromList for BTreeSet<T>
where
    T: TryFromSexp<Error = SexpError> + Ord,
{
    type Error = SexpError;

    fn try_from_list(list: List) -> Result<Self, Self::Error> {
        let vec: Vec<T> = TryFromList::try_from_list(list)?;
        Ok(vec.into_iter().collect())
    }
}

impl List {
    /// Build a list from `(name, value)` pairs, setting `names` in one pass.
    pub fn from_pairs<N, T>(pairs: Vec<(N, T)>) -> Self
    where
        N: AsRef<str>,
        T: IntoR,
    {
        let raw: Vec<(N, SEXP)> = pairs.into_iter().map(|(n, v)| (n, v.into_sexp())).collect();
        Self::from_raw_pairs(raw)
    }

    /// Build an unnamed list from values.
    ///
    /// Use this for tuple-like structures where positional access is more natural.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let list = List::from_values(vec![1i32, 2i32, 3i32]);
    /// // R: list(1L, 2L, 3L) - accessed as [[1]], [[2]], [[3]]
    /// ```
    pub fn from_values<T: IntoR>(values: Vec<T>) -> Self {
        values.into_list()
    }

    /// Build an unnamed list from pre-converted SEXPs.
    ///
    /// # Safety Note
    ///
    /// The input SEXPs should already be protected or be children of protected
    /// containers. This function protects the list during construction.
    pub fn from_raw_values(values: Vec<SEXP>) -> Self {
        let n = values.len() as isize;
        unsafe {
            // Protect list during construction. SET_VECTOR_ELT doesn't allocate,
            // but we protect defensively in case this code is modified later.
            let list = OwnedProtect::new(ffi::Rf_allocVector(VECSXP, n));
            for (i, val) in values.into_iter().enumerate() {
                ffi::SET_VECTOR_ELT(list.get(), i as isize, val);
            }
            List(list.get())
        }
    }

    /// Build a list from `(name, SEXP)` pairs (heterogeneous-friendly).
    ///
    /// # Safety Note
    ///
    /// The input SEXPs should already be protected or be children of protected
    /// containers. This function protects the list and names vector during
    /// construction.
    pub fn from_raw_pairs<N>(pairs: Vec<(N, SEXP)>) -> Self
    where
        N: AsRef<str>,
    {
        let n = pairs.len() as isize;
        unsafe {
            // CRITICAL: Both list and names must be protected because
            // Rf_mkCharLenCE can allocate and trigger GC in the loop below.
            let list = OwnedProtect::new(ffi::Rf_allocVector(VECSXP, n));
            let names = OwnedProtect::new(ffi::Rf_allocVector(STRSXP, n));
            for (i, (name, val)) in pairs.into_iter().enumerate() {
                ffi::SET_VECTOR_ELT(list.get(), i as isize, val);

                let s = name.as_ref();
                // Rf_mkCharLenCE allocates - list and names must be protected!
                let chars = ffi::Rf_mkCharLenCE(s.as_ptr().cast(), s.len() as i32, ffi::CE_UTF8);
                ffi::SET_STRING_ELT(names.get(), i as isize, chars);
            }
            ffi::Rf_namesgets(list.get(), names.get());
            List(list.get())
        }
    }
}

impl IntoR for List {
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.0
    }
}

impl IntoR for ListMut {
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.0
    }
}

/// Error when a list has duplicate non-NA names.
#[derive(Debug, Clone)]
pub struct DuplicateNameError {
    /// The duplicate name that was found.
    pub name: String,
}

impl std::fmt::Display for DuplicateNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "list has duplicate name: {:?}", self.name)
    }
}

impl std::error::Error for DuplicateNameError {}

/// Error when converting SEXP to List fails.
#[derive(Debug, Clone)]
pub enum ListFromSexpError {
    /// Wrong SEXP type.
    Type(crate::from_r::SexpTypeError),
    /// Duplicate non-NA name found.
    DuplicateName(DuplicateNameError),
}

impl std::fmt::Display for ListFromSexpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListFromSexpError::Type(e) => write!(f, "{}", e),
            ListFromSexpError::DuplicateName(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ListFromSexpError {}

impl From<crate::from_r::SexpTypeError> for ListFromSexpError {
    fn from(e: crate::from_r::SexpTypeError) -> Self {
        ListFromSexpError::Type(e)
    }
}

impl TryFromSexp for List {
    type Error = ListFromSexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = unsafe { ffi::TYPEOF(sexp) };

        // Accept VECSXP (generic list) directly
        // Also accept LISTSXP (pairlist) by coercing to VECSXP
        // Note: Rf_isList() only returns true for LISTSXP/NILSXP, not VECSXP
        let list_sexp = if actual == VECSXP {
            sexp
        } else if actual == LISTSXP {
            // Accept pairlists by coercing to a VECSXP list.
            unsafe { ffi::Rf_coerceVector(sexp, VECSXP) }
        } else {
            return Err(crate::from_r::SexpTypeError {
                expected: VECSXP,
                actual,
            }
            .into());
        };

        // Check for duplicate non-NA names
        let names_sexp = unsafe { ffi::Rf_getAttrib(list_sexp, ffi::R_NamesSymbol) };
        if names_sexp != unsafe { ffi::R_NilValue } {
            let n = unsafe { ffi::Rf_xlength(list_sexp) };
            let mut seen = HashSet::new();

            for i in 0..n {
                let name_sexp = unsafe { ffi::STRING_ELT(names_sexp, i) };
                // Skip NA names
                if name_sexp == unsafe { ffi::R_NaString } {
                    continue;
                }
                // Skip empty names
                let name_ptr = unsafe { ffi::R_CHAR(name_sexp) };
                let name_cstr = unsafe { std::ffi::CStr::from_ptr(name_ptr) };
                if let Ok(s) = name_cstr.to_str() {
                    if s.is_empty() {
                        continue;
                    }
                    if !seen.insert(s) {
                        return Err(ListFromSexpError::DuplicateName(DuplicateNameError {
                            name: s.to_string(),
                        }));
                    }
                }
            }
        }

        Ok(List(list_sexp))
    }
}

impl TryFromSexp for Option<List> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp == unsafe { ffi::R_NilValue } {
            return Ok(None);
        }
        let list = List::try_from_sexp(sexp).map_err(|e| SexpError::InvalidValue(e.to_string()))?;
        Ok(Some(list))
    }
}

impl TryFromSexp for Option<ListMut> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp == unsafe { ffi::R_NilValue } {
            return Ok(None);
        }
        let list = ListMut::try_from_sexp(sexp)?;
        Ok(Some(list))
    }
}

impl TryFromSexp for ListMut {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = unsafe { ffi::TYPEOF(sexp) };
        if actual != VECSXP {
            return Err(SexpTypeError {
                expected: VECSXP,
                actual,
            }
            .into());
        }
        Ok(ListMut(sexp))
    }
}
