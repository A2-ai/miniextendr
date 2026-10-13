//! Borrowed expansions retain erased lifetimes without inventing from-R borrows (#2028).
#![allow(dead_code)]

use miniextendr_api::{ColumnarFrame, DataFrameRow};

#[derive(DataFrameRow)]
pub struct Array<'a> {
    pub id: i32,
    pub values: &'a [String; 2],
}

#[derive(DataFrameRow)]
pub struct Width<'a> {
    pub id: i32,
    #[dataframe(width = 2)]
    pub values: &'a [String],
}

#[derive(DataFrameRow)]
pub struct Tuple<'a>(pub &'a [f64; 2], #[dataframe(width = 2)] pub &'a [f64]);

#[derive(DataFrameRow)]
pub struct Mixed<'a, 'b> {
    // A real column must not collide with the generated lifetime marker.
    pub __mx_lifetimes: i32,
    pub pair: &'a [f64; 2],
    #[dataframe(expand)]
    pub rest: &'b [f64],
}

#[derive(DataFrameRow)]
pub struct Skipped<'a> {
    pub id: i32,
    #[dataframe(skip)]
    pub label: &'a str,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum ArrayEvent<'a> {
    Named { values: &'a [f64; 2] },
    Tuple(&'a [f64; 2]),
    Empty,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum WidthEvent<'a> {
    Named { #[dataframe(width = 2)] values: &'a [f64] },
    Tuple(#[dataframe(width = 2)] &'a [f64]),
    Empty,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum ExpandEvent<'a> {
    Named { #[dataframe(expand)] values: &'a [f64] },
    Tuple(#[dataframe(expand)] &'a [f64]),
    Empty,
}

// Compile the feature-gated parallel constructors without starting an R pool.
fn parallel<'a>(strings: &'a [String; 2], numbers: &'a [f64; 2]) {
    miniextendr_api::__dataframe_row_when_rayon! {
        let _ = ArrayDataFrame::from_rows_par(vec![Array { id: 1, values: strings }]);
        let _ = WidthDataFrame::from_rows_par(vec![Width { id: 1, values: strings }]);
        let _ = TupleDataFrame::from_rows_par(vec![Tuple(numbers, numbers)]);
        let _ = MixedDataFrame::from_rows_par(vec![Mixed {
            __mx_lifetimes: 1, pair: numbers, rest: numbers,
        }]);
        let _ = SkippedDataFrame::from_rows_par(vec![Skipped { id: 1, label: "ignored" }]);
        let _ = ArrayEventDataFrame::from_rows_par(vec![ArrayEvent::Tuple(numbers)]);
        let _ = WidthEventDataFrame::from_rows_par(vec![WidthEvent::Tuple(numbers)]);
        let _ = ExpandEventDataFrame::from_rows_par(vec![ExpandEvent::Tuple(numbers)]);
    }
    let _ = (strings, numbers);
}

fn main() {
    let strings = [String::from("first"), String::from("second")];
    let array = ArrayDataFrame::from_rows(vec![Array { id: 1, values: &strings }]);
    assert_eq!(array.clone().values_1, ["first"]);
    let width = WidthDataFrame::from_rows(vec![Width { id: 2, values: &strings[..1] }]);
    assert_eq!(width.clone().values_1, [Some(String::from("first"))]);
    assert_eq!(width.values_2, [None]);
    let numbers = [3.0, 5.0];
    let _ = TupleDataFrame::from_rows(vec![Tuple(&numbers, &numbers)]).clone();
    let mixed = MixedDataFrame::from_rows(vec![Mixed {
        __mx_lifetimes: 8, pair: &numbers, rest: &numbers,
    }]);
    assert_eq!(mixed.clone().__mx_lifetimes, [8]);
    let mut rows = SkippedDataFrame::from_rows(vec![Skipped { id: 13, label: "ignored" }])
        .clone().into_iter();
    let row = rows.next().unwrap();
    assert_eq!((row.id, row.label), (13, ""));
    assert!(rows.next().is_none());
    let _ = ArrayEventDataFrame::from_rows(vec![ArrayEvent::Named { values: &numbers }]).clone();
    let _ = WidthEventDataFrame::from_rows(vec![WidthEvent::Named { values: &numbers }]).clone();
    let _ = ExpandEventDataFrame::from_rows(vec![ExpandEvent::Named { values: &numbers }]).clone();
}
