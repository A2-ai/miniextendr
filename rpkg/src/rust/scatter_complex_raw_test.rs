//! Test fixtures for `scatter_column` with `CPLXSXP` and `RAWSXP` columns (issue #902).
//!
//! ## Background
//!
//! [`scatter_column`][miniextendr_api::convert::scatter_column] is invoked
//! when an enum derives `DataFrameRow` and one variant uses a *struct-typed
//! field* (struct-flatten). When other variants are present for some rows the
//! inner struct's pre-built columns must be scattered back to the full
//! row-count with type-appropriate fill:
//!
//! - CPLXSXP absent rows → `NA_complex_` (both parts `NA_real_`)
//! - RAWSXP  absent rows → `0x00` (`as.raw(0)`) — R raw has no NA concept
//!
//! PR #831 refactored `scatter_column` to cover these branches but no
//! end-to-end test verified the output.
//!
//! ## Test design
//!
//! ### Raw (RAWSXP) — unconditional
//!
//! `RawPayload` — a one-field `DataFrameRow` struct with `byte: u8`.
//! `ScatterRawEvent` — a two-variant enum where `WithPayload` carries a
//! `data: RawPayload` struct-flatten field and `Empty` has no raw column.
//! Mixed rows trigger `scatter_column` for the RAWSXP `data_byte` column.
//!
//! ### Complex (CPLXSXP) — requires `num-complex` feature
//!
//! `ComplexPayload` — a one-field `DataFrameRow` struct with
//! `value: Complex<f64>`.  `Complex<f64>` has angle-bracket generic args, so
//! the `DataFrameRow` macro classifies it as `Scalar` (not struct-flatten)
//! and stores it as `Vec<Complex<f64>>` in the companion DataFrame; its
//! `IntoR` impl produces a `CPLXSXP` vector.
//! `ScatterComplexEvent` — like `ScatterRawEvent` but for CPLXSXP.
//!
//! R-side assertions live in
//! `rpkg/tests/testthat/test-scatter-complex-raw.R`.

#![allow(dead_code)]

use miniextendr_api::{DataFrame, DataFrameRow, IntoDataFrame, IntoList, miniextendr};

// region: Raw scatter — RAWSXP (unconditional)

/// Inner struct producing a RAWSXP column (`byte: u8`).
///
/// Used as a struct-flatten field in `ScatterRawEvent::WithPayload` so that
/// `scatter_column` receives a RAWSXP when `Empty` rows are present.
#[derive(Clone, Debug, DataFrameRow, IntoList)]
pub struct RawPayload {
    /// RAWSXP column: absent rows → `as.raw(0)` (no NA in R raw).
    pub byte: u8,
}

/// Two-variant enum: `WithPayload` carries a struct-flatten `data: RawPayload`
/// field; `Empty` does not. When rows of both variants are present
/// `scatter_column` is called for the `data_byte` RAWSXP column, filling
/// absent rows with `0x00`.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum ScatterRawEvent {
    /// Contains a `data: RawPayload` struct-flatten field producing a
    /// `data_byte` RAWSXP column.
    WithPayload {
        id: i32,
        data: RawPayload,
    },
    /// No raw payload — absent rows produce `as.raw(0)` in `data_byte`.
    Empty {
        id: i32,
    },
}

/// Returns a 5-row data.frame with interleaved `WithPayload` and `Empty` rows.
///
/// Row layout:
///   row 1: WithPayload — data_byte = 0xFF
///   row 2: Empty       — data_byte = as.raw(0)
///   row 3: WithPayload — data_byte = 0x01
///   row 4: Empty       — data_byte = as.raw(0)
///   row 5: WithPayload — data_byte = 0x7F
///
/// R-side assertions: rows 1/3/5 have the expected byte values;
/// rows 2/4 are `as.raw(0)`.
#[miniextendr]
pub fn scatter_raw_mixed() -> DataFrame {
    vec![
        ScatterRawEvent::WithPayload {
            id: 1,
            data: RawPayload { byte: 0xFF },
        },
        ScatterRawEvent::Empty { id: 2 },
        ScatterRawEvent::WithPayload {
            id: 3,
            data: RawPayload { byte: 0x01 },
        },
        ScatterRawEvent::Empty { id: 4 },
        ScatterRawEvent::WithPayload {
            id: 5,
            data: RawPayload { byte: 0x7F },
        },
    ]
    .into_dataframe()
    .unwrap()
}

