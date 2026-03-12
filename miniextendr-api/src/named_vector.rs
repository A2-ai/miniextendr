//! Named atomic vector wrapper for HashMap/BTreeMap ↔ named R atomic vector conversions.
//!
//! By default, `HashMap<String, V>` and `BTreeMap<String, V>` convert to/from named R
//! lists (VECSXP). This module provides [`NamedVector`] for converting to/from named
//! **atomic** vectors (INTSXP, REALSXP, LGLSXP, RAWSXP, STRSXP) instead — a more
//! compact and idiomatic representation when values are scalar.
//!
//! # Example
//!
//! ```ignore
//! use std::collections::HashMap;
//! use miniextendr_api::NamedVector;
//!
//! #[miniextendr]
//! fn make_scores() -> NamedVector<HashMap<String, i32>> {
//!     let mut m = HashMap::new();
//!     m.insert("alice".into(), 95);
//!     m.insert("bob".into(), 87);
//!     NamedVector(m)
//! }
//! // In R: make_scores() returns c(alice = 95L, bob = 87L)
//! ```

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::ffi::{self, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;

// region: AtomicElement trait

/// Marker trait for types that can be elements of named atomic R vectors.
///
/// Each implementation knows how to convert `Vec<Self>` to/from an R atomic
/// vector (INTSXP, REALSXP, LGLSXP, RAWSXP, or STRSXP).
pub trait AtomicElement: Sized {
    /// Convert a Rust vector to an R atomic SEXP.
    fn vec_to_sexp(values: Vec<Self>) -> SEXP;

    /// Convert an R atomic SEXP to a Rust vector.
    fn vec_from_sexp(sexp: SEXP) -> Result<Vec<Self>, SexpError>;
}

// --- Primitive numeric types (delegate to existing IntoR / TryFromSexp) ---

impl AtomicElement for i32 {
    fn vec_to_sexp(values: Vec<Self>) -> SEXP {
        values.into_sexp()
    }

    fn vec_from_sexp(sexp: SEXP) -> Result<Vec<Self>, SexpError> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::INTSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::INTSXP,
                actual,
            }
            .into());
        }
        let slice: &[i32] = TryFromSexp::try_from_sexp(sexp)?;
        Ok(slice.to_vec())
    }
}

impl AtomicElement for f64 {
    fn vec_to_sexp(values: Vec<Self>) -> SEXP {
        values.into_sexp()
    }

    fn vec_from_sexp(sexp: SEXP) -> Result<Vec<Self>, SexpError> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }
        let slice: &[f64] = TryFromSexp::try_from_sexp(sexp)?;
        Ok(slice.to_vec())
    }
}

impl AtomicElement for u8 {
    fn vec_to_sexp(values: Vec<Self>) -> SEXP {
        values.into_sexp()
    }

    fn vec_from_sexp(sexp: SEXP) -> Result<Vec<Self>, SexpError> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::RAWSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::RAWSXP,
                actual,
            }
            .into());
        }
        let slice: &[u8] = TryFromSexp::try_from_sexp(sexp)?;
        Ok(slice.to_vec())
    }
}

// --- Bool (non-NA) ---

impl AtomicElement for bool {
    fn vec_to_sexp(values: Vec<Self>) -> SEXP {
        values.into_sexp()
    }

    fn vec_from_sexp(sexp: SEXP) -> Result<Vec<Self>, SexpError> {
        <Vec<bool>>::try_from_sexp(sexp)
    }
}

// --- String (non-NA) ---

impl AtomicElement for String {
    fn vec_to_sexp(values: Vec<Self>) -> SEXP {
        values.into_sexp()
    }

    fn vec_from_sexp(sexp: SEXP) -> Result<Vec<Self>, SexpError> {
        <Vec<String>>::try_from_sexp(sexp)
    }
}

// --- Option<T> types (NA-aware) ---

impl AtomicElement for Option<i32> {
    fn vec_to_sexp(values: Vec<Self>) -> SEXP {
        values.into_sexp()
    }

    fn vec_from_sexp(sexp: SEXP) -> Result<Vec<Self>, SexpError> {
        <Vec<Option<i32>>>::try_from_sexp(sexp)
    }
}

impl AtomicElement for Option<f64> {
    fn vec_to_sexp(values: Vec<Self>) -> SEXP {
        values.into_sexp()
    }

    fn vec_from_sexp(sexp: SEXP) -> Result<Vec<Self>, SexpError> {
        <Vec<Option<f64>>>::try_from_sexp(sexp)
    }
}

impl AtomicElement for Option<bool> {
    fn vec_to_sexp(values: Vec<Self>) -> SEXP {
        values.into_sexp()
    }

    fn vec_from_sexp(sexp: SEXP) -> Result<Vec<Self>, SexpError> {
        <Vec<Option<bool>>>::try_from_sexp(sexp)
    }
}

impl AtomicElement for Option<String> {
    fn vec_to_sexp(values: Vec<Self>) -> SEXP {
        values.into_sexp()
    }

    fn vec_from_sexp(sexp: SEXP) -> Result<Vec<Self>, SexpError> {
        <Vec<Option<String>>>::try_from_sexp(sexp)
    }
}
// endregion

// region: NamedVector wrapper

