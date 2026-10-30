//! `RValue` — an owned, `Send`, R-native value tree.
//!
//! The `serde_json::Value` analogue for R's base data types: an owned (no live
//! `SEXP`), inspectable (`Debug + Clone`), `Send` (a `panic_any` payload can
//! carry it) representation of "an arbitrary R native value". Use it wherever
//! you need to carry a dynamic/heterogeneous R value through Rust — e.g.
//! condition `data =` payloads, which travel through `panic_any` before any R
//! object exists.
//!
//! Tuples with 1–12 fields convert into unnamed [`RValue::List`] entries,
//! recursively using each field's `Into<RValue>` implementation. This lets
//! condition data carry a tuple without allocating in R or requiring `Debug`.
//!
//! The variants enumerate R's (finite, closed) data type system — vectors only,
//! since R has no true scalars. NA is modelled with `Option` where R carries it
//! out of band (logical/integer/character); `Double` carries NA in the `NA_REAL`
//! bit pattern inside `Vec<f64>` (no `Option`), matching R's in-band convention.
//!
//! Out of scope (existing homes): closures/environments/symbols/language objects
//! (not data), S4 objects, `EXTPTRSXP` (use [`crate::externalptr::ExternalPtr`]),
//! ALTREP internals. Attribute-carrying values (factor, `Date`, `POSIXct`) are
//! modelled as their plain `Integer`/`Double` storage in v1 — richer handling is
//! deferred until a consumer needs it.

use crate::SexpExt;
use crate::from_r::{SexpError, SexpLengthError, SexpNaError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;
use crate::sexp_types::{Rcomplex, SEXPTYPE};
use crate::{IsNA, SEXP};

/// An owned, `Send`, R-native value tree. See the [module docs](self).
#[derive(Debug, Clone)]
pub enum RValue {
    /// `NULL` (`NILSXP`).
    Null,
    /// Logical vector (`LGLSXP`). `None` is `NA`.
    Logical(Vec<Option<bool>>),
    /// Integer vector (`INTSXP`). `None` is `NA`.
    Integer(Vec<Option<i32>>),
    /// Double vector (`REALSXP`). NA is carried in the `NA_REAL` bit pattern.
    Double(Vec<f64>),
    /// Complex vector (`CPLXSXP`).
    Complex(Vec<Rcomplex>),
    /// Character vector (`STRSXP`). `None` is `NA`.
    Character(Vec<Option<String>>),
    /// Raw byte vector (`RAWSXP`). No NA.
    Raw(Vec<u8>),
    /// Generic list (`VECSXP`). Recursive; a `None` name is an unnamed slot.
    List(Vec<(Option<String>, RValue)>),
}

// region: IntoR

impl IntoR for RValue {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }

    fn into_sexp(self) -> SEXP {
        match self {
            RValue::Null => SEXP::nil(),
            RValue::Logical(v) => v.into_sexp(),
            RValue::Integer(v) => v.into_sexp(),
            RValue::Double(v) => v.into_sexp(),
            RValue::Complex(v) => v.into_sexp(),
            RValue::Character(v) => v.into_sexp(),
            RValue::Raw(v) => v.into_sexp(),
            RValue::List(pairs) => list_into_sexp(pairs),
        }
    }
}

/// Build a `VECSXP` from list pairs, recursing into each child.
///
/// Each child `SEXP` is `protect_raw`-rooted before the next child allocates —
/// the same GC discipline as `AsNamedList::into_sexp` (#1030/#1045), required
/// under `gctorture`. The scope drops once the list is built.
fn list_into_sexp(pairs: Vec<(Option<String>, RValue)>) -> SEXP {
    use crate::list::List;
    // SAFETY: `IntoR::into_sexp` for `#[miniextendr]` return values runs on the R
    // main thread.
    unsafe {
        let scope = crate::gc_protect::ProtectScope::new();
        if pairs.iter().any(|(name, _)| name.is_some()) {
            // Named (R fills unnamed slots with ""). `from_raw_pairs` sets names.
            let named: Vec<(String, SEXP)> = pairs
                .into_iter()
                .map(|(name, val)| (name.unwrap_or_default(), scope.protect_raw(val.into_sexp())))
                .collect();
            List::from_raw_pairs(named).into_sexp()
        } else {
            let values: Vec<SEXP> = pairs
                .into_iter()
                .map(|(_, val)| scope.protect_raw(val.into_sexp()))
                .collect();
            List::from_raw_values(values).into_sexp()
        }
    }
}

// endregion

// region: TryFromSexp

