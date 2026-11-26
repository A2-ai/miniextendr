//! Same-named enum fields spelled otherwise share one column (#2141): every
//! field joins its column through `SameColumn`, so rustc, not the derive,
//! decides that an alias, a renamed struct or a module-relative path names the
//! column's type.

use core::primitive;

use miniextendr_api::dataframe::{DataFrame, FromDataFrame, IntoDataFrame};
use miniextendr_api::into_r::IntoR as _;
use miniextendr_api::{DataFrameRow, SEXP, miniextendr};

/// An element type spelled through an alias.
type Count = i32;

/// A nested struct, flattened into `at_x` / `at_y`.
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
pub struct SharedSpot {
    pub x: f64,
    pub y: f64,
}

/// The same struct under another name.
type Place = SharedSpot;

/// `There` spells each of `Here`'s fields otherwise.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum SharedColumnRow {
    Here {
        id: i32,
        at: SharedSpot,
        pair: [i32; 2],
    },
    There {
        id: primitive::i32,
        at: Place,
        pair: [Count; 2],
    },
}

/// The fields a reader can read back, spelled otherwise in `There`.
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum SharedColumnReadRow {
    Here { id: i32, at: SharedSpot },
    There { id: primitive::i32, at: Place },
}

/// A frame of `SharedColumnRow`s: one `id`, `at_x`, `at_y`, `pair_1` and
/// `pair_2` column for both variants.
/// @noRd
#[miniextendr(noexport)]
pub fn df_shared_columns() -> SEXP {
    vec![
        SharedColumnRow::Here {
            id: 1,
            at: SharedSpot { x: 1.0, y: 2.0 },
            pair: [10, 20],
        },
        SharedColumnRow::There {
            id: 2,
            at: SharedSpot { x: 3.0, y: 4.0 },
            pair: [30, 40],
        },
    ]
    .into_dataframe()
    .unwrap()
    .into_sexp()
}

/// A frame of `SharedColumnReadRow`s.
/// @noRd
#[miniextendr(noexport)]
pub fn df_shared_columns_read() -> SEXP {
    vec![
        SharedColumnReadRow::Here {
            id: 1,
            at: SharedSpot { x: 1.0, y: 2.0 },
        },
        SharedColumnReadRow::There {
            id: 2,
            at: SharedSpot { x: 3.0, y: 4.0 },
        },
    ]
    .into_dataframe()
    .unwrap()
    .into_sexp()
}

/// Read a `df_shared_columns_read()` frame back into rows and write them again.
/// @noRd
#[miniextendr(noexport)]
pub fn df_shared_columns_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<SharedColumnReadRow> =
        <Vec<SharedColumnReadRow>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}