/// Wrapper that converts a map to/from a **named atomic R vector** instead of a
/// named list.
///
/// The inner map must have `String` keys and values that implement [`AtomicElement`].
///
/// # Supported value types
///
/// | Rust type | R SEXPTYPE |
/// |-----------|-----------|
/// | `i32` | INTSXP |
/// | `f64` | REALSXP |
/// | `u8` | RAWSXP |
/// | `bool` | LGLSXP |
/// | `String` | STRSXP |
/// | `Option<i32>` | INTSXP (NA = NA_INTEGER) |
/// | `Option<f64>` | REALSXP (NA = NA_REAL) |
/// | `Option<bool>` | LGLSXP (NA = NA_LOGICAL) |
/// | `Option<String>` | STRSXP (NA = NA_character_) |
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedVector<M>(pub M);

impl<M> NamedVector<M> {
    /// Unwrap, returning the inner map.
    pub fn into_inner(self) -> M {
        self.0
    }
}

impl<M> From<M> for NamedVector<M> {
    fn from(m: M) -> Self {
        NamedVector(m)
    }
}
// endregion

// region: Helpers

/// Set names attribute on an R SEXP from a slice of name-like values.
///
/// # Safety
///
/// `sexp` must be a valid, protected SEXP. Caller must manage protect stack.
pub(crate) unsafe fn set_names_on_sexp<S: AsRef<str>>(sexp: SEXP, keys: &[S]) {
    unsafe {
        let n = keys.len();
        let names = ffi::Rf_allocVector(SEXPTYPE::STRSXP, n as ffi::R_xlen_t);
        ffi::Rf_protect(names);

        for (i, key) in keys.iter().enumerate() {
            let s = key.as_ref();
            let charsxp = ffi::Rf_mkCharLenCE(s.as_ptr().cast(), s.len() as i32, ffi::CE_UTF8);
            ffi::SET_STRING_ELT(names, i as ffi::R_xlen_t, charsxp);
        }

        ffi::Rf_setAttrib(sexp, ffi::R_NamesSymbol, names);
        ffi::Rf_unprotect(1);
    }
}

/// Extract names from an R SEXP with strict validation.
///
/// Errors on: missing names attribute, NA names, empty names, duplicate names.
fn extract_names_strict(sexp: SEXP) -> Result<Vec<String>, SexpError> {
    use ffi::{Rf_getAttrib, Rf_translateCharUTF8, STRING_ELT};

    let names = unsafe { Rf_getAttrib(sexp, ffi::R_NamesSymbol) };
    let len = sexp.len();

    if names.type_of() != SEXPTYPE::STRSXP || names.len() != len {
        return Err(SexpError::InvalidValue(
            "NamedVector requires a names attribute on the input vector".to_string(),
        ));
    }

    let mut result = Vec::with_capacity(len);
    let mut seen = HashSet::with_capacity(len);

    for i in 0..len {
        let charsxp = unsafe { STRING_ELT(names, i as ffi::R_xlen_t) };

        // Reject NA names
        if charsxp == unsafe { ffi::R_NaString } {
            return Err(SexpError::InvalidValue(
                "NamedVector does not allow NA names".to_string(),
            ));
        }

        let c_str = unsafe { Rf_translateCharUTF8(charsxp) };
        if c_str.is_null() {
            return Err(SexpError::InvalidValue(
                "NamedVector does not allow NA names".to_string(),
            ));
        }

        let name = unsafe { std::ffi::CStr::from_ptr(c_str) }
            .to_str()
            .unwrap_or("");

        // Reject empty names
        if name.is_empty() {
            return Err(SexpError::InvalidValue(
                "NamedVector does not allow empty names".to_string(),
            ));
        }

        // Reject duplicate names
        if !seen.insert(name.to_string()) {
            return Err(SexpError::DuplicateName(name.to_string()));
        }

        result.push(name.to_string());
    }

    Ok(result)
}
// endregion

// region: IntoR impls

impl<V: AtomicElement> IntoR for NamedVector<HashMap<String, V>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        let (keys, values): (Vec<String>, Vec<V>) = self.0.into_iter().unzip();
        let sexp = V::vec_to_sexp(values);
        unsafe {
            ffi::Rf_protect(sexp);
            set_names_on_sexp(sexp, &keys);
            ffi::Rf_unprotect(1);
        }
        sexp
    }
}

impl<V: AtomicElement> IntoR for NamedVector<BTreeMap<String, V>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        let (keys, values): (Vec<String>, Vec<V>) = self.0.into_iter().unzip();
        let sexp = V::vec_to_sexp(values);
        unsafe {
            ffi::Rf_protect(sexp);
            set_names_on_sexp(sexp, &keys);
            ffi::Rf_unprotect(1);
        }
        sexp
    }
}
// endregion

// region: TryFromSexp impls

impl<V: AtomicElement> TryFromSexp for NamedVector<HashMap<String, V>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let names = extract_names_strict(sexp)?;
        let values = V::vec_from_sexp(sexp)?;

        let mut map = HashMap::with_capacity(names.len());
        for (k, v) in names.into_iter().zip(values) {
            map.insert(k, v);
        }
        Ok(NamedVector(map))
    }
}

impl<V: AtomicElement> TryFromSexp for NamedVector<BTreeMap<String, V>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let names = extract_names_strict(sexp)?;
        let values = V::vec_from_sexp(sexp)?;

        let mut map = BTreeMap::new();
        for (k, v) in names.into_iter().zip(values) {
            map.insert(k, v);
        }
        Ok(NamedVector(map))
    }
}
// endregion
