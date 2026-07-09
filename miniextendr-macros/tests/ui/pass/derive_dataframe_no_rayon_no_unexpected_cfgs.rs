//! Compile-pass regression guard for #1117.
//!
//! `#[derive(DataFrameRow)]` must not stamp a raw `#[cfg(feature = "rayon")]`
//! into the *consumer* crate. A `cfg` inside a derive is evaluated against the
//! destination crate, and a consumer that doesn't declare a `rayon` feature of
//! its own would then trip the `unexpected_cfgs` lint on every derive. The
//! parallel `*_par` methods are instead gated on `miniextendr-api`'s own
//! `rayon` feature through the `__dataframe_row_when_rayon!` shim (see
//! `miniextendr-api/src/dataframe.rs`).
//!
//! This crate declares no `rayon` feature and `#![deny(unexpected_cfgs)]`, so a
//! regression that re-introduced the raw `#[cfg(feature = "rayon")]` would fail
//! to compile here. Exercises both the struct-align and enum-align emission
//! paths (each carried its own `#[cfg(feature = "rayon")]` sites).

#![deny(unexpected_cfgs)]
#![allow(dead_code)]

use miniextendr_macros::{DataFrameRow, IntoList};

/// Scalar struct — emits `from_rows`/`from_rows_par` (builder) and
/// `try_from_dataframe`/`try_from_dataframe_par` (reader). Named structs also
/// need `IntoList` (the derive asserts `Row: IntoList`).
#[derive(IntoList, DataFrameRow)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub label: String,
}

/// Payload enum — exercises the enum-align builder/reader emission paths.
#[derive(DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum Event {
    Login { user_id: i32 },
    Logout { user_id: i32, reason: String },
}

fn main() {}
