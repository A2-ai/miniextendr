//! Round-trip fixtures for enum-path `FromDataFrame` readers (#807).
//!
//! `#[derive(DataFrameRow)]` now generates a `try_from_dataframe` / `_par` reader
//! for tagged enum shapes, surfaced as `Vec::<E>::from_dataframe(&df)`. This file
//! exercises the following shapes:
//!
//!   - **Scalar tagged union** — unit + payload variants, scalar `Single` fields only.
//!   - **Column-expansion in variants** — `[T; N]` fixed-array and `Vec<T>` + `width`.
//!   - **`as_factor` unit-only nested enum** — factor column round-trip.
//!   - **Nested payload-bearing enum flatten** — inner enum read back via its own reader
//!     after densifying the sub-frame with `DataFrame::select_rows`.
//!   - **Struct-flatten variant field** — inner `DataFrameRow` struct.
//!
//! Each `*_roundtrip(df)` reads a `data.frame` into `Vec<E>` with the reader, then
//! rebuilds it with the writer — so `roundtrip(make()) == make()` (R-side column
//! equality) proves the reader reconstructs rows that re-serialise to the same frame.
//!
//! GC-stress fixtures (`gc_stress_reader_enum_*`) are in `gc_stress_fixtures.rs`.
//! **Map-column enum readers** are deferred to a follow-up issue (#814).
//!
//! # Round-trip caveat
//!
//! Absent-variant cells are `NA` in the writer's output and the reader produces
//! variants that re-write the same `NA` pattern, so `roundtrip(input) == input` holds
//! exactly when `input` is itself a writer-shaped frame. Tests build inputs either by
//! calling an existing align entrypoint or by hand-constructing them with the precise
//! `NA`-fill layout the writer would produce.

#![allow(dead_code)]

use miniextendr_api::dataframe::{DataFrame, FromDataFrame, IntoDataFrame};
use miniextendr_api::into_r::IntoR as _;
use miniextendr_api::{DataFrameRow, IntoList, SEXP, miniextendr};

// region: enum row types

// region: REScalar — scalar tagged union (unit + payload variants)

/// Scalar tagged-union enum: scalar `Single` fields, includes a unit variant.
///
/// Columns: `_type` (tag), `id` (i32, present in Click + Key), `x` (f64, Click only),
/// `name` (String, Key only). Unit variant `Tick` → all payload columns NA.
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum REScalar {
    Click { id: i32, x: f64 },
    Key { id: i32, name: String },
    Tick,
}

// endregion

// region: REExpand — column-expansion in variants

/// Column-expansion: fixed-array `[f64; 2]` and `Vec<f64>` + `width = 3`.
///
/// Columns: `_type`, `id` (i32), `c_1..c_2` (f64, Coords only),
/// `s_1..s_3` (f64 Option, Scores only). `Bare` variant → all expansion columns NA.
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum REExpand {
    Coords {
        id: i32,
        c: [f64; 2],
    },
    Scores {
        id: i32,
        #[dataframe(width = 3)]
        s: Vec<f64>,
    },
    Bare {
        id: i32,
    },
}

// endregion

// region: REDir + REMove — as_factor unit-only nested enum

/// Unit-only direction enum — factor column in REMove.
#[derive(Clone, Copy, Debug, PartialEq, DataFrameRow)]
#[dataframe(align, tag = "variant")]
pub enum REDir {
    North,
    South,
    East,
    West,
}

/// Enum with an `as_factor` nested unit enum. Columns: `_type`, `id` (i32),
/// `dir` (factor<REDir>, Move only).
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum REMove {
    Move {
        id: i32,
        #[dataframe(as_factor)]
        dir: REDir,
    },
    Stop {
        id: i32,
    },
}

// endregion

// region: REStatus + RETracked — nested payload-bearing enum flatten

/// Inner enum: payload-bearing, tagged by `variant`.
/// Columns (when flattened): `<prefix>_variant`, `<prefix>_code`.
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
#[dataframe(align, tag = "variant")]
pub enum REStatus {
    Ok,
    Err { code: i32 },
}

/// Outer enum with a nested `REStatus` flatten. Columns:
/// `_type`, `id` (i32), `status_variant` (String, Tracked only),
/// `status_code` (i32, Tracked/Err only).
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum RETracked {
    Tracked { id: i32, status: REStatus },
    Other { id: i32 },
}

// endregion

// region: REPoint + RELoc — struct-flatten variant field

/// Inner struct: implements DataFrameRow (struct path).
#[derive(Clone, Debug, PartialEq, IntoList, DataFrameRow)]
pub struct REPoint {
    pub x: f64,
    pub y: f64,
}

/// Enum with a struct-flatten field. Columns: `_type`, `id` (i32),
/// `p_x` (f64, At only), `p_y` (f64, At only).
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum RELoc {
    At { id: i32, p: REPoint },
    Nowhere { id: i32 },
}

