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
use crate::from_r::{SexpError, SexpLengthError, SexpNaError, SexpTypeError, TryFromSexp};
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
            Ok(n) if n != i32::MIN => RValue::Integer(vec![Some(n)]),
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
    /// R's NA (the `NA_REAL` bit pattern) yields `None`, matching [`as_i32`](Self::as_i32).
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            RValue::Double(v) if v.len() == 1 => {
                let x = v[0];
                (x.to_bits() != crate::altrep_traits::NA_REAL.to_bits()).then_some(x)
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
                if x.to_bits() == crate::altrep_traits::NA_REAL.to_bits() {
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
mod tests {
    use super::*;

    // Accessors and TryFrom operate on owned `RValue` — no SEXP, no R runtime.

    #[test]
    fn as_scalar_accessors() {
        assert_eq!(RValue::from(7i32).as_i32(), Some(7));
        assert_eq!(RValue::from(1.5).as_f64(), Some(1.5));
        assert_eq!(RValue::from(true).as_bool(), Some(true));
        assert_eq!(RValue::from("hi").as_str(), Some("hi"));

        // wrong variant
        assert_eq!(RValue::from("hi").as_i32(), None);
        // wrong length
        assert_eq!(RValue::Integer(vec![Some(1), Some(2)]).as_i32(), None);
        // NA
        assert_eq!(RValue::Integer(vec![None]).as_i32(), None);
        assert_eq!(
            RValue::Double(vec![crate::altrep_traits::NA_REAL]).as_f64(),
            None
        );
        assert!(RValue::Null.is_null());
    }

    #[test]
    fn try_from_scalar_ok_and_errors() {
        assert_eq!(i32::try_from(RValue::from(7i32)).unwrap(), 7);
        assert_eq!(f64::try_from(RValue::from(2.5)).unwrap(), 2.5);
        assert!(!bool::try_from(RValue::from(false)).unwrap());
        assert_eq!(String::try_from(RValue::from("x")).unwrap(), "x");

        // wrong variant → Type
        assert!(matches!(
            i32::try_from(RValue::from("x")),
            Err(SexpError::Type(_))
        ));
        // wrong length → Length
        assert!(matches!(
            i32::try_from(RValue::Integer(vec![Some(1), Some(2)])),
            Err(SexpError::Length(_))
        ));
        // NA → Na
        assert!(matches!(
            i32::try_from(RValue::Integer(vec![None])),
            Err(SexpError::Na(_))
        ));
        assert!(matches!(
            f64::try_from(RValue::Double(vec![crate::altrep_traits::NA_REAL])),
            Err(SexpError::Na(_))
        ));
    }

    #[test]
    fn from_option_scalars() {
        assert!(matches!(RValue::from(Some(7_i32)), RValue::Integer(v) if v == vec![Some(7)]));
        assert!(matches!(RValue::from(None::<i32>), RValue::Integer(v) if v == vec![None]));
        assert!(matches!(RValue::from(Some(true)), RValue::Logical(v) if v == vec![Some(true)]));
        assert!(matches!(RValue::from(None::<bool>), RValue::Logical(v) if v == vec![None]));
        assert!(
            matches!(RValue::from(Some("x")), RValue::Character(v) if v == vec![Some("x".to_string())])
        );
        assert!(matches!(RValue::from(None::<&str>), RValue::Character(v) if v == vec![None]));
        assert!(
            matches!(RValue::from(Some("y".to_string())), RValue::Character(v) if v == vec![Some("y".to_string())])
        );

        // f64 has no Option variant: Some → value, None → NA_REAL bit pattern.
        assert_eq!(RValue::from(Some(1.5_f64)).as_f64(), Some(1.5));
        assert_eq!(RValue::from(None::<f64>).as_f64(), None);
    }

    #[test]
    fn from_vec_option() {
        assert!(
            matches!(RValue::from(vec![Some(1_i32), None, Some(3)]), RValue::Integer(v) if v == vec![Some(1), None, Some(3)])
        );
        assert!(
            matches!(RValue::from(vec![Some(true), None]), RValue::Logical(v) if v == vec![Some(true), None])
        );
        assert!(
            matches!(RValue::from(vec![Some("a".to_string()), None]), RValue::Character(v) if v == vec![Some("a".to_string()), None])
        );
        assert!(
            matches!(RValue::from(vec![Some("b"), None]), RValue::Character(v) if v == vec![Some("b".to_string()), None])
        );

        // Vec<Option<f64>> → Double with NA_REAL in the None slot.
        let RValue::Double(v) = RValue::from(vec![Some(0.5_f64), None]) else {
            panic!("expected Double");
        };
        assert_eq!(v[0], 0.5);
        assert_eq!(v[1].to_bits(), crate::altrep_traits::NA_REAL.to_bits());
    }

    #[test]
    fn from_wide_integer_ladder() {
        // Fits in i32 (and not i32::MIN) → Integer.
        assert!(matches!(RValue::from(42_i64), RValue::Integer(v) if v == vec![Some(42)]));
        assert!(
            matches!(RValue::from(i32::MAX as i64), RValue::Integer(v) if v == vec![Some(i32::MAX)])
        );
        assert!(matches!(RValue::from(7_u32), RValue::Integer(v) if v == vec![Some(7)]));
        // i32::MIN is NA_integer_ → promote to Double rather than emit NA.
        assert!(matches!(RValue::from(i32::MIN as i64), RValue::Double(_)));
        // Just past i32::MAX → Double.
        assert!(
            matches!(RValue::from(i32::MAX as i64 + 1), RValue::Double(v) if v == vec![(i32::MAX as i64 + 1) as f64])
        );
        // u32::MAX exceeds i32::MAX → Double.
        assert!(
            matches!(RValue::from(u32::MAX), RValue::Double(v) if v == vec![f64::from(u32::MAX)])
        );
    }

    #[test]
    fn debug_stringifies() {
        assert_eq!(RValue::debug(0..=100).as_str(), Some("0..=100"));
        assert_eq!(RValue::debug(vec![1, 2]).as_str(), Some("[1, 2]"));
    }

    #[test]
    fn try_from_whole_vector() {
        let v: Vec<Option<i32>> = RValue::Integer(vec![Some(1), None]).try_into().unwrap();
        assert_eq!(v, vec![Some(1), None]);

        let raw: Vec<u8> = RValue::Raw(vec![1, 2, 3]).try_into().unwrap();
        assert_eq!(raw, vec![1, 2, 3]);

        // wrong variant → Type
        let bad: Result<Vec<u8>, _> = RValue::from(1i32).try_into();
        assert!(matches!(bad, Err(SexpError::Type(_))));
    }
}
