//! Test fixtures for what `#[derive(DataFrameRow)]` asks of field types.
//!
//! The derive needs only what turning a field into columns needs: a column's
//! `IntoR`, `IntoList` for an `as_list` value, `DataFrameRow` for a flattened
//! struct (#2006, #2013). Everything else it generates is conditional:
//!
//! - A `#[dataframe(skip)]` field needs `Default` only for the companion's rows
//!   iterator (`into_rows`); `SkipNoDefault` builds a frame without it.
//! - The companion is `Debug` / `Clone` exactly when its columns are.
//! - Elements of an owned `[T; N]` / `width` / `expand` field are moved into
//!   their columns, so `Moved` needs neither `Clone` nor `Default`.
//! - Flattened and factor values are moved out of their columns when a frame
//!   is read back, so `ReadRow` / `ReadEvent` have `try_from_dataframe` and
//!   `try_from_dataframe_par` readers although `ReadInner` and `ReadLevel` are
//!   not `Clone`.
//! - `from_rows_par` (`rayon`) fills the columns in parallel where the rows
//!   are `Send` and the columns allow it, and is the sequential `from_rows`
//!   otherwise: `SkipNoDefault` is not `Send`, and `MovedElems`'s `pair`
//!   columns have no `Default` placeholder. The compile-time check at the end
//!   asserts that ordinary shapes do get the parallel fill.

#![allow(dead_code)]

use std::rc::Rc;

use miniextendr_api::dataframe::ColumnarFrame;
use miniextendr_api::{BuiltDataFrame, DataFrameRow, IntoDataFrame, IntoR, miniextendr};

// region: Field types

/// A value that implements nothing: not `Default`, `Clone`, `Debug` or `Send`.
pub struct Opaque(Rc<i32>);

/// A column value that converts to R and does nothing else: not `Clone`,
/// `Default` or `Debug`.
#[derive(IntoR)]
pub struct Moved(pub f64);

// endregion

// region: Rows

/// A skipped field whose type implements nothing.
#[derive(DataFrameRow)]
pub struct SkipNoDefault {
    pub id: i32,
    pub label: String,
    #[dataframe(skip)]
    pub cache: Opaque,
}

fn skip_no_default_rows() -> Vec<SkipNoDefault> {
    vec![
        SkipNoDefault {
            id: 1,
            label: "a".to_string(),
            cache: Opaque(Rc::new(10)),
        },
        SkipNoDefault {
            id: 2,
            label: "b".to_string(),
            cache: Opaque(Rc::new(20)),
        },
    ]
}

/// Expanded fields over an element type that is neither `Clone` nor `Default`.
#[derive(DataFrameRow)]
pub struct MovedElems {
    pub id: i32,
    pub pair: [Moved; 2],
    #[dataframe(width = 3)]
    pub firsts: Vec<Moved>,
    #[dataframe(expand)]
    pub rest: Box<[Moved]>,
}

fn moved_elems_rows() -> Vec<MovedElems> {
    vec![
        MovedElems {
            id: 1,
            pair: [Moved(1.0), Moved(2.0)],
            firsts: vec![Moved(10.0)],
            rest: vec![Moved(100.0), Moved(200.0)].into_boxed_slice(),
        },
        MovedElems {
            id: 2,
            pair: [Moved(3.0), Moved(4.0)],
            firsts: vec![Moved(20.0), Moved(21.0), Moved(22.0), Moved(23.0)],
            rest: vec![Moved(300.0)].into_boxed_slice(),
        },
    ]
}

/// The enum counterpart of `MovedElems`.
#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum MovedEvent {
    Pair {
        id: i32,
        pair: [Moved; 2],
    },
    Spread {
        id: i32,
        #[dataframe(width = 2)]
        vals: Vec<Moved>,
    },
}

fn moved_event_rows() -> Vec<MovedEvent> {
    vec![
        MovedEvent::Pair {
            id: 1,
            pair: [Moved(1.0), Moved(2.0)],
        },
        MovedEvent::Spread {
            id: 2,
            vals: vec![Moved(5.0)],
        },
    ]
}