/// All rows present — exercises the dense (no-scatter-zero) RAWSXP path.
#[miniextendr]
pub fn scatter_raw_all_present() -> DataFrame {
    vec![
        ScatterRawEvent::WithPayload {
            id: 1,
            data: RawPayload { byte: 0xAB },
        },
        ScatterRawEvent::WithPayload {
            id: 2,
            data: RawPayload { byte: 0xCD },
        },
        ScatterRawEvent::WithPayload {
            id: 3,
            data: RawPayload { byte: 0xEF },
        },
    ]
    .into_dataframe()
    .unwrap()
}

/// All rows absent — every cell in the `data_byte` column should be
/// `as.raw(0)`.
#[miniextendr]
pub fn scatter_raw_all_absent() -> DataFrame {
    vec![
        ScatterRawEvent::Empty { id: 1 },
        ScatterRawEvent::Empty { id: 2 },
        ScatterRawEvent::Empty { id: 3 },
    ]
    .into_dataframe()
    .unwrap()
}

/// Exercise the RAWSXP `scatter_column` path under GC pressure.
///
/// Allocates a `ScatterRawEvent` batch (interleaved present/absent rows),
/// calls `into_dataframe()`, and converts to SEXP, driving the RAWSXP branch
/// of `scatter_native` which fills absent rows with `0x00`.
///
/// No arguments — suitable for the fast `gctorture(TRUE)` no-arg sweep.
#[miniextendr]
pub fn gc_stress_scatter_raw() {
    use miniextendr_api::into_r::IntoR as _;

    let rows: Vec<ScatterRawEvent> = (0i32..32)
        .map(|i| {
            if i % 3 == 0 {
                ScatterRawEvent::WithPayload {
                    id: i,
                    data: RawPayload {
                        byte: (i as u8).wrapping_mul(7),
                    },
                }
            } else {
                ScatterRawEvent::Empty { id: i }
            }
        })
        .collect();
    let _ = rows.into_dataframe().unwrap().into_sexp();
}

// endregion

// region: Complex scatter — CPLXSXP (requires num-complex feature)

#[cfg(feature = "num-complex")]
use miniextendr_api::num_complex_impl::Complex;

/// Inner struct producing a CPLXSXP column (`value: Complex<f64>`).
///
/// `Complex<f64>` is a generic-arg type so the `DataFrameRow` macro
/// classifies it as `Scalar`, storing it as `Vec<Complex<f64>>` in the
/// companion DataFrame. `Vec<Complex<f64>>::into_sexp()` → CPLXSXP.
///
/// Used as a struct-flatten field in `ScatterComplexEvent::WithPayload` so
/// that `scatter_column` receives a CPLXSXP when `Empty` rows are present.
#[cfg(feature = "num-complex")]
#[derive(Clone, Debug, DataFrameRow, IntoList)]
pub struct ComplexPayload {
    /// CPLXSXP column: absent rows → `NA_complex_`.
    pub value: Complex<f64>,
}

/// Two-variant enum: `WithPayload` carries a struct-flatten `data: ComplexPayload`
/// field; `Empty` does not. Mixed rows trigger `scatter_column` for the
/// `data_value` CPLXSXP column, filling absent rows with `NA_complex_`.
#[cfg(feature = "num-complex")]
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum ScatterComplexEvent {
    /// Contains a `data: ComplexPayload` struct-flatten field producing a
    /// `data_value` CPLXSXP column.
    WithPayload {
        id: i32,
        data: ComplexPayload,
    },
    /// No complex payload — absent rows produce `NA_complex_` in `data_value`.
    Empty {
        id: i32,
    },
}