impl TryFromSexp for RValue {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let n = sexp.len();
        match sexp.type_of() {
            SEXPTYPE::NILSXP => Ok(RValue::Null),
            SEXPTYPE::LGLSXP => Ok(RValue::Logical(
                (0..n as isize)
                    .map(|i| {
                        let x = sexp.logical_elt(i);
                        (!x.is_na()).then_some(x != 0)
                    })
                    .collect(),
            )),
            SEXPTYPE::INTSXP => Ok(RValue::Integer(
                (0..n as isize)
                    .map(|i| {
                        let x = sexp.integer_elt(i);
                        (!x.is_na()).then_some(x)
                    })
                    .collect(),
            )),
            // NA stays in the bit pattern — no Option needed (decision 2).
            SEXPTYPE::REALSXP => Ok(RValue::Double(
                (0..n as isize).map(|i| sexp.real_elt(i)).collect(),
            )),
            SEXPTYPE::CPLXSXP => Ok(RValue::Complex(
                (0..n as isize).map(|i| sexp.complex_elt(i)).collect(),
            )),
            SEXPTYPE::STRSXP => Ok(RValue::Character(
                (0..n as isize)
                    .map(|i| {
                        if sexp.string_elt(i).is_na_string() {
                            None
                        } else {
                            sexp.string_elt_str(i).map(str::to_string)
                        }
                    })
                    .collect(),
            )),
            // SAFETY: type checked to be RAWSXP; `as_slice` handles the empty (0x1) case.
            SEXPTYPE::RAWSXP => Ok(RValue::Raw(unsafe { sexp.as_slice::<u8>() }.to_vec())),
            SEXPTYPE::VECSXP => {
                let names = sexp.get_names();
                let mut pairs = Vec::with_capacity(n);
                for i in 0..n as isize {
                    // `||` short-circuits, so `string_elt` is only reached for a
                    // character names vector. NA / empty names are unnamed slots.
                    let name = if names.is_nil()
                        || !names.is_character()
                        || names.string_elt(i).is_na_string()
                    {
                        None
                    } else {
                        names
                            .string_elt_str(i)
                            .filter(|s| !s.is_empty())
                            .map(str::to_string)
                    };
                    pairs.push((name, RValue::try_from_sexp(sexp.vector_elt(i))?));
                }
                Ok(RValue::List(pairs))
            }
            other => Err(SexpError::InvalidValue(format!(
                "RValue cannot represent SEXP of type {other:?} (only R data vectors are modelled)"
            ))),
        }
    }
}

// endregion

// region: ergonomic From impls (scalars wrap to length-1 vectors, per R semantics)

impl From<i32> for RValue {
    fn from(v: i32) -> Self {
        RValue::Integer(vec![Some(v)])
    }
}
impl From<f64> for RValue {
    fn from(v: f64) -> Self {
        RValue::Double(vec![v])
    }
}
impl From<bool> for RValue {
    fn from(v: bool) -> Self {
        RValue::Logical(vec![Some(v)])
    }
}
impl From<String> for RValue {
    fn from(v: String) -> Self {
        RValue::Character(vec![Some(v)])
    }
}
impl From<&str> for RValue {
    fn from(v: &str) -> Self {
        RValue::Character(vec![Some(v.to_string())])
    }
}
impl From<Vec<i32>> for RValue {
    fn from(v: Vec<i32>) -> Self {
        RValue::Integer(v.into_iter().map(Some).collect())
    }
}
impl From<Vec<f64>> for RValue {
    fn from(v: Vec<f64>) -> Self {
        RValue::Double(v)
    }
}
impl From<Vec<bool>> for RValue {
    fn from(v: Vec<bool>) -> Self {
        RValue::Logical(v.into_iter().map(Some).collect())
    }
}
impl From<Vec<String>> for RValue {
    fn from(v: Vec<String>) -> Self {
        RValue::Character(v.into_iter().map(Some).collect())
    }
}
impl From<Vec<&str>> for RValue {
    fn from(v: Vec<&str>) -> Self {
        RValue::Character(v.into_iter().map(|s| Some(s.to_string())).collect())
    }
}

// endregion

// region: Tuples (owned positional lists, no R allocation)

macro_rules! impl_tuple_rvalue {
    ($n:literal, $name:literal; $($T:ident $R:ident $idx:tt),+) => {
        impl<$($T: Into<RValue>),+> From<($($T,)+)> for RValue {
            fn from(value: ($($T,)+)) -> Self {
                Self::List(vec![$((None, value.$idx.into()),)+])
            }
        }
    };
}

crate::tuple_macros::for_each_tuple!(impl_tuple_rvalue);

// endregion

// region: NA-aware Option / Vec<Option> + wide-integer ladder (ported from #1044/#995)

