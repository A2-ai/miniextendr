#![allow(rustdoc::private_intra_doc_links)]
//! Thin wrapper around R list (`VECSXP`).
//!
//! Provides safe construction from Rust values and typed extraction.
//!
//! # Submodules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`accumulator`] | `ListAccumulator` — dynamic list construction with bounded protect stack |
//! | [`named`] | `NamedList` — O(1) name-indexed access via `HashMap` index |
//!
//! # Core Types
//!
//! - [`List`] — owned handle to an R list (VECSXP)
//! - [`ListMut`] — mutable view for in-place element replacement
//! - [`ListBuilder`] — fixed-size batch construction
//! - [`IntoList`] / [`TryFromList`] — conversion traits

use crate::SEXPTYPE::{LISTSXP, STRSXP, VECSXP};
use crate::from_r::{SexpError, SexpLengthError, SexpTypeError, TryFromSexp};
use crate::gc_protect::OwnedProtect;
use crate::into_r::IntoR;
use crate::sys::{self};
use crate::{SEXP, SexpExt};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

/// Owned handle to an R list (`VECSXP`).
///
/// # Examples
///
/// ```no_run
/// use miniextendr_api::list::List;
///
/// let list = List::from_values(vec![1i32, 2, 3]);
/// assert_eq!(list.len(), 3);
/// let first: Option<i32> = list.get_index(0);
/// ```
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
    ///
    /// Uses `SexpExt::is_list` (VECSXP check) — **not** `is_pair_list` (LISTSXP).
    #[inline]
    pub fn is_list(self) -> bool {
        <SEXP as crate::SexpExt>::is_list(&self.0)
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
        self.0.xlength()
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
        Some(self.0.vector_elt(idx))
    }

    /// Get element at 0-based index and convert to type `T`.
    ///
    /// Returns `None` if index is out of bounds or conversion fails.
    ///
    /// The conversion error is discarded, so `T`'s `TryFromSexp::Error` is
    /// unconstrained — any element type works, not only those whose error is
    /// `SexpError`. Callers that need the error (e.g. to distinguish "missing"
    /// from "wrong type") should use [`get`](Self::get) and convert directly.
    #[inline]
    pub fn get_index<T>(self, idx: isize) -> Option<T>
    where
        T: TryFromSexp,
    {
        let sexp = self.get(idx)?;
        T::try_from_sexp(sexp).ok()
    }

    /// The names this list can be looked up by: `(position, name)` for each
    /// element whose name is neither `NA` nor invalid UTF-8, in list order.
    /// Empty names are included. Yields nothing when the list has no `names`
    /// attribute.
    ///
    /// [`get_named_sexp`](Self::get_named_sexp),
    /// [`first_duplicate_name`](Self::first_duplicate_name), [`NamedList`] and
    /// the `#[derive(TryFromList)]` field lookup all walk the names through
    /// this, so they agree on which names count.
    ///
    /// The `&str`s point into R's CHARSXP cache; they stay valid while the
    /// list (which roots its `names` attribute) is reachable.
    fn text_names(self) -> impl Iterator<Item = (isize, &'static str)> {
        let names = self.names();
        let n = if names.is_some() { self.len() } else { 0 };
        (0..n).filter_map(move |i| {
            let name_sexp = names?.string_elt(i);
            if name_sexp == SEXP::na_string() {
                return None;
            }
            let name_cstr = unsafe { std::ffi::CStr::from_ptr(name_sexp.r_char()) };
            name_cstr.to_str().ok().map(|s| (i, s))
        })
    }

    /// Get the raw element `SEXP` associated with `name`, without conversion.
    ///
    /// Returns the element exactly as stored so callers can convert it with any
    /// [`TryFromSexp`] error type — not only those whose error is `SexpError`.
    /// Returns `None` when the list has no `names` attribute or no name matches.
    ///
    /// When the name appears more than once, the first element with that name
    /// is returned, as R's `x[["name"]]` does.
    pub fn get_named_sexp(self, name: &str) -> Option<SEXP> {
        self.text_names()
            .find(|&(_, s)| s == name)
            .map(|(i, _)| self.0.vector_elt(i))
    }

    /// The first name that appears more than once in the list, if any.
    ///
    /// An R list may repeat a name (`list(a = 1, a = 2)`), and a `List`
    /// argument accepts such a list, as R does. A function that needs each
    /// name to be unique checks for it itself, and words its own error:
    ///
    /// ```no_run
    /// use miniextendr_api::{List, miniextendr};
    ///
    /// #[miniextendr]
    /// pub fn settings_count(settings: List) -> i32 {
    ///     if let Some(name) = settings.first_duplicate_name() {
    ///         panic!("`settings` has the name {name:?} more than once; each entry needs a unique name");
    ///     }
    ///     settings.len() as i32
    /// }
    /// ```
    ///
    /// Only names that can identify an element are compared: `NA` and empty
    /// names are skipped (R lets any number of elements go unnamed), and so
    /// are names that are not valid UTF-8. Returns `None` for a list without
    /// a `names` attribute.
    pub fn first_duplicate_name(self) -> Option<String> {
        let mut seen = HashSet::new();
        self.text_names()
            .map(|(_, s)| s)
            .filter(|s| !s.is_empty())
            .find(|s| !seen.insert(*s))
            .map(str::to_owned)
    }

    /// Positions of a derived struct's fields, for the code
    /// `#[derive(TryFromList)]` generates. Not public API.
    ///
    /// `sorted_fields` holds the struct's field names in ascending byte
    /// order (the derive sorts them); entry `k` of the result is the position
    /// of the first element named `sorted_fields[k]`, or `None` when no
    /// element has that name. One pass over the names, `O(n log fields)`.
    ///
    /// A field name that appears more than once is an error
    /// ([`SexpError::DuplicateName`]): the struct could only keep one of the
    /// values. Other repeated names are ignored, as names that are not fields
    /// are.
    #[doc(hidden)]
    pub fn __mx_field_positions<const N: usize>(
        self,
        sorted_fields: [&str; N],
    ) -> Result<[Option<isize>; N], SexpError> {
        debug_assert!(sorted_fields.is_sorted(), "field names must be sorted");
        let mut positions = [None; N];
        for (i, name) in self.text_names() {
            if let Ok(k) = sorted_fields.binary_search(&name) {
                if positions[k].is_some() {
                    return Err(SexpError::DuplicateName(name.to_owned()));
                }
                positions[k] = Some(i);
            }
        }
        Ok(positions)
    }

    /// Get element by name and convert to type `T`.
    ///
    /// Returns `None` if name not found or conversion fails.
    ///
    /// The conversion error is discarded, so `T`'s `TryFromSexp::Error` is
    /// unconstrained. Use [`get_named_sexp`](Self::get_named_sexp) and convert
    /// directly when you need to inspect the conversion failure.
    pub fn get_named<T>(self, name: &str) -> Option<T>
    where
        T: TryFromSexp,
    {
        let sexp = self.get_named_sexp(name)?;
        T::try_from_sexp(sexp).ok()
    }

    // region: Attribute getters (equivalent to R's GET_* macros)

    /// Get an arbitrary attribute by symbol, returning `None` for `R_NilValue`.
    #[inline]
    fn get_attr_opt(self, name: SEXP) -> Option<SEXP> {
        let attr = self.0.get_attr(name);
        if attr.is_nil() { None } else { Some(attr) }
    }

    /// Get the `names` attribute if present.
    #[inline]
    pub fn names(self) -> Option<SEXP> {
        self.get_attr_opt(SEXP::names_symbol())
    }

    /// Get the `class` attribute if present.
    #[inline]
    pub fn get_class(self) -> Option<SEXP> {
        self.get_attr_opt(SEXP::class_symbol())
    }

    /// Get the `dim` attribute if present.
    #[inline]
    pub fn get_dim(self) -> Option<SEXP> {
        self.get_attr_opt(SEXP::dim_symbol())
    }

    /// Get the `dimnames` attribute if present.
    #[inline]
    pub fn get_dimnames(self) -> Option<SEXP> {
        self.get_attr_opt(SEXP::dimnames_symbol())
    }

    /// Get row names from the `dimnames` attribute.
    #[inline]
    pub fn get_rownames(self) -> Option<SEXP> {
        let rownames = unsafe { sys::Rf_GetRowNames(self.0) };
        if rownames.is_nil() {
            None
        } else {
            Some(rownames)
        }
    }

    /// Get column names from the `dimnames` attribute.
    #[inline]
    pub fn get_colnames(self) -> Option<SEXP> {
        let dimnames = self.0.get_dimnames();
        if dimnames.is_nil() {
            return None;
        }
        let colnames = unsafe { sys::Rf_GetColNames(dimnames) };
        if colnames.is_nil() {
            None
        } else {
            Some(colnames)
        }
    }

    /// Get the `levels` attribute if present (for factors).
    #[inline]
    pub fn get_levels(self) -> Option<SEXP> {
        self.get_attr_opt(SEXP::levels_symbol())
    }

    /// Get the `tsp` attribute if present (for time series).
    #[inline]
    pub fn get_tsp(self) -> Option<SEXP> {
        self.get_attr_opt(SEXP::tsp_symbol())
    }
    // endregion

    // region: Attribute setters (equivalent to R's SET_* macros)

    /// Set the `names` attribute; returns the same list for chaining.
    ///
    /// Equivalent to R's `SET_NAMES(x, n)`.
    #[inline]
    pub fn set_names(self, names: SEXP) -> Self {
        self.0.set_names(names);
        self
    }

    /// Set the `class` attribute; returns the same list for chaining.
    ///
    /// Equivalent to R's `SET_CLASS(x, n)`.
    #[inline]
    pub fn set_class(self, class: SEXP) -> Self {
        self.0.set_class(class);
        self
    }

    /// Set the `dim` attribute; returns the same list for chaining.
    ///
    /// Equivalent to R's `SET_DIM(x, n)`.
    #[inline]
    pub fn set_dim(self, dim: SEXP) -> Self {
        self.0.set_dim(dim);
        self
    }

    /// Set the `dimnames` attribute; returns the same list for chaining.
    ///
    /// Equivalent to R's `SET_DIMNAMES(x, n)`.
    #[inline]
    pub fn set_dimnames(self, dimnames: SEXP) -> Self {
        self.0.set_dimnames(dimnames);
        self
    }

    /// Set the `levels` attribute; returns the same list for chaining.
    ///
    /// Equivalent to R's `SET_LEVELS(x, l)`.
    #[inline]
    pub fn set_levels(self, levels: SEXP) -> Self {
        self.0.set_levels(levels);
        self
    }
    // endregion

    // region: Convenience setters (string-based)

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
        use crate::SEXPTYPE::STRSXP;

        let n: isize = classes
            .len()
            .try_into()
            .expect("classes length exceeds isize::MAX");
        unsafe {
            // Protect self across the class-vector allocation; otherwise the
            // parent list can be freed during `Rf_allocVector` while it sits
            // unrooted in our Rust handle (UAF under gctorture).
            let _self_guard = OwnedProtect::new(self.0);
            let class_vec = OwnedProtect::new(sys::Rf_allocVector(STRSXP, n));
            for (i, class) in classes.iter().enumerate() {
                let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                class_vec.get().set_string_elt(idx, SEXP::charsxp(class));
            }
            self.0.set_class(class_vec.get());
        }
        self
    }

    /// Set class = `"data.frame"` using a cached class STRSXP.
    ///
    /// Equivalent to `set_class_str(&["data.frame"])` but avoids allocation.
    #[inline]
    pub fn set_data_frame_class(self) -> Self {
        self.0
            .set_class(crate::cached_class::data_frame_class_sexp());
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
        use crate::SEXPTYPE::STRSXP;

        let n: isize = names
            .len()
            .try_into()
            .expect("names length exceeds isize::MAX");
        unsafe {
            // Protect self across the names-vector allocation; see set_class_str.
            let _self_guard = OwnedProtect::new(self.0);
            let names_vec = OwnedProtect::new(sys::Rf_allocVector(STRSXP, n));
            for (i, name) in names.iter().enumerate() {
                let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                names_vec.get().set_string_elt(idx, SEXP::charsxp(name));
            }
            self.0.set_names(names_vec.get());
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
        unsafe {
            // Protect self across the row.names allocation; see set_class_str.
            let _self_guard = OwnedProtect::new(self.0);
            // R's compact row.names: c(NA_integer_, -n)
            let (row_names, rn) = crate::into_r::alloc_r_vector::<i32>(2);
            let _guard = OwnedProtect::new(row_names);
            rn[0] = i32::MIN; // NA_INTEGER
            let n_i32 = i32::try_from(n).unwrap_or_else(|_| {
                panic!("row count {n} exceeds i32::MAX");
            });
            rn[1] = -n_i32;
            self.0.set_row_names(row_names);
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
        use crate::SEXPTYPE::STRSXP;

        let n: isize = row_names
            .len()
            .try_into()
            .expect("row_names length exceeds isize::MAX");
        unsafe {
            // Protect self across the row.names allocation; see set_class_str.
            let _self_guard = OwnedProtect::new(self.0);
            let names_vec = OwnedProtect::new(sys::Rf_allocVector(STRSXP, n));
            for (i, name) in row_names.iter().enumerate() {
                let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                names_vec.get().set_string_elt(idx, SEXP::charsxp(name));
            }
            self.0.set_row_names(names_vec.get());
        }
        self
    }
    // endregion

    // region: Safe element insertion

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
    /// let list = List::from_raw(scope.alloc_vecsxp(n).into_raw());
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
            self.0.set_vector_elt(idx, child);
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
        self.0.set_vector_elt(idx, child);
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
    /// let list = List::from_raw(scope.alloc_vecsxp(n).into_raw());
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
            self.0.set_vector_elt(idx, child.get());
        }
    }
    // endregion
}

