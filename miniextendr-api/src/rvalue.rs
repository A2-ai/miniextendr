//! `RValue` — an owned, `Send`, R-native value tree.
//!
//! The `serde_json::Value` analogue for R's base data types: an owned (no live
//! `SEXP`), inspectable (`Debug + Clone`), `Send` (crosses the worker boundary)
//! representation of "an arbitrary R native value". Use it wherever you need to
//! carry a dynamic/heterogeneous R value through Rust — e.g. condition `data =`
//! payloads, which travel through `panic_any` and may be raised on the worker
//! thread where a live `SEXP` would be illegal.
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

use crate::SEXP;
use crate::SexpExt;
use crate::from_r::{SexpError, TryFromSexp};
use crate::into_r::IntoR;
use crate::sexp_types::{Rcomplex, SEXPTYPE};

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
        use crate::altrep_traits::{NA_INTEGER, NA_LOGICAL};

        let n = sexp.len();
        match sexp.type_of() {
            SEXPTYPE::NILSXP => Ok(RValue::Null),
            SEXPTYPE::LGLSXP => Ok(RValue::Logical(
                (0..n as isize)
                    .map(|i| {
                        let x = sexp.logical_elt(i);
                        (x != NA_LOGICAL).then_some(x != 0)
                    })
                    .collect(),
            )),
            SEXPTYPE::INTSXP => Ok(RValue::Integer(
                (0..n as isize)
                    .map(|i| {
                        let x = sexp.integer_elt(i);
                        (x != NA_INTEGER).then_some(x)
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