// `Option<T>` → length-1 vector, `None` → `NA`. Integer/logical/character carry
// NA out of band via the `None` slot; `Double` carries it in the `NA_REAL` bit
// pattern (no `Option<f64>` variant — see the module docs).

impl From<Option<i32>> for RValue {
    fn from(v: Option<i32>) -> Self {
        RValue::Integer(vec![v])
    }
}
impl From<Option<f64>> for RValue {
    fn from(v: Option<f64>) -> Self {
        RValue::Double(vec![v.unwrap_or(crate::altrep_traits::NA_REAL)])
    }
}
impl From<Option<bool>> for RValue {
    fn from(v: Option<bool>) -> Self {
        RValue::Logical(vec![v])
    }
}
impl From<Option<String>> for RValue {
    fn from(v: Option<String>) -> Self {
        RValue::Character(vec![v])
    }
}
impl From<Option<&str>> for RValue {
    fn from(v: Option<&str>) -> Self {
        RValue::Character(vec![v.map(str::to_string)])
    }
}
impl From<Vec<Option<i32>>> for RValue {
    fn from(v: Vec<Option<i32>>) -> Self {
        RValue::Integer(v)
    }
}
impl From<Vec<Option<f64>>> for RValue {
    fn from(v: Vec<Option<f64>>) -> Self {
        RValue::Double(
            v.into_iter()
                .map(|x| x.unwrap_or(crate::altrep_traits::NA_REAL))
                .collect(),
        )
    }
}
impl From<Vec<Option<bool>>> for RValue {
    fn from(v: Vec<Option<bool>>) -> Self {
        RValue::Logical(v)
    }
}
impl From<Vec<Option<String>>> for RValue {
    fn from(v: Vec<Option<String>>) -> Self {
        RValue::Character(v)
    }
}
impl From<Vec<Option<&str>>> for RValue {
    fn from(v: Vec<Option<&str>>) -> Self {
        RValue::Character(v.into_iter().map(|s| s.map(str::to_string)).collect())
    }
}

/// Wide-integer ladder: an `i64` (or `u32`) that fits in `i32` **and** is not
/// `i32::MIN` (R's `NA_integer_`) becomes an `Integer`; anything else becomes a
/// `Double`. Mirrors R's own integer→double promotion for out-of-range values.
impl From<i64> for RValue {
    fn from(v: i64) -> Self {
        match i32::try_from(v) {
            Ok(n) if !n.is_na() => RValue::Integer(vec![Some(n)]),
            // Out of `i32` range (or exactly `NA_integer_`) — promote to double.
            // `i64 → f64` is lossy past 2^53, matching R's own integer overflow.
            _ => RValue::Double(vec![v as f64]),
        }
    }
}
impl From<u32> for RValue {
    /// Lossless widening to `i64`, then the shared ladder. A `u32` is never
    /// `i32::MIN` as an `i32`, so only the `> i32::MAX` case falls to `Double`.
    fn from(v: u32) -> Self {
        RValue::from(i64::from(v))
    }
}

// endregion

// region: inspection accessors + owned extraction (#1050 decision 4, on demand)

impl RValue {
    /// The `SEXPTYPE` this value materialises to via [`IntoR`].
    pub fn sexptype(&self) -> SEXPTYPE {
        match self {
            RValue::Null => SEXPTYPE::NILSXP,
            RValue::Logical(_) => SEXPTYPE::LGLSXP,
            RValue::Integer(_) => SEXPTYPE::INTSXP,
            RValue::Double(_) => SEXPTYPE::REALSXP,
            RValue::Complex(_) => SEXPTYPE::CPLXSXP,
            RValue::Character(_) => SEXPTYPE::STRSXP,
            RValue::Raw(_) => SEXPTYPE::RAWSXP,
            RValue::List(_) => SEXPTYPE::VECSXP,
        }
    }

    /// `true` if this is [`RValue::Null`].
    pub fn is_null(&self) -> bool {
        matches!(self, RValue::Null)
    }

    /// Wrap any `T: Debug` as a length-1 `Character` carrying its `{:?}`
    /// rendering — the escape hatch for a value with no R-native mapping
    /// (e.g. a Rust range). The rendering happens eagerly here, so nothing
    /// borrows across the `Send` boundary a condition payload crosses.
    ///
    /// ```ignore
    /// # use miniextendr_api::RValue;
    /// assert_eq!(RValue::debug(0..=100).as_str(), Some("0..=100"));
    /// ```
    pub fn debug<T: std::fmt::Debug>(value: T) -> Self {
        RValue::Character(vec![Some(format!("{value:?}"))])
    }

