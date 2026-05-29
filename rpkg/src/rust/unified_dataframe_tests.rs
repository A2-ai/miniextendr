//! End-to-end fixtures for the unified DataFrame surface: the public verbs
//! [`IntoDataFrame::into_dataframe`] (`rows.into_dataframe()?`) and
//! [`FromDataFrame::from_dataframe`] (`Vec::<Row>::from_dataframe(&df)?`), which
//! `#[derive(DataFrameRow)]` wires up via the local `DataFrameRowConvert` marker
//! and the `miniextendr_api` blanket impls (orphan-rule bridge).
//!
//! The round-trip fixtures take a `SEXP` argument so they run on the R main
//! thread, where the eager R allocation inside `into_dataframe()` (column
//! assembly) is safe.

use miniextendr_api::dataframe::{DataFrame, FromDataFrame, IntoDataFrame};
use miniextendr_api::{DataFrameRow, IntoList, IntoR, SEXP, miniextendr};

#[derive(Clone, Debug, IntoList, DataFrameRow)]
pub struct UnifiedPoint {
    pub x: f64,
    pub y: f64,
    pub label: String,
}

/// Round-trip a `data.frame` through the unified verbs: read into `Vec<Row>` with
/// the `from_dataframe` reader, then rebuild with the `into_dataframe` writer.
///
/// Returns a fresh `data.frame` identical to the input, proving both public verbs
/// compose. Takes a `SEXP` so it runs on the R main thread.
///
/// @param df A data.frame with numeric `x`, `y` and character `label` columns.
/// @export
#[miniextendr]
pub fn unified_roundtrip(df: SEXP) -> SEXP {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<UnifiedPoint> = <Vec<UnifiedPoint>>::from_dataframe(&frame).unwrap();
    rows.into_dataframe().unwrap().into_sexp()
}

/// Read a `data.frame` into `Vec<UnifiedPoint>` via the unified `from_dataframe`
/// reader and return the row count.
///
/// @param df A data.frame with numeric `x`, `y` and character `label` columns.
/// @export
#[miniextendr]
pub fn unified_roundtrip_count(df: SEXP) -> i32 {
    let frame = DataFrame::from_sexp(df).unwrap();
    let rows: Vec<UnifiedPoint> = <Vec<UnifiedPoint>>::from_dataframe(&frame).unwrap();
    rows.len() as i32
}

/// GC-stress fixture for the unified DataFrame verbs.
///
/// No-arg so the fast `gctorture(TRUE)` sweep over `rpkg`'s exports exercises it.
/// Builds rows, assembles a `DataFrame` with the `into_dataframe` writer (the
/// SEXP-allocating write path), then reads them back with the `from_dataframe`
/// reader while the assembled `DataFrame` SEXP is held live across the second
/// allocation — the window where a GC barrier bug would surface.
///
/// @export
#[miniextendr]
pub fn gc_stress_unified_dataframe() {
    let rows: Vec<UnifiedPoint> = (0..16)
        .map(|i| UnifiedPoint {
            x: i as f64,
            y: (i * 2) as f64,
            label: format!("p{i}"),
        })
        .collect();
    let df = rows.clone().into_dataframe().unwrap();
    let _back: Vec<UnifiedPoint> = <Vec<UnifiedPoint>>::from_dataframe(&df).unwrap();
    // Hold `df` (the assembled SEXP) live past the read-back allocation.
    let _ = df;
}
