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

use crate::from_r::{SexpError, SexpTypeError, TryFromSexp};
use crate::gc_protect::{OwnedProtect, ProtectScope};
use crate::into_r::IntoR;
use crate::{R_xlen_t, SEXP, SEXPTYPE, SexpExt};

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
        let scope = ProtectScope::new();
        let names = scope.alloc_strsxp(keys.len()).into_raw();

        for (i, key) in keys.iter().enumerate() {
            let s = key.as_ref();
            // Preserve the R_BlankString short-circuit: skips hash-lookup
            // on the interned empty CHARSXP for empty keys.
            let charsxp = if s.is_empty() {
                SEXP::blank_string()
            } else {
                SEXP::charsxp(s)
            };
            names.set_string_elt(i as R_xlen_t, charsxp);
        }

        sexp.set_names(names);
    }
}

/// Extract names from an R SEXP with strict validation.
///
/// Errors on: missing names attribute, NA names, empty names, duplicate names.
fn extract_names_strict(sexp: SEXP) -> Result<Vec<String>, SexpError> {
    use crate::from_r::{name_refusal, try_charsxp_to_str};

    let names = sexp.get_names();
    let len = sexp.len();

    if names.type_of() != SEXPTYPE::STRSXP || names.len() != len {
        return Err(SexpError::InvalidValue(
            "NamedVector requires a names attribute on the input vector".to_string(),
        ));
    }

    let mut result = Vec::with_capacity(len);
    let mut seen = HashSet::with_capacity(len);

    for i in 0..len {
        let charsxp = names.string_elt(i as R_xlen_t);

        // Reject NA names
        if charsxp == SEXP::na_string() {
            return Err(SexpError::InvalidValue(
                "NamedVector does not allow NA names".to_string(),
            ));
        }

        let name = unsafe { try_charsxp_to_str(charsxp) }.map_err(|e| name_refusal(i, &e))?;

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
    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        let (keys, values): (Vec<String>, Vec<V>) = self.0.into_iter().unzip();
        let sexp = V::vec_to_sexp(values);
        unsafe {
            let guard = OwnedProtect::new(sexp);
            set_names_on_sexp(guard.get(), &keys);
        }
        sexp
    }
}