/// Returns a 5-row data.frame with interleaved `WithPayload` and `Empty` rows.
///
/// Row layout:
///   row 1: WithPayload — data_value = 1.0+2.0i
///   row 2: Empty       — data_value = NA_complex_
///   row 3: WithPayload — data_value = -0.5+0.5i
///   row 4: Empty       — data_value = NA_complex_
///   row 5: WithPayload — data_value = 0.0+0.0i  (both parts zero, not NA)
///
/// R-side assertions: rows 1/3/5 have correct `Re`/`Im`; rows 2/4 are
/// `NA_complex_`.
#[cfg(feature = "num-complex")]
#[miniextendr]
pub fn scatter_complex_mixed() -> DataFrame {
    vec![
        ScatterComplexEvent::WithPayload {
            id: 1,
            data: ComplexPayload {
                value: Complex::new(1.0, 2.0),
            },
        },
        ScatterComplexEvent::Empty { id: 2 },
        ScatterComplexEvent::WithPayload {
            id: 3,
            data: ComplexPayload {
                value: Complex::new(-0.5, 0.5),
            },
        },
        ScatterComplexEvent::Empty { id: 4 },
        ScatterComplexEvent::WithPayload {
            id: 5,
            data: ComplexPayload {
                value: Complex::new(0.0, 0.0),
            },
        },
    ]
    .into_dataframe()
    .unwrap()
}

/// All rows present — exercises the dense (no-scatter-NA) CPLXSXP path.
#[cfg(feature = "num-complex")]
#[miniextendr]
pub fn scatter_complex_all_present() -> DataFrame {
    vec![
        ScatterComplexEvent::WithPayload {
            id: 1,
            data: ComplexPayload {
                value: Complex::new(3.0, 4.0),
            },
        },
        ScatterComplexEvent::WithPayload {
            id: 2,
            data: ComplexPayload {
                value: Complex::new(-1.0, -1.0),
            },
        },
        ScatterComplexEvent::WithPayload {
            id: 3,
            data: ComplexPayload {
                value: Complex::new(0.0, 1.0),
            },
        },
    ]
    .into_dataframe()
    .unwrap()
}

/// All rows absent — every cell in the `data_value` column should be
/// `NA_complex_`.
#[cfg(feature = "num-complex")]
#[miniextendr]
pub fn scatter_complex_all_absent() -> DataFrame {
    vec![
        ScatterComplexEvent::Empty { id: 1 },
        ScatterComplexEvent::Empty { id: 2 },
        ScatterComplexEvent::Empty { id: 3 },
    ]
    .into_dataframe()
    .unwrap()
}

/// Exercise the CPLXSXP `scatter_column` path under GC pressure.
///
/// Allocates a `ScatterComplexEvent` batch (interleaved present/absent rows),
/// calls `into_dataframe()`, and converts to SEXP, driving the CPLXSXP branch
/// of `scatter_native` which fills absent rows with `NA_complex_`.
///
/// No arguments — suitable for the fast `gctorture(TRUE)` no-arg sweep.
#[cfg(feature = "num-complex")]
#[miniextendr]
pub fn gc_stress_scatter_complex() {
    use miniextendr_api::into_r::IntoR as _;

    let rows: Vec<ScatterComplexEvent> = (0i32..32)
        .map(|i| {
            if i % 2 == 0 {
                ScatterComplexEvent::WithPayload {
                    id: i,
                    data: ComplexPayload {
                        value: Complex::new(i as f64, (i as f64) * -0.5),
                    },
                }
            } else {
                ScatterComplexEvent::Empty { id: i }
            }
        })
        .collect();
    let _ = rows.into_dataframe().unwrap().into_sexp();
}

// endregion
