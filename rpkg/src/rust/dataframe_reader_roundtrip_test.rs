//! Round-trip fixtures for the non-scalar `FromDataFrame` readers (#782, #809).
//!
//! `#[derive(DataFrameRow)]` now generates an R→Rust reader
//! (`try_from_dataframe` / `_par`, surfaced as `Vec::<Row>::from_dataframe(&df)`)
//! for the struct-path shapes that previously had only a *writer*:
//!
//!   - column expansion: `[T; N]`, `Vec<T>` + `width`, `Vec<T>`/`Box<[T]>` + `expand`
//!   - struct-flatten: nested `DataFrameRow` fields (`<field>_<inner>` prefix),
//!     including recursion through several levels
//!   - opaque list-columns: un-annotated `Vec<scalar>` / `Box<[scalar]>` fields
//!     stored as VECSXP list-columns; each row's element is deserialized via
//!     `Vec<elem>: TryFromSexp` and `.into()`-converted to the field container type
//!
//! Each `*_roundtrip(df)` reads a `data.frame` into `Vec<Row>` with the reader,
//! then rebuilds it with the writer — so `roundtrip(make()) == make()` proves the
//! reader reconstructs rows that re-serialise to the identical frame. The R-side
//! assertions in `tests/testthat/test-dataframe-readers.R` also pin the column
//! values directly, anchoring the ground truth.
//!
//! The struct-flatten reader selects the `<field>_`-prefixed sub-columns into a
//! fresh sub-frame (an R allocation) before recursing, so it carries the no-arg
//! `gc_stress_reader_*` fixtures for the fast `gctorture(TRUE)` sweep, per
//! `rpkg/CLAUDE.md`'s SEXP-storage convention. The list-column reader does per-row
//! R access in a loop, so it also ships a no-arg `gc_stress_reader_list_column`
//! fixture.

#![allow(dead_code)]

use miniextendr_api::dataframe::{DataFrame, FromDataFrame, IntoDataFrame};
use miniextendr_api::{DataFrameRow, IntoList, IntoR, SEXP, miniextendr};

// region: row types — one per reader shape

// region: opaque list-column row types (#809)

/// Opaque list-column `Vec<f64>` (un-annotated) → single list-column `data`.
#[derive(Clone, Debug, PartialEq, IntoList, DataFrameRow)]
pub struct RListVecRow {
    pub id: i32,
    pub data: Vec<f64>,
}

/// Opaque list-column `Box<[i32]>` → exercises `.into()` to the boxed slice.
///
/// `IntoList` is manual because `Box<[T]>` has no blanket `IntoR` impl that
/// `#[derive(IntoList)]` can use directly (`into_vec().into_sexp()` is needed).
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
pub struct RListBoxRow {
    pub tag: String,
    pub xs: Box<[i32]>,
}

impl ::miniextendr_api::list::IntoList for RListBoxRow {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        ::miniextendr_api::List::from_raw_pairs(vec![
            ("tag", self.tag.into_sexp()),
            ("xs", self.xs.into_vec().into_sexp()),
        ])
    }
}

/// Two list-columns of differing element types in one row (`Vec<i32>` + `Vec<String>`).
#[derive(Clone, Debug, PartialEq, IntoList, DataFrameRow)]
pub struct RListMultiRow {
    pub ids: Vec<i32>,
    pub names: Vec<String>,
}

// endregion

/// Fixed-array expansion: `coords: [f64; 3]` → columns `coords_1..coords_3`.
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
pub struct RFixedRow {
    pub id: i32,
    pub coords: [f64; 3],
}

/// Pinned-width `Vec<f64>` expansion → columns `scores_1..scores_3` (`Option`).
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
pub struct RPinnedRow {
    pub name: String,
    #[dataframe(width = 3)]
    pub scores: Vec<f64>,
}

/// Pinned-width `Box<[f64]>` expansion → exercises the reader's `.into()`
/// conversion from the collected `Vec<f64>` back to `Box<[f64]>`.
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
pub struct RBoxPinnedRow {
    pub k: i32,
    #[dataframe(width = 2)]
    pub vals: Box<[f64]>,
}

/// Auto-expand `Vec<f64>` → runtime column count `values_1..values_k`.
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
pub struct RAutoRow {
    pub name: String,
    #[dataframe(expand)]
    pub values: Vec<f64>,
}

