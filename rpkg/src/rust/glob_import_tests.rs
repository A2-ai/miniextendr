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
    (0..4)
        .map(|i| GlobRow {
            id: i,
            weight: f64::from(i) / 2.0,
            maybe: (i % 2 == 0).then_some(f64::from(i)),
            coords: [f64::from(i), -f64::from(i)],
            corners: [i, i + 1],
            scores: vec![1.0, 2.0],
            extra: vec![i, i * 2],
            counts: HashMap::from([(format!("k{i}"), i)]),
        })
        .collect()
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
            counts: BTreeMap::from([("a".to_string(), 1)]),
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
    use miniextendr_api::into_r::IntoR as _;

    fn count<T>(rows: Result<Vec<T>, String>) -> i32 {
        rows.map_or(-2, |rows| i32::try_from(rows.len()).unwrap_or(i32::MAX))
    }

    let rows = GlobRow::to_dataframe(glob_rows()).into_sexp();
    // SAFETY: on the R thread; each frame is fresh and stays protected while
    // the readers allocate.
    let rows = unsafe { miniextendr_api::gc_protect::OwnedProtect::new(rows) };
    let events = GlobEvent::to_dataframe(glob_events()).into_sexp();
    let events = unsafe { miniextendr_api::gc_protect::OwnedProtect::new(events) };
    let nested = GlobNested::to_dataframe(glob_nested()).into_sexp();
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

/// A vctrs class derived under the glob import.
#[cfg(feature = "vctrs")]
#[derive(Vctrs)]
#[vctrs(class = "glob_percent", base = "double")]
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
    if xs.is_empty() {
        Err("empty".to_string())
    } else {
        Ok(xs.iter().sum::<f64>() + label.len() as f64)
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

/// The serde-classed error's options as a named shape (#2207).
#[cfg(feature = "serde")]
const GLOB_SHAPE: condition::SerdeErrorShape = condition::SerdeErrorShape::new().prefix("glob");

/// Refuse `code` with a serde-classed error.
/// @param code An integer.
#[cfg(feature = "serde")]
#[miniextendr(noexport, serde_error(shape = GLOB_SHAPE, as_integer("code")))]
pub fn glob_import_serde(code: i32) -> Result<i32, GlobSerdeError> {
    Err(GlobSerdeError::Refused { code })
}