// endregion

// endregion

// region: round-trip entrypoints

// region: REScalar round-trips

/// Scalar tagged-union enum round-trip. Columns: `_type`, `id`, `x`, `name`.
/// @param df data.frame from `re_scalar_align(...)`.
/// @export
#[miniextendr]
pub fn re_scalar_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<REScalar> = <Vec<REScalar>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// Produce the writer output for a mixed REScalar frame (used by R tests).
/// @export
#[miniextendr]
pub fn re_scalar_align() -> SEXP {
    vec![
        REScalar::Click { id: 1, x: 1.5 },
        REScalar::Key {
            id: 2,
            name: "enter".to_string(),
        },
        REScalar::Tick,
        REScalar::Click { id: 4, x: 2.5 },
    ]
    .into_dataframe()
    .unwrap()
    .into_sexp()
}

/// Zero-row REScalar round-trip (empty frame). @export
#[miniextendr]
pub fn re_scalar_roundtrip_zero(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<REScalar> = <Vec<REScalar>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// Parallel REScalar round-trip. @export
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn re_scalar_roundtrip_par(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<REScalar> = <Vec<REScalar>>::from_dataframe_par(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

// endregion

// region: REExpand round-trips

/// Column-expansion round-trip. Columns: `_type`, `id`, `c_1..c_2`, `s_1..s_3`.
/// @param df data.frame from `re_expand_align(...)`.
/// @export
#[miniextendr]
pub fn re_expand_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<REExpand> = <Vec<REExpand>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// Produce the writer output for a mixed REExpand frame. @export
#[miniextendr]
pub fn re_expand_align() -> SEXP {
    vec![
        REExpand::Coords {
            id: 1,
            c: [3.0, 4.0],
        },
        REExpand::Scores {
            id: 2,
            s: vec![10.0, 20.0, 30.0],
        },
        REExpand::Bare { id: 3 },
    ]
    .into_dataframe()
    .unwrap()
    .into_sexp()
}

/// Parallel REExpand round-trip. @export
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn re_expand_roundtrip_par(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<REExpand> = <Vec<REExpand>>::from_dataframe_par(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

// endregion

// region: REMove round-trips

/// as_factor nested enum round-trip. Columns: `_type`, `id`, `dir` (factor).
/// @param df data.frame from `re_move_align(...)`.
/// @export
#[miniextendr]
pub fn re_move_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<REMove> = <Vec<REMove>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// Produce the writer output for a mixed REMove frame. @export
#[miniextendr]
pub fn re_move_align() -> SEXP {
    vec![
        REMove::Move {
            id: 1,
            dir: REDir::North,
        },
        REMove::Stop { id: 2 },
        REMove::Move {
            id: 3,
            dir: REDir::West,
        },
    ]
    .into_dataframe()
    .unwrap()
    .into_sexp()
}

// endregion

// region: RETracked round-trips

/// Nested payload-bearing enum flatten round-trip.
/// Columns: `_type`, `id`, `status_variant`, `status_code`.
/// @param df data.frame from `re_tracked_align(...)`.
/// @export
#[miniextendr]
pub fn re_tracked_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<RETracked> = <Vec<RETracked>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// Produce the writer output for a mixed RETracked frame. @export
#[miniextendr]
pub fn re_tracked_align() -> SEXP {
    vec![
        RETracked::Tracked {
            id: 1,
            status: REStatus::Ok,
        },
        RETracked::Tracked {
            id: 2,
            status: REStatus::Err { code: 404 },
        },
        RETracked::Other { id: 3 },
    ]
    .into_dataframe()
    .unwrap()
    .into_sexp()
}

/// Parallel RETracked round-trip (delegates to sequential due to Struct field). @export
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn re_tracked_roundtrip_par(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<RETracked> = <Vec<RETracked>>::from_dataframe_par(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

// endregion

// region: RELoc round-trips

/// Struct-flatten variant field round-trip. Columns: `_type`, `id`, `p_x`, `p_y`.
/// @param df data.frame from `re_loc_align(...)`.
/// @export
#[miniextendr]
pub fn re_loc_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<RELoc> = <Vec<RELoc>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// Produce the writer output for a mixed RELoc frame. @export
#[miniextendr]
pub fn re_loc_align() -> SEXP {
    vec![
        RELoc::At {
            id: 1,
            p: REPoint { x: 1.0, y: 2.0 },
        },
        RELoc::Nowhere { id: 2 },
        RELoc::At {
            id: 3,
            p: REPoint { x: -1.0, y: 0.5 },
        },
    ]
    .into_dataframe()
    .unwrap()
    .into_sexp()
}

/// Parallel RELoc round-trip (delegates to sequential due to Struct field). @export
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn re_loc_roundtrip_par(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<RELoc> = <Vec<RELoc>>::from_dataframe_par(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

// endregion

// endregion
