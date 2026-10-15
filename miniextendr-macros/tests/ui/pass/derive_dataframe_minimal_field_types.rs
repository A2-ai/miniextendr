//! Compile-pass test: `#[derive(DataFrameRow)]` asks of a field type only what
//! turning it into columns needs: `IntoR` for a column value, `IntoList` for an
//! `as_list` value, `DataFrameRow` for a flattened struct or factor. None of the
//! types below is `Clone`, `Copy`, `Default`, `Debug` or `Send`.

#![allow(dead_code)]

use std::collections::HashMap;
use std::rc::Rc;

use miniextendr_api::{ColumnarFrame, DataFrameRow, IntoList, IntoR};

/// Implements nothing.
struct Opaque(Rc<i32>);

/// A column value: `IntoR` only.
#[derive(IntoR)]
struct Col(f64);

/// A flattened struct: `DataFrameRow` only.
#[derive(DataFrameRow)]
struct Inner {
    q: i32,
}

/// A factor: `DataFrameRow` only.
#[derive(DataFrameRow)]
enum Level {
    Low,
    High,
}

/// An `as_list` value: `IntoList` only.
#[derive(IntoList)]
struct Listed {
    p: f64,
}

#[derive(DataFrameRow)]
struct Row<'a> {
    id: i32,
    name: &'a str,
    values: Vec<Col>,
    boxed: Box<[Col]>,
    by_key: HashMap<String, Col>,
    pair: [Col; 2],
    #[dataframe(width = 2)]
    firsts: Vec<Col>,
    #[dataframe(width = 2)]
    boxed_firsts: Box<[Col]>,
    #[dataframe(expand)]
    rest: Vec<Col>,
    inner: Inner,
    #[dataframe(as_list)]
    listed: Listed,
    #[dataframe(skip)]
    cache: Opaque,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
enum Event {
    A {
        id: i32,
        pair: [Col; 2],
        #[dataframe(width = 2)]
        firsts: Vec<Col>,
        #[dataframe(expand)]
        rest: Vec<Col>,
        inner: Inner,
        #[dataframe(as_factor)]
        level: Level,
        #[dataframe(as_list)]
        listed: Listed,
        #[dataframe(skip)]
        cache: Opaque,
    },
    B {
        id: i32,
        values: Vec<Col>,
        by_key: HashMap<String, Col>,
    },
}

/// Rows that are read back from R: flattened and factor values are moved out
/// of their columns, so `Inner` and `Level` still need no `Clone`.
#[derive(DataFrameRow)]
struct Readable {
    id: i32,
    inner: Inner,
    #[dataframe(as_factor)]
    level: Level,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
enum ReadableEvent {
    A {
        id: i32,
        inner: Inner,
    },
    B {
        id: i32,
        #[dataframe(as_factor)]
        level: Level,
    },
}

fn build(rows: Vec<Row<'_>>, events: Vec<Event>) {
    let _ = RowDataFrame::from_rows(rows);
    let _ = EventDataFrame::from_rows(events);
}

// A factor-only struct must not accidentally demand Clone through the
// parallel reader (no flattened field is present to trigger that fallback).
#[derive(DataFrameRow)]
struct FactorRow {
    #[dataframe(as_factor)]
    level: Level,
}

#[derive(DataFrameRow)]
struct FactorTuple(#[dataframe(as_factor)] Level);

// DataFrameRow is not required of the field: UnitEnumFactor suffices.
struct ManualFactor;
impl miniextendr_api::factor::UnitEnumFactor for ManualFactor {
    const FACTOR_LEVELS: &'static [&'static str] = &["one"];
    fn to_factor_index(self) -> i32 { 1 }
    fn from_factor_index(idx: i32) -> Option<Self> {
        (idx == 1).then_some(Self)
    }
}
#[derive(DataFrameRow)]
struct ManualRow {
    #[dataframe(as_factor)]
    value: ManualFactor,
}

fn read(df: miniextendr_api::SEXP) {
    let _ = FactorRow::try_from_dataframe(df);
    let _ = ManualRow::try_from_dataframe(df);
    let _ = Readable::try_from_dataframe(df);
    let _ = ReadableEvent::try_from_dataframe(df);
}

fn main() {}
