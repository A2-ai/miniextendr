//! #2020: generated code stays unambiguous under `use miniextendr_api::*`.
//! The glob brings in the adapter traits, whose blanket impls gave every
//! `Clone` type a second `clone` while `RClone`'s method had that name (it
//! is `r_clone` now), so a generated `x.clone()` was ambiguous. The UI
//! fixture `tests/ui/pass/glob_import_method_calls.rs` covers every macro
//! the UI tests' crate can compile; this module covers the
//! paths that need features that crate lacks: the parallel `DataFrameRow`
//! readers (`rayon`), the Vctrs derive (`vctrs`), a worker wrapper
//! (`worker-thread`) and a serde-classed error (`serde`).
#![allow(dead_code)]
// Exercises the deprecated `worker` option (#2117).
#![allow(deprecated)]

use miniextendr_api::*;
use std::collections::{BTreeMap, HashMap};

// Consumer blankets are a separate hazard from the crate's own glob (#2020).
// The vctrs examples import this too, covering all three conversion branches.
pub(crate) trait ConsumerConversions: Sized {
    fn into_sexp(self) {}
    fn try_into_sexp(self) {}
    fn into_sexp_unchecked(self) {}
    fn try_into_sexp_unchecked(self) {}
}
impl<T> ConsumerConversions for T {}

// A distinct return type also catches silent interception, not just E0034.
pub(crate) struct ConsumerIteratorCalled;

pub(crate) trait ConsumerIterators: Sized {
    fn into_iter(self) -> ConsumerIteratorCalled {
        ConsumerIteratorCalled
    }
    fn into_par_iter(self) -> ConsumerIteratorCalled {
        ConsumerIteratorCalled
    }
    fn next(&mut self) -> ConsumerIteratorCalled {
        ConsumerIteratorCalled
    }
    fn map<F>(self, _f: F) -> ConsumerIteratorCalled {
        ConsumerIteratorCalled
    }
    fn filter<F>(self, _f: F) -> ConsumerIteratorCalled {
        ConsumerIteratorCalled
    }
    fn filter_map<F>(self, _f: F) -> ConsumerIteratorCalled {
        ConsumerIteratorCalled
    }
    fn collect<T>(self) -> ConsumerIteratorCalled {
        ConsumerIteratorCalled
    }
    fn enumerate(self) -> ConsumerIteratorCalled {
        ConsumerIteratorCalled
    }
    fn unzip<A, B, C, D>(self) -> ConsumerIteratorCalled {
        ConsumerIteratorCalled
    }
    fn flatten(self) -> ConsumerIteratorCalled {
        ConsumerIteratorCalled
    }
    fn zip<T>(self, _other: T) -> ConsumerIteratorCalled {
        ConsumerIteratorCalled
    }
    fn for_each<F>(self, _f: F) -> ConsumerIteratorCalled {
        ConsumerIteratorCalled
    }
    fn position<F>(&mut self, _f: F) -> ConsumerIteratorCalled {
        ConsumerIteratorCalled
    }
    fn chain<T>(self, _other: T) -> ConsumerIteratorCalled {
        ConsumerIteratorCalled
    }
    fn copied(self) -> ConsumerIteratorCalled {
        ConsumerIteratorCalled
    }
    fn max(self) -> ConsumerIteratorCalled {
        ConsumerIteratorCalled
    }
}
impl<T> ConsumerIterators for T {}

pub(crate) struct ConsumerCollectionCalled;

// A by-value receiver can win over an inherent method that borrows a Vec.
#[allow(clippy::wrong_self_convention)]
pub(crate) trait ConsumerCollections: Sized {
    fn len(self) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
    fn push<T>(self, _value: T) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
    fn iter(self) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
    fn iter_mut(self) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
    fn as_str(self) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
    fn as_bytes(self) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
    fn is_empty(self) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
    fn contains<T>(self, _value: T) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
    fn starts_with<T>(self, _value: T) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
    fn strip_prefix<T>(self, _value: T) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
    fn join<T>(self, _sep: T) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
    fn into_boxed_slice(self) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
}
impl<T> ConsumerCollections for T {}

