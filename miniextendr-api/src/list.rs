//! Thin wrapper around R list (`VECSXP`).
//! Provides safe construction from Rust values and typed extraction.

use crate::ffi::SEXPTYPE::{LISTSXP, STRSXP, VECSXP};
use crate::ffi::{self, Rboolean, SEXP};
use crate::from_r::{SexpError, SexpLengthError, TryFromSexp};
use crate::into_r::IntoR;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

/// Owned handle to an R list (`VECSXP`).
#[derive(Clone, Copy, Debug)]
pub struct List(SEXP);

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
        let converted: Vec<SEXP> = self.into_iter().map(|v| v.into_sexp()).collect();
        let n = converted.len() as isize;
        unsafe {
            let list = ffi::Rf_allocVector(VECSXP, n);
            // PROTECT not required: Rf_allocVector returns protected until we return to R.
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
    pub fn from_raw_values(values: Vec<SEXP>) -> Self {
        let n = values.len() as isize;
        unsafe {
            let list = ffi::Rf_allocVector(VECSXP, n);
            for (i, val) in values.into_iter().enumerate() {
                ffi::SET_VECTOR_ELT(list, i as isize, val);
            }
            List(list)
        }
    }

    /// Build a list from `(name, SEXP)` pairs (heterogeneous-friendly).
    pub fn from_raw_pairs<N>(pairs: Vec<(N, SEXP)>) -> Self
    where
        N: AsRef<str>,
    {
        let n = pairs.len() as isize;
        unsafe {
            let list = ffi::Rf_allocVector(VECSXP, n);
            let names = ffi::Rf_allocVector(STRSXP, n);
            for (i, (name, val)) in pairs.into_iter().enumerate() {
                ffi::SET_VECTOR_ELT(list, i as isize, val);

                let s = name.as_ref();
                let chars = ffi::Rf_mkCharLenCE(s.as_ptr().cast(), s.len() as i32, ffi::CE_UTF8);
                ffi::SET_STRING_ELT(names, i as isize, chars);
            }
            ffi::Rf_namesgets(list, names);
            List(list)
        }
    }
}

impl IntoR for List {
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