impl<V: AtomicElement> IntoR for NamedVector<BTreeMap<String, V>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        let (keys, values): (Vec<String>, Vec<V>) = self.0.into_iter().unzip();
        let sexp = V::vec_to_sexp(values);
        unsafe {
            let guard = OwnedProtect::new(sexp);
            set_names_on_sexp(guard.get(), &keys);
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

// region: NamedVec

/// A vector's values and its names, in order, duplicate names included
/// (#1945).
///
/// [`NamedVector`] reads a named vector into a map, which needs unique names
/// and forgets their order. `NamedVec<T>` keeps the vector as it is, for a
/// vector whose order means something (the state, derivative and tolerance
/// order of an ODE system):
///
/// - **Values** convert exactly as `Vec<T>` converts the vector, so `NA`
///   follows `T`: `NamedVec<f64>` keeps `NA_real_`, `NamedVec<i32>` keeps
///   `NA_integer_` as `i32::MIN`, `NamedVec<Option<f64>>` reads it as `None`
///   and `NamedVec<bool>` refuses it. The R type must be the one `Vec<T>`
///   takes, so an integer vector is not a `NamedVec<f64>`. A named list reads
///   as `NamedVec<Vec<T>>` (each element a vector) or `NamedVec<List>`.
/// - **Names** are `None` without a `names` attribute, else one per value in
///   order. Duplicates are kept (check uniqueness yourself if you need it),
///   and an empty name is `""`. An `NA` name has no `String` form, so it is
///   refused, with every `NA` position in the message.
/// - **Refused:** a matrix or array (a `dim` attribute) and a classed object
///   (a factor, `Date`, `difftime`, ...), whose values would lose their
///   meaning as plain numbers. Pass `c(x)` or `unclass(x)` to read the values
///   anyway. Other attributes are ignored; [`NamedVec::sexp`] still has them.
/// - **`Option<NamedVec<T>>`** reads `NULL` as `None`.
/// - **Returned to R,** it is the values (as `Vec<T>` returns them) with the
///   names set, so a named vector read in comes back unchanged, minus any
///   other attributes.
///
/// ```ignore
/// #[miniextendr]
/// pub fn scaled(x: NamedVec<f64>, by: f64) -> NamedVec<f64> {
///     let (values, names) = x.into_parts();
///     NamedVec::new(values.into_iter().map(|v| v * by).collect(), names)
/// }
/// // In R: scaled(c(y = 1, dy = 2), 10) is c(y = 10, dy = 20)
/// ```
#[derive(Debug, Clone)]
pub struct NamedVec<T> {
    values: Vec<T>,
    names: Option<Vec<String>>,
    sexp: Option<SEXP>,
}

impl<T> NamedVec<T> {
    /// Values with names, or without (`None`), to return to R.
    ///
    /// # Panics
    ///
    /// When `names` is `Some` and its length differs from `values`'.
    pub fn new(values: Vec<T>, names: Option<Vec<String>>) -> Self {
        if let Some(names) = &names {
            assert_eq!(
                names.len(),
                values.len(),
                "NamedVec::new: {} names for {} values",
                names.len(),
                values.len()
            );
        }
        NamedVec {
            values,
            names,
            sexp: None,
        }
    }

    /// The values, in order.
    pub fn values(&self) -> &[T] {
        &self.values
    }

    /// The names, one per value in order, or `None` when the vector had no
    /// `names` attribute.
    pub fn names(&self) -> Option<&[String]> {
        self.names.as_deref()
    }

    /// The R vector this was read from, `None` for one made with
    /// [`NamedVec::new`]. It is the argument of the call that received it:
    /// valid for that call and not rooted, so don't keep it past the call.
    pub fn sexp(&self) -> Option<SEXP> {
        self.sexp
    }

    /// The values and the names.
    pub fn into_parts(self) -> (Vec<T>, Option<Vec<String>>) {
        (self.values, self.names)
    }
}

/// Refuse what [`NamedVec`] does not read: a `dim` attribute (a matrix or an
/// array) and a class attribute.
fn refuse_shaped_or_classed(sexp: SEXP) -> Result<(), SexpError> {
    if !sexp.get_dim().is_nil() {
        return Err(SexpError::InvalidValue(
            "got a matrix or array (it has a dim attribute); c() drops it".to_string(),
        ));
    }
    let class = sexp.get_class();
    if !class.is_nil() {
        let class = <Vec<String>>::try_from_sexp(class)?;
        let class = class
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(SexpError::InvalidValue(format!(
            "got an object of class {class}; unclass() drops it"
        )));
    }
    Ok(())
}

/// The vector's names for [`NamedVec`]: `None` without the attribute, an
/// error naming every `NA` position.
fn names_in_order(sexp: SEXP) -> Result<Option<Vec<String>>, SexpError> {
    let names = sexp.get_names();
    if names.is_nil() {
        return Ok(None);
    }
    let names = <Vec<Option<String>>>::try_from_sexp(names)?;
    let na: Vec<String> = names
        .iter()
        .enumerate()
        .filter(|(_, name)| name.is_none())
        .map(|(i, _)| (i + 1).to_string())
        .collect();
    if !na.is_empty() {
        return Err(SexpError::InvalidValue(format!(
            "NA names at position{} {}",
            if na.len() == 1 { "" } else { "s" },
            na.join(", ")
        )));
    }
    Ok(Some(names.into_iter().flatten().collect()))
}

impl<T> TryFromSexp for NamedVec<T>
where
    Vec<T>: TryFromSexp<Error = SexpError>,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        refuse_shaped_or_classed(sexp)?;
        let values = <Vec<T>>::try_from_sexp(sexp)?;
        let names = names_in_order(sexp)?;
        Ok(NamedVec {
            values,
            names,
            sexp: Some(sexp),
        })
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        refuse_shaped_or_classed(sexp)?;
        let values = unsafe { <Vec<T>>::try_from_sexp_unchecked(sexp) }?;
        let names = names_in_order(sexp)?;
        Ok(NamedVec {
            values,
            names,
            sexp: Some(sexp),
        })
    }
}

impl<T> TryFromSexp for Option<NamedVec<T>>
where
    Vec<T>: TryFromSexp<Error = SexpError>,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.is_nil() {
            return Ok(None);
        }
        <NamedVec<T>>::try_from_sexp(sexp).map(Some)
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.is_nil() {
            return Ok(None);
        }
        unsafe { <NamedVec<T>>::try_from_sexp_unchecked(sexp) }.map(Some)
    }
}

impl<T> IntoR for NamedVec<T>
where
    Vec<T>: IntoR,
{
    type Error = <Vec<T> as IntoR>::Error;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        let sexp = self.values.try_into_sexp()?;
        if let Some(names) = &self.names {
            unsafe {
                let guard = OwnedProtect::new(sexp);
                set_names_on_sexp(guard.get(), names);
            }
        }
        Ok(sexp)
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        let sexp = unsafe { self.values.try_into_sexp_unchecked() }?;
        if let Some(names) = &self.names {
            unsafe {
                let guard = OwnedProtect::new(sexp);
                set_names_on_sexp(guard.get(), names);
            }
        }
        Ok(sexp)
    }
}
// endregion
