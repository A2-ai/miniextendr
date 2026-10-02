//! Compile-pass test: `DataFrameRow` list columns whose rows are wide-integer
//! vectors (`usize` / `u64` / `i64`).
//!
//! An `#[dataframe(as_list)] starts: Vec<usize>` field lands in the companion
//! struct as `Vec<Vec<usize>>`, so the derive needs `Vec<Vec<usize>>: IntoR`.
//! The neighbouring list-column shapes need the matching impls too:
//! `Option<Vec<T>>` fields (`Vec<Option<Vec<T>>>` in the companion and
//! `Option<Vec<T>>: IntoR` for `IntoList`), borrowed `&[T]` fields
//! (`Vec<&[T]>`, and `&[T]: IntoR` for `IntoList`), `Option<Box<[T]>>` fields,
//! and enum fields, which the enum companion stores as `Vec<Option<Vec<T>>>` /
//! `Vec<Option<&[T]>>` / `Vec<Option<Box<[T]>>>`.
//!
//! `Row` is private, as in the shape that first hit the missing impl: the
//! companion struct and its row iterator take the row's visibility, so their
//! `IntoIterator` / `Iterator` impls do not leak a private type (E0446).

#![allow(dead_code)]

use miniextendr_api::IntoR;
use miniextendr_macros::{DataFrameRow, IntoList};

#[derive(Clone, IntoList, DataFrameRow)]
struct Row {
    #[dataframe(as_list)]
    starts: Vec<usize>,
}

/// Without `as_list` an owned `Vec<T>` field is the same opaque list column.
#[derive(Clone, IntoList, DataFrameRow)]
pub struct WideIds {
    pub ids: Vec<u64>,
    pub offsets: Box<[i64]>,
}

#[derive(Clone, IntoList, DataFrameRow)]
pub(crate) struct MaybeStarts {
    #[dataframe(as_list)]
    starts: Option<Vec<usize>>,
}

#[derive(Clone, IntoList, DataFrameRow)]
pub struct BorrowedStarts<'a> {
    pub starts: &'a [usize],
}

#[derive(Clone, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum StartsEvent {
    Hit { starts: Vec<usize> },
    Miss { id: i32 },
}

#[derive(Clone, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum BorrowedStartsEvent<'a> {
    Hit { starts: &'a [i64] },
    Miss { id: i32 },
}

/// An opaque `Box<[T]>` variant field is stored as `Vec<Option<Box<[T]>>>`.
#[derive(Clone, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum BoxedStartsEvent {
    Hit { starts: Box<[usize]>, weights: Box<[f64]> },
    Miss { id: i32 },
}

/// `Option<Box<[T]>>` needs `Option<Box<[T]>>: IntoR` for `IntoList`.
#[derive(Clone, IntoList, DataFrameRow)]
pub struct MaybeBoxedStarts {
    #[dataframe(as_list)]
    pub starts: Option<Box<[u64]>>,
}

fn assert_into_r<T: IntoR>() {}

fn _companions_convert() {
    assert_into_r::<RowDataFrame>();
    assert_into_r::<WideIdsDataFrame>();
    assert_into_r::<MaybeStartsDataFrame>();
    assert_into_r::<BorrowedStartsDataFrame<'static>>();
    assert_into_r::<StartsEventDataFrame>();
    assert_into_r::<BorrowedStartsEventDataFrame<'static>>();
    assert_into_r::<BoxedStartsEventDataFrame>();
    assert_into_r::<MaybeBoxedStartsDataFrame>();
    let rows = vec![Row { starts: vec![1, 5] }, Row { starts: vec![] }];
    let _df: RowDataFrame = Row::to_dataframe(rows);
}

fn main() {}