/// Columns that are all `Debug + Clone`, so the companion is too.
#[derive(DataFrameRow)]
pub struct DebugPoint {
    pub x: f64,
    pub label: String,
}

/// The enum counterpart of `DebugPoint`.
#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum DebugEvent {
    A { x: i32 },
    B { y: String },
}

/// A flattened struct that is not `Clone`.
#[derive(DataFrameRow)]
pub struct ReadInner {
    pub q: i32,
}

/// A factor that is neither `Clone` nor `Copy`.
#[derive(DataFrameRow)]
pub enum ReadLevel {
    Low,
    High,
}

/// A struct row that is read back from R: flattened values are moved out of
/// their columns, sequentially and with `rayon`.
#[derive(DataFrameRow)]
pub struct ReadRow {
    pub id: i32,
    pub inner: ReadInner,
    #[dataframe(as_factor)]
    pub level: ReadLevel,
}

/// The enum counterpart of `ReadRow`.
#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum ReadEvent {
    A {
        id: i32,
        inner: ReadInner,
    },
    B {
        id: i32,
        #[dataframe(as_factor)]
        level: ReadLevel,
    },
}

// endregion

// region: #[miniextendr] entrypoints

/// A row whose skipped field implements nothing still becomes a data.frame.
/// Expected columns: `id`, `label`.
#[miniextendr]
pub fn bounds_skip_no_default() -> BuiltDataFrame {
    skip_no_default_rows().into_dataframe().unwrap()
}

/// The parallel build of `SkipNoDefault`: the rows are not `Send`, so this is
/// the sequential fill. Same frame as `bounds_skip_no_default()`.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn bounds_skip_no_default_par() -> BuiltDataFrame {
    skip_no_default_rows().into_dataframe_par().unwrap()
}

/// Expanded fields whose elements are moved, not cloned.
/// Expected columns: `id`, `pair_1`, `pair_2`, `firsts_1`..`firsts_3`,
/// `rest_1`, `rest_2`.
#[miniextendr]
pub fn bounds_moved_elems() -> BuiltDataFrame {
    moved_elems_rows().into_dataframe().unwrap()
}

/// The parallel build of `MovedElems`: `Moved` is not `Default`, so the
/// `pair` columns have no placeholder and this is the sequential fill. Same
/// frame as `bounds_moved_elems()`.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn bounds_moved_elems_par() -> BuiltDataFrame {
    moved_elems_rows().into_dataframe_par().unwrap()
}

/// The enum counterpart of `bounds_moved_elems()`.
/// Expected columns: `kind`, `id`, `pair_1`, `pair_2`, `vals_1`, `vals_2`.
#[miniextendr]
pub fn bounds_moved_event() -> BuiltDataFrame {
    moved_event_rows().into_dataframe().unwrap()
}

/// The parallel build of `MovedEvent`: enum columns are `None`-filled, so
/// this is the parallel fill. Same frame as `bounds_moved_event()`.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn bounds_moved_event_par() -> BuiltDataFrame {
    moved_event_rows().into_dataframe_par().unwrap()
}

/// `Debug` of a clone of a `DebugPoint` companion.
#[miniextendr]
pub fn bounds_companion_debug_clone() -> String {
    let df = DebugPointDataFrame::from_rows(vec![
        DebugPoint {
            x: 1.5,
            label: "a".to_string(),
        },
        DebugPoint {
            x: 2.5,
            label: "b".to_string(),
        },
    ]);
    format!("{:?}", df.clone())
}

/// `Debug` of a clone of a `DebugEvent` companion.
#[miniextendr]
pub fn bounds_enum_companion_debug_clone() -> String {
    let df = DebugEventDataFrame::from_rows(vec![
        DebugEvent::A { x: 1 },
        DebugEvent::B { y: "b".to_string() },
    ]);
    format!("{:?}", df.clone())
}