// region: ListBuilder - efficient batch list construction

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
///         let child = scope.alloc_real(10).into_raw();
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
    pub unsafe fn new(scope: &'a ProtectScope, len: usize) -> Self {
        // SAFETY: caller guarantees R main thread
        let list = unsafe { scope.alloc_vecsxp(len).into_raw() };
        Self {
            list,
            _scope: scope,
        }
    }

    /// Create a new list builder via the **unchecked** FFI allocation path.
    ///
    /// `_unchecked` twin of [`new`](Self::new): the VECSXP is allocated with
    /// `Rf_allocVector_unchecked` (see [`ProtectScope::alloc_vecsxp_unchecked`]),
    /// bypassing the main-thread assertion. Use inside ALTREP callbacks,
    /// `with_r_unwind_protect`, or `with_r_thread` bodies, and pair element
    /// insertion with [`set_unchecked`](Self::set_unchecked).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread, in a context where the checked-FFI
    /// assertion is intentionally bypassed (see CLAUDE.md "FFI thread checking").
    #[inline]
    pub unsafe fn new_unchecked(scope: &'a ProtectScope, len: usize) -> Self {
        // SAFETY: caller guarantees R main thread in a checked-bypass context.
        let list = unsafe { scope.alloc_vecsxp_unchecked(len).into_raw() };
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
        debug_assert!(idx >= 0 && idx < self.list.xlength());
        self.list.set_vector_elt(idx, child);
    }

    /// Set an element via the **unchecked** FFI path.
    ///
    /// `_unchecked` twin of [`set`](Self::set): inserts with
    /// `set_vector_elt_unchecked`, bypassing the main-thread assertion. Pair with
    /// [`new_unchecked`](Self::new_unchecked) inside ALTREP callbacks,
    /// `with_r_unwind_protect`, or `with_r_thread` bodies.
    ///
    /// # Safety
    ///
    /// - `child` must be a valid SEXP, already protected (typically by the same
    ///   scope, or by being inserted into this protected parent immediately after
    ///   allocation with no intervening allocation)
    /// - Must be called from the R main thread, in a checked-bypass context
    #[inline]
    pub unsafe fn set_unchecked(&self, idx: isize, child: SEXP) {
        // SAFETY: caller guarantees valid, protected child in a checked-bypass context.
        debug_assert!(idx >= 0 && idx < self.list.xlength());
        unsafe { self.list.set_vector_elt_unchecked(idx, child) };
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
            debug_assert!(idx >= 0 && idx < self.list.xlength());
            let _guard = OwnedProtect::new(child);
            self.list.set_vector_elt(idx, child);
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
        self.list.xlength()
    }

    /// Check if the list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