pub(crate) trait ConsumerFormatting {
    fn to_string(&self) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
}
impl<T: ?Sized> ConsumerFormatting for T {}

// Borrowing Option methods can lose to a consumer method taking self by value.
#[allow(clippy::wrong_self_convention)]
pub(crate) trait ConsumerOptionMethods: Sized {
    fn is_none(self) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
    fn is_some(self) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
    fn as_ref(self) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
    fn as_deref(self) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
}
impl<T> ConsumerOptionMethods for T {}

pub(crate) trait ConsumerAssociated: Sized {
    fn from<T>(_value: T) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
    fn try_from<T>(_value: T) -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
    fn default() -> ConsumerCollectionCalled {
        ConsumerCollectionCalled
    }
}
impl<T> ConsumerAssociated for T {}

// No nested struct: the parallel reader assembles each row by index, cloning
// the cells out of the pre-extracted columns.
#[derive(Clone, DataFrameRow)]
pub struct GlobRow {
    pub id: i32,
    pub weight: f64,
    pub maybe: Option<f64>,
    pub coords: [f64; 2],
    #[dataframe(expand)]
    pub corners: [i32; 2],
    #[dataframe(width = 2)]
    pub scores: Vec<f64>,
    #[dataframe(expand)]
    pub extra: Vec<i32>,
    pub counts: HashMap<String, i32>,
}

#[derive(Clone, DataFrameRow)]
#[dataframe(tag = "_type")]
pub enum GlobEvent {
    Point {
        id: i32,
        coords: [f64; 2],
    },
    Scores {
        #[dataframe(width = 2)]
        scores: Vec<f64>,
        #[dataframe(expand)]
        extra: Vec<i32>,
        counts: BTreeMap<String, i32>,
    },
    Empty,
}

#[derive(Clone, DataFrameRow)]
pub struct GlobInner {
    pub a: i32,
    pub b: f64,
}

// A nested struct: the parallel reader hands off to the sequential one.
#[derive(Clone, DataFrameRow)]
#[dataframe(tag = "_type")]
pub enum GlobNested {
    Has { id: i32, inner: GlobInner },
    Without { id: i32 },
}

fn glob_rows() -> Vec<GlobRow> {
    Iterator::collect(Iterator::map(0..4, |i| GlobRow {
        id: i,
        weight: <f64 as std::convert::From<_>>::from(i) / 2.0,
        maybe: (i % 2 == 0).then_some(<f64 as std::convert::From<_>>::from(i)),
        coords: [
            <f64 as std::convert::From<_>>::from(i),
            -<f64 as std::convert::From<_>>::from(i),
        ],
        corners: [i, i + 1],
        scores: vec![1.0, 2.0],
        extra: vec![i, i * 2],
        counts: <HashMap<_, _> as std::convert::From<_>>::from([(format!("k{i}"), i)]),
    }))
}

fn glob_events() -> Vec<GlobEvent> {
    vec![
        GlobEvent::Point {
            id: 1,
            coords: [1.0, 2.0],
        },
        GlobEvent::Scores {
            scores: vec![3.0, 4.0],
            extra: vec![5, 6],
            counts: <BTreeMap<_, _> as std::convert::From<_>>::from([(
                std::string::ToString::to_string("a"),
                1,
            )]),
        },
        GlobEvent::Empty,
    ]
}

fn glob_nested() -> Vec<GlobNested> {
    vec![
        GlobNested::Has {
            id: 1,
            inner: GlobInner { a: 2, b: 3.0 },
        },
        GlobNested::Without { id: 4 },
    ]
}

