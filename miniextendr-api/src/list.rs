//! Thin wrapper around R list (`VECSXP`).
//! Provides safe construction from Rust values and typed extraction.

use crate::ffi::SEXPTYPE::{LISTSXP, STRSXP, VECSXP};
use crate::ffi::{self, Rboolean, SEXP};
use crate::from_r::{SexpError, SexpLengthError, TryFromSexp};
use crate::into_r::IntoR;

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

    /// Read the `names` attribute if present.
    #[inline]
    pub fn names(self) -> Option<SEXP> {
        unsafe {
            let names = ffi::Rf_getAttrib(self.0, ffi::R_NamesSymbol);
            if names == ffi::R_NilValue {
                None
            } else {
                Some(names)
            }
        }
    }

    /// Set the `names` attribute; returns the same list for chaining.
    #[inline]
    pub fn set_names(self, names: SEXP) -> Self {
        unsafe { ffi::Rf_namesgets(self.0, names) };
        self
    }

    /// Set the `dim` attribute; returns the same list for chaining.
    #[inline]
    pub fn set_dim(self, dim: SEXP) -> Self {
        unsafe { ffi::Rf_dimgets(self.0, dim) };
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

impl TryFromSexp for List {
    type Error = crate::from_r::SexpTypeError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // Use R's helper to ensure list semantics first
        let is_list = unsafe { ffi::Rf_isList(sexp) != Rboolean::FALSE };
        let actual = unsafe { ffi::TYPEOF(sexp) };

        if is_list {
            if actual == VECSXP {
                return Ok(List(sexp));
            }
            // Accept pairlists by coercing to a VECSXP list.
            if actual == LISTSXP {
                let coerced = unsafe { ffi::Rf_coerceVector(sexp, VECSXP) };
                return Ok(List(coerced));
            }
        }

        Err(crate::from_r::SexpTypeError {
            expected: VECSXP,
            actual,
        })
    }
}