// endregion

mod accumulator;
mod named;

pub use accumulator::*;
pub use named::*;

// region: IntoList and TryFromList traits

/// Convert things into an R list.
pub trait IntoList {
    /// Convert `self` into an R list wrapper.
    fn into_list(self) -> List;
}

/// Fallible conversion from an R list into a Rust value.
pub trait TryFromList: Sized {
    /// Error returned when conversion fails.
    type Error;

    /// Attempt to convert an R list wrapper into `Self`.
    fn try_from_list(list: List) -> Result<Self, Self::Error>;
}

impl<T: IntoR> IntoList for Vec<T> {
    fn into_list(self) -> List {
        // Allocate + protect the parent first, then call `into_sexp()` per
        // element and write straight into the parent. Pre-collecting elements
        // into `Vec<SEXP>` would leave them unrooted across allocations — same
        // UAF shape as the columnar `Generic` buffer (PR #424 / issue #307).
        let n: isize = self
            .len()
            .try_into()
            .expect("list length exceeds isize::MAX");
        unsafe {
            let list = OwnedProtect::new(sys::Rf_allocVector(VECSXP, n));
            for (i, val) in self.into_iter().enumerate() {
                let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                list.get().set_vector_elt(idx, val.into_sexp());
            }
            List(list.get())
        }
    }
}