/// Auto-expand `Box<[i32]>` → runtime column count, exercises `.into()` on the
/// boxed-slice container.
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
pub struct RAutoBoxRow {
    pub tag: String,
    #[dataframe(expand)]
    pub xs: Box<[i32]>,
}

/// Scalar inner type for struct-flatten. Needs `IntoList` (pure-scalar shape).
#[derive(Clone, Debug, PartialEq, DataFrameRow, IntoList)]
pub struct RInner {
    pub x: f64,
    pub y: f64,
}

/// Basic struct-flatten: `origin: RInner` → columns `origin_x`, `origin_y`.
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
pub struct ROuter {
    pub id: i32,
    pub origin: RInner,
}

/// Scalar inner with a mixed (`String` + `i32`) column set.
#[derive(Clone, Debug, PartialEq, DataFrameRow, IntoList)]
pub struct RNamed {
    pub label: String,
    pub age: i32,
}

/// Struct-flatten with a non-numeric inner column → `owner_label`, `owner_age`.
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
pub struct RTaggedOuter {
    pub id: i32,
    pub owner: RNamed,
}

/// Innermost scalar type for the recursive-flatten case.
#[derive(Clone, Debug, PartialEq, DataFrameRow, IntoList)]
pub struct RLeaf {
    pub z: f64,
}

/// Middle type — itself struct-flattens `RLeaf` (so its own reader recurses).
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
pub struct RMid {
    pub a: f64,
    pub leaf: RLeaf,
}

/// Three-level nested flatten: `id`, `mid_a`, `mid_leaf_z`.
#[derive(Clone, Debug, PartialEq, DataFrameRow)]
pub struct RNestOuter {
    pub id: i32,
    pub mid: RMid,
}

// endregion

// region: round-trip entrypoints (read → rebuild)

/// `Vec::<RFixedRow>::from_dataframe(&df)` → rebuild. Columns `id`, `coords_1..3`.
/// @param df data.frame with `id` and `coords_1`/`coords_2`/`coords_3`.
/// @export
#[miniextendr]
pub fn reader_fixed_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<RFixedRow> = <Vec<RFixedRow>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// Pinned-width `Vec<f64>` round-trip. Columns `name`, `scores_1..3`.
/// @param df data.frame with `name` and `scores_1`/`scores_2`/`scores_3`.
/// @export
#[miniextendr]
pub fn reader_pinned_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<RPinnedRow> = <Vec<RPinnedRow>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// Pinned-width `Box<[f64]>` round-trip. Columns `k`, `vals_1`, `vals_2`.
/// @param df data.frame with `k` and `vals_1`/`vals_2`.
/// @export
#[miniextendr]
pub fn reader_box_pinned_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<RBoxPinnedRow> = <Vec<RBoxPinnedRow>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// Auto-expand `Vec<f64>` round-trip. Columns `name`, `values_1..values_k`.
/// @param df data.frame with `name` and `values_*` columns.
/// @export
#[miniextendr]
pub fn reader_auto_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<RAutoRow> = <Vec<RAutoRow>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// Auto-expand `Box<[i32]>` round-trip. Columns `tag`, `xs_1..xs_k`.
/// @param df data.frame with `tag` and `xs_*` columns.
/// @export
#[miniextendr]
pub fn reader_auto_box_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<RAutoBoxRow> = <Vec<RAutoBoxRow>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// Struct-flatten round-trip. Columns `id`, `origin_x`, `origin_y`.
/// @param df data.frame with `id`, `origin_x`, `origin_y`.
/// @export
#[miniextendr]
pub fn reader_flatten_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<ROuter> = <Vec<ROuter>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// Struct-flatten with a `String` inner column. Columns `id`, `owner_label`, `owner_age`.
/// @param df data.frame with `id`, `owner_label`, `owner_age`.
/// @export
#[miniextendr]
pub fn reader_flatten_mixed_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<RTaggedOuter> = <Vec<RTaggedOuter>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// Recursive (three-level) struct-flatten round-trip. Columns `id`, `mid_a`, `mid_leaf_z`.
/// @param df data.frame with `id`, `mid_a`, `mid_leaf_z`.
/// @export
#[miniextendr]
pub fn reader_flatten_nested_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<RNestOuter> = <Vec<RNestOuter>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

// endregion

// region: opaque list-column round-trip entrypoints (#809)