// endregion

// region: The parallel fill applies to ordinary shapes

// `from_rows_par` falls back to `from_rows` silently when the parallel fill
// does not apply, so assert at compile time that it applies where it should.
#[cfg(feature = "rayon")]
const _: () = {
    use crate::dataframe_collections_test::{ParallelExpanded, ParallelExpandedDataFrame};
    use crate::dataframe_struct_flatten_test::{
        FlatAsList, FlatAsListDataFrame, FlatLocated, FlatLocatedDataFrame,
    };
    use miniextendr_api::dataframe::ParallelColumns;

    fn assert_parallel<Df: ParallelColumns<Row>, Row>() {}

    fn assert_shapes() {
        assert_parallel::<DebugPointDataFrame, DebugPoint>();
        assert_parallel::<DebugEventDataFrame, DebugEvent>();
        assert_parallel::<MovedEventDataFrame, MovedEvent>();
        assert_parallel::<ParallelExpandedDataFrame, ParallelExpanded>();
        assert_parallel::<FlatLocatedDataFrame, FlatLocated>();
        assert_parallel::<FlatAsListDataFrame, FlatAsList>();
    }

    // The parallel readers exist for rows whose flattened and factor types
    // are not `Clone`.
    fn read_par(df: miniextendr_api::SEXP) {
        let _ = ReadRow::try_from_dataframe_par(df);
        let _ = ReadEvent::try_from_dataframe_par(df);
    }
};

// endregion

// region: Rust tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_without_default_builds_columns() {
        let df = SkipNoDefaultDataFrame::from_rows(skip_no_default_rows());
        assert_eq!(df.id, vec![1, 2]);
        assert_eq!(df.label, vec!["a".to_string(), "b".to_string()]);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn skip_without_send_falls_back_to_sequential() {
        let df = SkipNoDefaultDataFrame::from_rows_par(skip_no_default_rows());
        assert_eq!(df.id, vec![1, 2]);
        assert_eq!(df.label, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn expanded_elements_are_moved() {
        let df = MovedElemsDataFrame::from_rows(moved_elems_rows());
        assert_eq!(
            df.pair_1.iter().map(|m| m.0).collect::<Vec<_>>(),
            [1.0, 3.0]
        );
        assert_eq!(
            df.pair_2.iter().map(|m| m.0).collect::<Vec<_>>(),
            [2.0, 4.0]
        );
        let firsts_2: Vec<Option<f64>> = df
            .firsts_2
            .iter()
            .map(|m| m.as_ref().map(|m| m.0))
            .collect();
        assert_eq!(firsts_2, [None, Some(21.0)]);
        assert_eq!(df.rest.len(), 2);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn expanded_elements_par_matches_sequential() {
        let seq = MovedElemsDataFrame::from_rows(moved_elems_rows());
        let par = MovedElemsDataFrame::from_rows_par(moved_elems_rows());
        let values = |col: &[Option<Moved>]| {
            col.iter()
                .map(|m| m.as_ref().map(|m| m.0))
                .collect::<Vec<_>>()
        };
        assert_eq!(values(&seq.firsts_1), values(&par.firsts_1));
        assert_eq!(values(&seq.firsts_3), values(&par.firsts_3));
        assert_eq!(
            seq.pair_1.iter().map(|m| m.0).collect::<Vec<_>>(),
            par.pair_1.iter().map(|m| m.0).collect::<Vec<_>>()
        );
    }

    #[test]
    fn companions_are_debug_and_clone() {
        assert_eq!(
            bounds_companion_debug_clone(),
            r#"DebugPointDataFrame { x: [1.5, 2.5], label: ["a", "b"] }"#
        );
        assert_eq!(
            bounds_enum_companion_debug_clone(),
            r#"DebugEventDataFrame { _tag: ["A", "B"], x: [Some(1), None], y: [None, Some("b")] }"#
        );
    }
}

// endregion