    /// The single non-NA `i32` of a length-1 `Integer`, else `None`.
    ///
    /// Returns `None` for the wrong variant, a length other than 1, or NA —
    /// the borrow-friendly counterpart to `i32::try_from` when you don't need
    /// to know *why* it failed.
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            RValue::Integer(v) if v.len() == 1 => v[0],
            _ => None,
        }
    }

    /// The single non-NA `f64` of a length-1 `Double`, else `None`.
    ///
    /// R's NA yields `None`, matching [`as_i32`](Self::as_i32). NA is decided by
    /// R's `R_IsNA` rule, so a computed NA (`NA_real_ * 1`) counts; any other NaN
    /// is returned as a value.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            RValue::Double(v) if v.len() == 1 => {
                let x = v[0];
                (!x.is_na()).then_some(x)
            }
            _ => None,
        }
    }

    /// The single non-NA `bool` of a length-1 `Logical`, else `None`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            RValue::Logical(v) if v.len() == 1 => v[0],
            _ => None,
        }
    }

    /// The single non-NA string of a length-1 `Character`, else `None`.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            RValue::Character(v) if v.len() == 1 => v[0].as_deref(),
            _ => None,
        }
    }
}

/// `Type` error for a variant mismatch: the value's `SEXPTYPE` vs the expected one.
fn wrong_type(expected: SEXPTYPE, actual: &RValue) -> SexpError {
    SexpError::Type(SexpTypeError {
        expected,
        actual: actual.sexptype(),
    })
}

/// `TryFrom<RValue>` for a scalar pulled from a length-1 Option-backed variant.
///
/// Fails with `Length` for a non-scalar, `Na` for NA, `Type` for the wrong
/// variant — the failure modes a numeric `CoerceError` can't express, which is
/// why this is `TryFrom`/`SexpError` rather than a `coerce.rs` impl.
macro_rules! try_from_scalar_opt {
    ($t:ty, $variant:ident, $ty:expr) => {
        impl TryFrom<RValue> for $t {
            type Error = SexpError;
            fn try_from(value: RValue) -> Result<Self, SexpError> {
                match value {
                    RValue::$variant(xs) => {
                        if xs.len() != 1 {
                            return Err(SexpError::Length(SexpLengthError {
                                expected: 1,
                                actual: xs.len(),
                            }));
                        }
                        xs.into_iter()
                            .next()
                            .unwrap()
                            .ok_or(SexpError::Na(SexpNaError { sexp_type: $ty }))
                    }
                    other => Err(wrong_type($ty, &other)),
                }
            }
        }
    };
}
try_from_scalar_opt!(i32, Integer, SEXPTYPE::INTSXP);
try_from_scalar_opt!(bool, Logical, SEXPTYPE::LGLSXP);
try_from_scalar_opt!(String, Character, SEXPTYPE::STRSXP);

// `Double` carries NA in the bit pattern, so it can't reuse the Option macro.
impl TryFrom<RValue> for f64 {
    type Error = SexpError;
    fn try_from(value: RValue) -> Result<Self, SexpError> {
        match value {
            RValue::Double(xs) => {
                if xs.len() != 1 {
                    return Err(SexpError::Length(SexpLengthError {
                        expected: 1,
                        actual: xs.len(),
                    }));
                }
                let x = xs[0];
                // `R_IsNA`: a computed NA is an NA error too; other NaNs pass.
                if x.is_na() {
                    Err(SexpError::Na(SexpNaError {
                        sexp_type: SEXPTYPE::REALSXP,
                    }))
                } else {
                    Ok(x)
                }
            }
            other => Err(wrong_type(SEXPTYPE::REALSXP, &other)),
        }
    }
}

/// `TryFrom<RValue>` for the whole vector of a single variant (NA preserved).
macro_rules! try_from_vec {
    ($t:ty, $variant:ident, $ty:expr) => {
        impl TryFrom<RValue> for $t {
            type Error = SexpError;
            fn try_from(value: RValue) -> Result<Self, SexpError> {
                match value {
                    RValue::$variant(xs) => Ok(xs),
                    other => Err(wrong_type($ty, &other)),
                }
            }
        }
    };
}
try_from_vec!(Vec<Option<bool>>, Logical, SEXPTYPE::LGLSXP);
try_from_vec!(Vec<Option<i32>>, Integer, SEXPTYPE::INTSXP);
try_from_vec!(Vec<f64>, Double, SEXPTYPE::REALSXP);
try_from_vec!(Vec<Rcomplex>, Complex, SEXPTYPE::CPLXSXP);
try_from_vec!(Vec<Option<String>>, Character, SEXPTYPE::STRSXP);
try_from_vec!(Vec<u8>, Raw, SEXPTYPE::RAWSXP);
try_from_vec!(Vec<(Option<String>, RValue)>, List, SEXPTYPE::VECSXP);

// endregion

#[cfg(test)]
mod tests;