/// `Vec::<RListVecRow>::from_dataframe(&df)` → rebuild. Columns `id`, `data` (list-column).
/// @param df data.frame with `id` (integer) and `data` (list of numeric vectors).
/// @export
#[miniextendr]
pub fn reader_list_vec_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<RListVecRow> = <Vec<RListVecRow>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// `Vec::<RListBoxRow>::from_dataframe(&df)` → rebuild. Columns `tag`, `xs` (list-column).
/// Exercises `.into()` from `Vec<i32>` to `Box<[i32]>` per row.
/// @param df data.frame with `tag` (character) and `xs` (list of integer vectors).
/// @export
#[miniextendr]
pub fn reader_list_box_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<RListBoxRow> = <Vec<RListBoxRow>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// `Vec::<RListMultiRow>::from_dataframe(&df)` → rebuild. Columns `ids`, `names` (both list-columns).
/// @param df data.frame with `ids` (list of integer vectors) and `names` (list of character vectors).
/// @export
#[miniextendr]
pub fn reader_list_multi_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<RListMultiRow> = <Vec<RListMultiRow>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// Parallel list-column round-trip (real off-thread index assembly). Columns `id`, `data`.
/// @param df data.frame with `id` (integer) and `data` (list of numeric vectors).
/// @export
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn reader_list_vec_roundtrip_par(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<RListVecRow> = <Vec<RListVecRow>>::from_dataframe_par(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

// endregion

// region: gctorture fixtures (no-arg, self-contained)
//
// The struct-flatten reader allocates a fresh sub-frame (`select` + `strip_prefix`)
// per nested field and holds it (protected) across the recursive read, so it must
// survive a `gctorture(TRUE)` sweep. These build a frame with the writer, then read
// it straight back with the reader while the assembled frame SEXP is held live.

/// Drives the basic struct-flatten reader under gctorture.
/// @export
#[miniextendr]
pub fn gc_stress_reader_struct_flatten() {
    let rows: Vec<ROuter> = (0..16)
        .map(|i| ROuter {
            id: i,
            origin: RInner {
                x: i as f64,
                y: (i as f64) * 2.0,
            },
        })
        .collect();
    let df = rows.clone().into_dataframe().unwrap();
    let _back: Vec<ROuter> = <Vec<ROuter>>::from_dataframe(&df).unwrap();
    let _ = df;
}

/// Drives the recursive (three-level) struct-flatten reader under gctorture.
/// @export
#[miniextendr]
pub fn gc_stress_reader_nested_flatten() {
    let rows: Vec<RNestOuter> = (0..16)
        .map(|i| RNestOuter {
            id: i,
            mid: RMid {
                a: i as f64,
                leaf: RLeaf {
                    z: (i as f64) * 10.0,
                },
            },
        })
        .collect();
    let df = rows.clone().into_dataframe().unwrap();
    let _back: Vec<RNestOuter> = <Vec<RNestOuter>>::from_dataframe(&df).unwrap();
    let _ = df;
}

/// Drives the list-column reader under gctorture. The reader does per-row R access in
/// a loop, so it must survive `gctorture(TRUE)` with SEXP elements protected correctly.
/// @export
#[miniextendr]
pub fn gc_stress_reader_list_column() {
    let rows: Vec<RListVecRow> = (0..16)
        .map(|i| RListVecRow {
            id: i,
            data: vec![i as f64, (i as f64) * 2.0],
        })
        .collect();
    let df = rows.clone().into_dataframe().unwrap();
    let _back: Vec<RListVecRow> = <Vec<RListVecRow>>::from_dataframe(&df).unwrap();
    let _ = df;
}

// endregion

// region: parallel reader entrypoints (feature = "rayon")
//
// `from_dataframe_par` exercises `try_from_dataframe_par`. Column-expansion shapes
// take the genuine off-thread index-assembly path; struct-flatten shapes delegate
// to the sequential reader (to avoid imposing `Inner: Clone`) — both are covered.

/// Parallel fixed-array round-trip (real off-thread assembly). Columns `id`, `coords_1..3`.
/// @param df data.frame with `id` and `coords_1`/`coords_2`/`coords_3`.
/// @export
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn reader_fixed_roundtrip_par(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<RFixedRow> = <Vec<RFixedRow>>::from_dataframe_par(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// Parallel struct-flatten round-trip (delegates to sequential). Columns `id`, `origin_x`, `origin_y`.
/// @param df data.frame with `id`, `origin_x`, `origin_y`.
/// @export
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn reader_flatten_roundtrip_par(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<ROuter> = <Vec<ROuter>>::from_dataframe_par(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

// endregion
