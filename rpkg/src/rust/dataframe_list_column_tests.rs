//! Test fixtures for `DataFrameRow` list columns of wide-integer vectors.
//!
//! An `#[dataframe(as_list)]` field `starts: Vec<usize>` lands in the companion
//! struct as `Vec<Vec<usize>>`, so the data frame needs `Vec<Vec<usize>>: IntoR`.
//! Each cell is converted exactly like a returned `Vec<usize>`: an integer
//! vector when every value fits, otherwise a double vector (decided per row);
//! `#[miniextendr(strict)]` returns take the checked path instead.

use miniextendr_api::{BuiltDataFrame, DataFrameRow, IntoDataFrame, IntoList, miniextendr};

/// Larger than `i32::MAX`, exact as a double.
const BIG_START: usize = 3_000_000_000;

#[derive(Clone, IntoList, DataFrameRow)]
struct ListColumnStartsRow {
    #[dataframe(as_list)]
    starts: Vec<usize>,
}

#[derive(Clone, IntoList, DataFrameRow)]
struct ListColumnMaybeStartsRow {
    #[dataframe(as_list)]
    starts: Option<Vec<usize>>,
}

fn starts_rows(rows: Vec<Vec<usize>>) -> BuiltDataFrame {
    rows.into_iter()
        .map(|starts| ListColumnStartsRow { starts })
        .collect::<Vec<_>>()
        .into_dataframe()
        .unwrap()
}

/// Data frame with one list column of start IDs; rows 2 and 4 have none.
#[miniextendr]
pub fn list_column_starts_df() -> BuiltDataFrame {
    starts_rows(vec![vec![1, 5, 9], vec![], vec![42], vec![]])
}

/// Data frame whose second row holds a start ID past `i32::MAX`.
#[miniextendr]
pub fn list_column_starts_wide_df() -> BuiltDataFrame {
    starts_rows(vec![vec![1, 2], vec![BIG_START, 7], vec![]])
}

/// Data frame with an optional list column: `None` rows are `NULL` cells.
#[miniextendr]
pub fn list_column_maybe_starts_df() -> BuiltDataFrame {
    vec![
        ListColumnMaybeStartsRow {
            starts: Some(vec![3, 4]),
        },
        ListColumnMaybeStartsRow { starts: None },
        ListColumnMaybeStartsRow {
            starts: Some(vec![]),
        },
    ]
    .into_dataframe()
    .unwrap()
}

#[derive(Clone, DataFrameRow)]
#[dataframe(align, tag = "_type")]
enum ListColumnStartsEvent {
    Hit {
        starts: Vec<usize>,
        offsets: Box<[usize]>,
    },
    Miss {
        id: i32,
    },
}

/// Enum data frame: the `Miss` rows have `NULL` list cells.
#[miniextendr]
pub fn list_column_starts_event_df() -> BuiltDataFrame {
    vec![
        ListColumnStartsEvent::Hit {
            starts: vec![1, BIG_START],
            offsets: vec![0].into_boxed_slice(),
        },
        ListColumnStartsEvent::Miss { id: 7 },
        ListColumnStartsEvent::Hit {
            starts: vec![],
            offsets: Box::new([]),
        },
    ]
    .into_dataframe()
    .unwrap()
}

/// `Vec<Vec<usize>>` returned directly (default, lossy widening).
#[miniextendr]
pub fn list_column_starts_rows_wide() -> Vec<Vec<usize>> {
    vec![vec![1, 2], vec![BIG_START], vec![]]
}

/// `Vec<Vec<usize>>` returned under strict mode, every value in range.
#[miniextendr(strict)]
pub fn strict_list_column_starts_rows() -> Vec<Vec<usize>> {
    vec![vec![1, 2], vec![], vec![i32::MAX as usize]]
}

/// `Vec<Vec<usize>>` returned under strict mode with values past `i32::MAX`.
#[miniextendr(strict)]
pub fn strict_list_column_starts_rows_overflow() -> Vec<Vec<usize>> {
    vec![vec![1], vec![BIG_START, 2], vec![], vec![3, BIG_START]]
}

/// `Option<Vec<usize>>` returned under strict mode with a value past `i32::MAX`.
#[miniextendr(strict)]
pub fn strict_maybe_starts_overflow() -> Option<Vec<usize>> {
    Some(vec![BIG_START])
}

/// `Option<Vec<usize>>` returned under strict mode as `None`.
#[miniextendr(strict)]
pub fn strict_maybe_starts_none() -> Option<Vec<usize>> {
    None
}