/// Write rows to data frames and read them back with every reader.
///
/// Returns the row counts read: struct sequential and parallel, enum
/// sequential and parallel, nested enum sequential and parallel. A reader
/// that's missing (without `rayon`) reports -1.
#[miniextendr(noexport)]
pub fn glob_import_roundtrip() -> Vec<i32> {
    fn count<T>(rows: Result<Vec<T>, String>) -> i32 {
        rows.map_or(-2, |rows| {
            <i32 as std::convert::TryFrom<_>>::try_from(Vec::len(&rows)).unwrap_or(i32::MAX)
        })
    }

    let rows = miniextendr_api::IntoR::into_sexp(GlobRow::to_dataframe(glob_rows()));
    // SAFETY: on the R thread; each frame is fresh and stays protected while
    // the readers allocate.
    let rows = unsafe { miniextendr_api::gc_protect::OwnedProtect::new(rows) };
    let events = miniextendr_api::IntoR::into_sexp(GlobEvent::to_dataframe(glob_events()));
    let events = unsafe { miniextendr_api::gc_protect::OwnedProtect::new(events) };
    let nested = miniextendr_api::IntoR::into_sexp(GlobNested::to_dataframe(glob_nested()));
    let nested = unsafe { miniextendr_api::gc_protect::OwnedProtect::new(nested) };

    #[cfg(feature = "rayon")]
    let par = [
        count(GlobRow::try_from_dataframe_par(rows.get())),
        count(GlobEvent::try_from_dataframe_par(events.get())),
        count(GlobNested::try_from_dataframe_par(nested.get())),
    ];
    #[cfg(not(feature = "rayon"))]
    let par = [-1; 3];
    vec![
        count(GlobRow::try_from_dataframe(rows.get())),
        par[0],
        count(GlobEvent::try_from_dataframe(events.get())),
        par[1],
        count(GlobNested::try_from_dataframe(nested.get())),
        par[2],
    ]
}

/// Exercise Option<()> return handling under the consumer blanket trait.
#[miniextendr(noexport)]
pub fn glob_import_option_unit(present: bool) -> Option<()> {
    present.then_some(())
}

/// Exercise condition field conversion under consumer associated functions.
#[miniextendr(noexport)]
pub fn glob_import_condition_payload() {
    miniextendr_api::rust_error!(
        class = "glob_conversion",
        data = { code = 7_i32, labels = vec!["a", "b"] },
        "refused",
    );
}

/// A vctrs class derived under the glob import.
#[cfg(feature = "vctrs")]
#[derive(Vctrs)]
// Inheritance emits a class-name iterator pipeline under ConsumerIterators.
#[vctrs(class = "glob_percent", base = "double", extends = "derived_percent")]
pub struct GlobPercent {
    #[vctrs(data)]
    values: Vec<f64>,
}

/// A vctrs record derived under the glob import.
#[cfg(feature = "vctrs")]
#[derive(Vctrs, PreferVctrs)]
#[vctrs(class = "glob_ratio", base = "record")]
pub struct GlobRatio {
    #[vctrs(data)]
    num: Vec<i32>,
    den: Vec<i32>,
}

/// Sum `xs` and add the byte length of `label` on the worker thread.
/// @param label A string.
/// @param xs A numeric vector.
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(worker, noexport)]
pub fn glob_import_worker(label: &str, xs: Vec<f64>) -> Result<f64, String> {
    if <[_]>::is_empty(&xs) {
        Err(std::string::ToString::to_string("empty"))
    } else {
        Ok(<[_]>::iter(&xs).sum::<f64>() + str::len(label) as f64)
    }
}

/// Stand-in for a serde-tagged error enum.
#[cfg(feature = "serde")]
#[derive(Debug, crate::serde::Serialize)]
#[serde(crate = "crate::serde", tag = "kind", rename_all = "snake_case")]
pub enum GlobSerdeError {
    Refused { code: i32 },
}

#[cfg(feature = "serde")]
impl std::fmt::Display for GlobSerdeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlobSerdeError::Refused { code } => write!(f, "refused with {code}"),
        }
    }
}

/// Refuse `code` with a serde-classed error.
/// @param code An integer.
#[cfg(feature = "serde")]
#[miniextendr(noexport, serde_error(prefix = "glob"))]
pub fn glob_import_serde(code: i32) -> Result<i32, GlobSerdeError> {
    Err(GlobSerdeError::Refused { code })
}
