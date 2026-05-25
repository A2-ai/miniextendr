//! Fixtures for the `typed_dataframe!` macro (#698).
//!
//! Declares a `TheophDf`-style typed wrapper for an R data.frame and exposes
//! `#[miniextendr]` entry points that exercise the per-column accessors plus
//! the validating `TryFromSexp` impl.

use miniextendr_api::{miniextendr, typed_dataframe};

typed_dataframe! {
    /// Theophylline PK observation shape.
    ///
    /// Required columns: `subject` (integer), `weight` (numeric),
    /// `dose` (numeric), `time` (numeric), `conc` (numeric). The optional
    /// `flag` column, when present, must be integer.
    pub TheophDf {
        /// Subject ID (integer).
        subject: i32,
        /// Subject weight in kg.
        weight: f64,
        /// Administered dose.
        dose: f64,
        /// Sample time in hours.
        time: f64,
        /// Measured concentration.
        conc: f64,
        /// Optional per-row flag (e.g. censoring marker).
        flag: Option<i32>,
    }
}

/// Number of rows in a Theoph-shaped data.frame.
///
/// @param df A data.frame with columns `subject`, `weight`, `dose`, `time`,
///   `conc`, and optionally `flag`. Type mismatches and missing required
///   columns are batched into a single error.
/// @return Integer scalar — the row count.
/// @export
#[miniextendr]
pub fn typed_df_theoph_nrow(df: TheophDf) -> i32 {
    // `nrow` is computed once at `try_from_sexp` time; the per-column slices
    // share that length thanks to the data.frame invariant.
    df.nrow() as i32
}

/// Sum the `conc` column of a Theoph-shaped data.frame.
///
/// @param df A data.frame matching the `TheophDf` shape.
/// @return Numeric scalar — `sum(df$conc)`.
/// @export
#[miniextendr]
pub fn typed_df_theoph_sum_conc(df: TheophDf) -> f64 {
    df.conc().iter().copied().sum()
}

/// Whether the optional `flag` column was present.
///
/// @param df A data.frame matching the `TheophDf` shape.
/// @return Logical scalar.
/// @export
#[miniextendr]
pub fn typed_df_theoph_has_flag(df: TheophDf) -> bool {
    df.flag().is_some()
}

/// Sum of the optional `flag` column, or `-1` if absent.
///
/// @param df A data.frame matching the `TheophDf` shape; `flag` is optional.
/// @return Integer scalar.
/// @export
#[miniextendr]
pub fn typed_df_theoph_flag_sum(df: TheophDf) -> i32 {
    match df.flag() {
        Some(flag) => flag.iter().copied().sum(),
        None => -1,
    }
}

// Strict-mode (no extra columns) variant for exercising `@exact;`.
typed_dataframe! {
    @exact;
    /// Strict variant — no columns beyond `x` are allowed.
    pub TypedDfStrict {
        x: i32,
    }
}

/// Sum of the `x` column under strict-mode validation.
///
/// @param df A data.frame containing exactly one integer column `x`.
/// @return Integer scalar.
/// @export
#[miniextendr]
pub fn typed_df_strict_sum_x(df: TypedDfStrict) -> i32 {
    df.x().iter().copied().sum()
}
