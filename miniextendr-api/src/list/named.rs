//! `NamedList` — O(1) name-indexed access to R lists.
//!
//! Wraps a [`List`](super::List) and builds a `HashMap<String, usize>` index
//! on construction. Use when accessing multiple elements by name from the
//! same list — each lookup is O(1) instead of O(n).

use std::collections::HashMap;

use crate::ffi::{self, SEXP};
use crate::from_r::{SexpError, TryFromSexp};
use crate::into_r::IntoR;

use super::List;

/// A named list with O(1) name-based element lookup.
///
/// Wraps a [`List`] and builds a `HashMap<String, usize>` index of element names
/// on construction. Use this when you need to access multiple elements by name
/// from the same list — each lookup is O(1) instead of O(n).
///
/// # When to Use
///
/// | Pattern | Type |
/// |---------|------|
/// | Single named lookup | [`List::get_named`] is fine |
/// | Multiple named lookups | `NamedList` (O(n) build + O(1) per lookup) |
/// | Positional access only | [`List`] — no indexing overhead |
///
/// # Name Handling
///
/// - `NA` and empty-string names are excluded from the index
/// - If duplicate names exist, the **last** occurrence wins
/// - Positional access via [`get_index`](Self::get_index) is always available
pub struct NamedList {
    list: List,
    index: HashMap<String, usize>,
}

impl NamedList {
    /// Build a `NamedList` from a `List`, indexing all non-empty, non-NA names.
    ///
    /// Returns `None` if the list has no `names` attribute.
    pub fn new(list: List) -> Option<Self> {
        let names_sexp = list.names()?;
        let n: usize = list
            .len()
            .try_into()
            .expect("list length must be non-negative");
        let mut index = HashMap::with_capacity(n);

        for i in 0..n {
            let idx: isize = i.try_into().expect("index exceeds isize::MAX");
            let name_sexp = unsafe { ffi::STRING_ELT(names_sexp, idx) };
            if name_sexp == unsafe { ffi::R_NaString } {
                continue;
            }
            let name_ptr = unsafe { ffi::R_CHAR(name_sexp) };
            let name_cstr = unsafe { std::ffi::CStr::from_ptr(name_ptr) };
            if let Ok(s) = name_cstr.to_str() {
                if !s.is_empty() {
                    index.insert(s.to_owned(), i);
                }
            }
        }

        Some(NamedList { list, index })
    }

    /// Get an element by name with O(1) lookup, converting to type `T`.
    ///
    /// Returns `None` if the name is not found or conversion fails.
    #[inline]
    pub fn get<T>(&self, name: &str) -> Option<T>
    where
        T: TryFromSexp<Error = SexpError>,
    {
        let &idx = self.index.get(name)?;
        let idx_isize: isize = idx.try_into().ok()?;
        let elem = unsafe { ffi::VECTOR_ELT(self.list.as_sexp(), idx_isize) };
        T::try_from_sexp(elem).ok()
    }

    /// Get a raw SEXP element by name with O(1) lookup.
    #[inline]
    pub fn get_raw(&self, name: &str) -> Option<SEXP> {
        let &idx = self.index.get(name)?;
        let idx_isize: isize = idx.try_into().ok()?;
        Some(unsafe { ffi::VECTOR_ELT(self.list.as_sexp(), idx_isize) })
    }

    /// Get element at 0-based index and convert to type `T`.
    ///
    /// Falls through to [`List::get_index`] — no name lookup involved.
    #[inline]
    pub fn get_index<T>(&self, idx: isize) -> Option<T>
    where
        T: TryFromSexp<Error = SexpError>,
    {
        self.list.get_index(idx)
    }

    /// Check if a name exists in the index.
    #[inline]
    pub fn contains(&self, name: &str) -> bool {
        self.index.contains_key(name)
    }

    /// Number of elements in the list (including unnamed ones).
    #[inline]
    pub fn len(&self) -> isize {
        self.list.len()
    }

    /// Returns `true` if the list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Number of indexed (named) elements.
    #[inline]
    pub fn named_len(&self) -> usize {
        self.index.len()
    }

    /// Get the underlying `List`.
    #[inline]
    pub fn as_list(&self) -> List {
        self.list
    }

    /// Consume and return the underlying `List`.
    #[inline]
    pub fn into_list(self) -> List {
        self.list
    }

    /// Iterate over indexed names (unordered).
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.index.keys().map(|s| s.as_str())
    }

    /// Iterate over `(name, position)` pairs (unordered).
    pub fn entries(&self) -> impl Iterator<Item = (&str, usize)> {
        self.index.iter().map(|(k, &v)| (k.as_str(), v))
    }
}

impl IntoR for NamedList {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.list.into_sexp()
    }
}

impl TryFromSexp for NamedList {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let list = List::try_from_sexp(sexp).map_err(|e| SexpError::InvalidValue(e.to_string()))?;
        NamedList::new(list)
            .ok_or_else(|| SexpError::InvalidValue("list has no names attribute".into()))
    }
}

impl TryFromSexp for Option<NamedList> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp == SEXP::null() {
            return Ok(None);
        }
        let named = NamedList::try_from_sexp(sexp)?;
        Ok(Some(named))
    }
}

// IntoList and TryFromList traits are defined in the parent list.rs module.