impl<T> TryFromList for Vec<T>
where
    T: TryFromSexp<Error = SexpError>,
{
    type Error = SexpError;

    fn try_from_list(list: List) -> Result<Self, Self::Error> {
        let expected: usize = list
            .len()
            .try_into()
            .expect("list length must be non-negative");
        let mut out = Vec::with_capacity(expected);
        for i in 0..expected {
            let idx: isize = i.try_into().expect("index exceeds isize::MAX");
            let sexp = list.get(idx).ok_or_else(|| {
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

// endregion

// region: HashMap conversions

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

/// Keys are the element names; an element without a usable name (no `names`
/// attribute, `NA`, or not valid UTF-8) is keyed by its 0-based position, and
/// empty names share the key `""` (the last one wins). A non-empty name that
/// appears more than once is an error ([`SexpError::DuplicateName`]), as for
/// the `TryFromSexp` map conversion.
impl<V> TryFromList for HashMap<String, V>
where
    V: TryFromSexp<Error = SexpError>,
{
    type Error = SexpError;

    fn try_from_list(list: List) -> Result<Self, Self::Error> {
        // A repeated name would leave only one of its values in the map.
        if let Some(name) = list.first_duplicate_name() {
            return Err(SexpError::DuplicateName(name));
        }
        let n: usize = list
            .len()
            .try_into()
            .expect("list length must be non-negative");
        let names_sexp = list.names();
        let mut map = HashMap::with_capacity(n);

        for i in 0..n {
            let idx: isize = i.try_into().expect("index exceeds isize::MAX");
            let sexp = list.get(idx).ok_or_else(|| {
                SexpError::from(SexpLengthError {
                    expected: n,
                    actual: i,
                })
            })?;
            let value: V = TryFromSexp::try_from_sexp(sexp)?;

            let key = if let Some(names) = names_sexp {
                let name_sexp = names.string_elt(idx);
                if name_sexp == SEXP::na_string() {
                    format!("{i}")
                } else {
                    let name_ptr = name_sexp.r_char();
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
// endregion

// region: BTreeMap conversions

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

/// Keys are the element names; an element without a usable name (no `names`
/// attribute, `NA`, or not valid UTF-8) is keyed by its 0-based position, and
/// empty names share the key `""` (the last one wins). A non-empty name that
/// appears more than once is an error ([`SexpError::DuplicateName`]), as for
/// the `TryFromSexp` map conversion.
impl<V> TryFromList for BTreeMap<String, V>
where
    V: TryFromSexp<Error = SexpError>,
{
    type Error = SexpError;

    fn try_from_list(list: List) -> Result<Self, Self::Error> {
        // A repeated name would leave only one of its values in the map.
        if let Some(name) = list.first_duplicate_name() {
            return Err(SexpError::DuplicateName(name));
        }
        let n: usize = list
            .len()
            .try_into()
            .expect("list length must be non-negative");
        let names_sexp = list.names();
        let mut map = BTreeMap::new();

        for i in 0..n {
            let idx: isize = i.try_into().expect("index exceeds isize::MAX");
            let sexp = list.get(idx).ok_or_else(|| {
                SexpError::from(SexpLengthError {
                    expected: n,
                    actual: i,
                })
            })?;
            let value: V = TryFromSexp::try_from_sexp(sexp)?;

            let key = if let Some(names) = names_sexp {
                let name_sexp = names.string_elt(idx);
                if name_sexp == SEXP::na_string() {
                    format!("{i}")
                } else {
                    let name_ptr = name_sexp.r_char();
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
// endregion

// region: HashSet conversions (unnamed list <-> set)

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
// endregion

// region: BTreeSet conversions (unnamed list <-> set)

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
        // Allocate + protect the parent list and names before calling
        // `into_sexp()` on each value. Pre-collecting `Vec<(N, SEXP)>` would
        // leave the value SEXPs unrooted across subsequent `into_sexp()` and
        // the names allocation — same UAF shape as #307.
        let n: isize = pairs
            .len()
            .try_into()
            .expect("pairs length exceeds isize::MAX");
        unsafe {
            let list = OwnedProtect::new(sys::Rf_allocVector(VECSXP, n));
            let names = OwnedProtect::new(sys::Rf_allocVector(STRSXP, n));
            for (i, (name, val)) in pairs.into_iter().enumerate() {
                let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                list.get().set_vector_elt(idx, val.into_sexp());
                names
                    .get()
                    .set_string_elt(idx, SEXP::charsxp(name.as_ref()));
            }
            list.get().set_names(names.get());
            List(list.get())
        }
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
        let n: isize = values
            .len()
            .try_into()
            .expect("values length exceeds isize::MAX");
        unsafe {
            // Protect list during construction. SET_VECTOR_ELT doesn't allocate,
            // but we protect defensively in case this code is modified later.
            let list = OwnedProtect::new(sys::Rf_allocVector(VECSXP, n));
            for (i, val) in values.into_iter().enumerate() {
                let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                list.get().set_vector_elt(idx, val);
            }
            List(list.get())
        }
    }

    /// Build an atomic vector from homogeneous length-1 scalar SEXPs.
    ///
    /// If all elements are length-1 scalars of the same coalesceable type
    /// (INTSXP, REALSXP, LGLSXP, STRSXP), returns that atomic vector.
    /// Otherwise returns a VECSXP (generic list).
    ///
    /// This is the canonical entry point for both `DataFrame::into_data_frame`
    /// (column building) and `SeqSerializer::end` (sequence coalescing).
    ///
    /// # Safety Note
    ///
    /// The input SEXPs should already be protected or be children of protected
    /// containers.
    pub fn from_scalars_or_list(elements: &[SEXP]) -> Self {
        use crate::SEXPTYPE;
        use crate::into_r::alloc_r_vector;

        if elements.is_empty() {
            return Self::from_raw_values(Vec::new());
        }

        let first_type = elements[0].type_of();
        let all_scalar_same_type = elements
            .iter()
            .all(|&e| e.xlength() == 1 && e.type_of() == first_type);

        if !all_scalar_same_type {
            return Self::from_raw_values(elements.to_vec());
        }

        let n = elements.len();
        let sexp = match first_type {
            // For native types: allocate R vector, get mutable slice, read source
            // scalars via as_slice()[0] — no per-element FFI calls.
            SEXPTYPE::INTSXP => unsafe {
                let (v, dst) = alloc_r_vector::<i32>(n);
                for (slot, &elem) in dst.iter_mut().zip(elements.iter()) {
                    *slot = *elem.as_slice::<i32>().first().expect("scalar has length 1");
                }
                v
            },
            SEXPTYPE::REALSXP => unsafe {
                let (v, dst) = alloc_r_vector::<f64>(n);
                for (slot, &elem) in dst.iter_mut().zip(elements.iter()) {
                    *slot = *elem.as_slice::<f64>().first().expect("scalar has length 1");
                }
                v
            },
            SEXPTYPE::LGLSXP => unsafe {
                let (v, dst) = alloc_r_vector::<crate::RLogical>(n);
                for (slot, &elem) in dst.iter_mut().zip(elements.iter()) {
                    *slot = *elem
                        .as_slice::<crate::RLogical>()
                        .first()
                        .expect("scalar has length 1");
                }
                v
            },
            // STRSXP elements are CHARSXPs — must use SET_STRING_ELT (no slice access).
            SEXPTYPE::STRSXP => unsafe {
                let v = OwnedProtect::new(sys::Rf_allocVector(SEXPTYPE::STRSXP, n as isize));
                for (i, &elem) in elements.iter().enumerate() {
                    let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                    v.get().set_string_elt(idx, elem.string_elt(0));
                }
                v.get()
            },
            _ => return Self::from_raw_values(elements.to_vec()),
        };
        List(sexp)
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
        let n: isize = pairs
            .len()
            .try_into()
            .expect("pairs length exceeds isize::MAX");
        unsafe {
            // CRITICAL: Both list and names must be protected because
            // Rf_mkCharLenCE can allocate and trigger GC in the loop below.
            let list = OwnedProtect::new(sys::Rf_allocVector(VECSXP, n));
            let names = OwnedProtect::new(sys::Rf_allocVector(STRSXP, n));
            for (i, (name, val)) in pairs.into_iter().enumerate() {
                let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                list.get().set_vector_elt(idx, val);

                let s = name.as_ref();
                // SEXP::charsxp allocates - list and names must be protected!
                names.get().set_string_elt(idx, SEXP::charsxp(s));
            }
            list.get().set_names(names.get());
            List(list.get())
        }
    }

    /// Build an empty named-list SEXP (zero elements, `names` attribute set).
    ///
    /// Equivalent to [`Self::from_raw_pairs`]`(vec![])`, but avoids the
    /// `Vec<(&str, SEXP)>` type annotation that Rust requires at empty-vector
    /// callsites where type inference cannot resolve the element type.
    ///
    /// Codegen paths that emit an empty `from_raw_pairs` call (e.g. unit-variant
    /// partitions in `#[derive(DataFrameRow)]`) use this helper so that a future
    /// signature change to `from_raw_pairs` only needs to be updated in one
    /// place.
    #[must_use]
    pub fn from_raw_pairs_empty() -> Self {
        Self::from_raw_pairs(Vec::<(&str, SEXP)>::new())
    }
}

impl IntoR for List {
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

impl IntoR for ListMut {
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

/// `Some(list)` → the list, `None` → `NULL`: the inverse of
/// `TryFromSexp for Option<List>`.
impl IntoR for Option<List> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.map_or_else(SEXP::nil, |list| list.0)
    }
}

/// `Some(list)` → the list, `None` → `NULL`: the inverse of
/// `TryFromSexp for Option<ListMut>`.
impl IntoR for Option<ListMut> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.map_or_else(SEXP::nil, |list| list.0)
    }
}

/// Build an R list-column (VECSXP) from values converted one at a time through
/// [`IntoList`].
///
/// The column is allocated and protected first; each value's list is stored in
/// it as soon as it is built, so no list is ever unrooted while the next one
/// allocates. Collecting a `Vec<List>` first and converting that would leave
/// every earlier list unrooted across the later allocations (#1748).
///
/// Used by `#[derive(DataFrameRow)]` for `#[dataframe(as_list)]` struct-typed
/// fields.
#[doc(hidden)]
pub fn into_list_column<T: IntoList>(values: Vec<T>) -> SEXP {
    let n: isize = values
        .len()
        .try_into()
        .expect("list-column length exceeds isize::MAX");
    unsafe {
        let out = OwnedProtect::new(sys::Rf_allocVector(VECSXP, n));
        for (i, value) in values.into_iter().enumerate() {
            let idx: isize = i.try_into().expect("index exceeds isize::MAX");
            // No allocation between building the list and storing it.
            out.get().set_vector_elt(idx, value.into_list().0);
        }
        out.get()
    }
}

/// [`into_list_column`] for optional cells: `None` becomes `NULL`.
#[doc(hidden)]
pub fn into_option_list_column<T: IntoList>(values: Vec<Option<T>>) -> SEXP {
    let n: isize = values
        .len()
        .try_into()
        .expect("list-column length exceeds isize::MAX");
    unsafe {
        // VECSXP slots start as R_NilValue, so `None` cells need no write.
        let out = OwnedProtect::new(sys::Rf_allocVector(VECSXP, n));
        for (i, value) in values.into_iter().enumerate() {
            if let Some(value) = value {
                let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                out.get().set_vector_elt(idx, value.into_list().0);
            }
        }
        out.get()
    }
}

/// Convert a `Vec<List>` to an R list-column (VECSXP).
///
/// Each element becomes an R list in the output VECSXP. The elements must be
/// rooted by the caller (for example, elements of an argument list): this
/// allocates the column first. To build the lists as you go, use
/// [`into_list_column`].
impl IntoR for Vec<List> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        unsafe {
            use crate::gc_protect::OwnedProtect;
            use crate::sys::Rf_allocVector;
            use crate::{SEXPTYPE, SexpExt as _};
            let n = self.len() as crate::R_xlen_t;
            // OwnedProtect guards `out` across the per-element fill; its Drop
            // issues the matching UNPROTECT(1) (RAII — count balanced by
            // construction).
            let out = OwnedProtect::new(Rf_allocVector(SEXPTYPE::VECSXP, n));
            for (i, list) in self.into_iter().enumerate() {
                out.get().set_vector_elt(i as crate::R_xlen_t, list.0);
            }
            *out
        }
    }
}

/// Convert a `Vec<Option<List>>` to an R list-column (VECSXP).
///
/// `Some(list)` elements are placed directly as list elements; `None` elements
/// become `R_NilValue`. As with `Vec<List>`, the elements must be rooted by
/// the caller; [`into_option_list_column`] builds them as it goes.
impl IntoR for Vec<Option<List>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        unsafe {
            use crate::gc_protect::OwnedProtect;
            use crate::sys::Rf_allocVector;
            use crate::{SEXPTYPE, SexpExt as _};
            let n = self.len() as crate::R_xlen_t;
            // VECSXP slots are zero-initialised to R_NilValue by Rf_allocVector,
            // so None elements require no explicit fill. OwnedProtect guards
            // `out` across the fill and issues the matching UNPROTECT(1) on Drop.
            let out = OwnedProtect::new(Rf_allocVector(SEXPTYPE::VECSXP, n));
            for (i, opt) in self.into_iter().enumerate() {
                if let Some(list) = opt {
                    out.get().set_vector_elt(i as crate::R_xlen_t, list.0);
                }
            }
            *out
        }
    }
}

/// A `List` reads any R list, whatever its names.
///
/// Accepts a `VECSXP`, and a pairlist (`LISTSXP`) coerced to one; anything
/// else is a [`SexpTypeError`]. Names are not checked: R allows a name to
/// appear more than once (`list(a = 1, a = 2)`), and so does `List`. A
/// function that needs unique names checks with
/// [`List::first_duplicate_name`].
impl TryFromSexp for List {
    type Error = SexpTypeError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();

        // Accept VECSXP (generic list) directly
        // Also accept LISTSXP (pairlist) by coercing to VECSXP
        // Note: Rf_isList() only returns true for LISTSXP/NILSXP, not VECSXP
        if actual == VECSXP {
            Ok(List(sexp))
        } else if actual == LISTSXP {
            // Accept pairlists by coercing to a VECSXP list.
            Ok(List(sexp.coerce(VECSXP)))
        } else {
            Err(SexpTypeError {
                expected: VECSXP,
                actual,
            })
        }
    }
}

impl TryFromSexp for Option<List> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp == SEXP::nil() {
            return Ok(None);
        }
        Ok(Some(List::try_from_sexp(sexp)?))
    }
}

impl TryFromSexp for Option<ListMut> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp == SEXP::nil() {
            return Ok(None);
        }
        let list = ListMut::try_from_sexp(sexp)?;
        Ok(Some(list))
    }
}

impl TryFromSexp for ListMut {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
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
// endregion
