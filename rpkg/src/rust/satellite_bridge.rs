//! Bridge layer: exposes the miniextendr-free [`satellite`] crate's serde types
//! to R. This is the *only* place that knows about both worlds. `satellite`
//! never mentions miniextendr; this module never adds conversion glue to
//! `satellite` — it relies purely on the types' derived serde impls flowing
//! through `miniextendr_api::serde`.
//!
//! Empirical question this answers: a third-party crate that derives serde, with
//! a *separate* `serde = "1"` dependency, gets full R interop here with zero
//! per-type code. Everything below is `satellite::T -> R` or `R -> satellite::T`
//! routed through the serde bridge.

use miniextendr_api::SEXP;
use miniextendr_api::dataframe::DataFrame;
use miniextendr_api::miniextendr;
use miniextendr_api::serde::{
    AsSerialize, DataFrameShape, SplitShape, from_r, vec_to_dataframe, vec_to_dataframe_split,
};

/// Rust → R, columnar: `Vec<Reading>` becomes a native data.frame, one atomic
/// column per field. `note: Option<String>` carries NA.
#[miniextendr]
pub fn satellite_readings_df() -> Result<DataFrame, String> {
    vec_to_dataframe(&satellite::sample_readings()).map_err(|e| e.to_string())
}

/// Nested-struct flatten: `Station.site` flattens to `site_lat` / `site_lon`.
/// Also probes i64 → numeric (`readings_taken`).
#[miniextendr]
pub fn satellite_stations_df() -> Result<DataFrame, String> {
    vec_to_dataframe(&satellite::sample_stations()).map_err(|e| e.to_string())
}

/// Rust → R, row-oriented: same data as a list-of-lists via `AsSerialize`.
#[miniextendr]
pub fn satellite_readings_list() -> AsSerialize<Vec<satellite::Reading>> {
    AsSerialize(satellite::sample_readings())
}

/// Enum split: `Vec<Event>` partitioned into one data.frame per variant.
#[miniextendr]
pub fn satellite_events_split() -> Result<DataFrameShape, String> {
    vec_to_dataframe_split(&satellite::sample_events(), SplitShape::PerVariantList)
        .map_err(|e| e.to_string())
}

/// R → Rust → R round-trip: deserialize an R list into `satellite::Reading`,
/// then serialize it straight back. Proves the satellite type is bidirectional
/// through the bridge with no hand-written conversion.
#[miniextendr]
pub fn satellite_echo_reading(x: SEXP) -> Result<AsSerialize<satellite::Reading>, String> {
    let r: satellite::Reading = from_r(x).map_err(|e| e.to_string())?;
    Ok(AsSerialize(r))
}
