//! Declared R formals (#1842): `#[miniextendr(formal = "row.names")]` on a
//! parameter, or `formal(p = "row.names")` on an impl method, gives it an R
//! name that is no Rust identifier. R then matches the argument by name,
//! partial name or position, like any formal.
//!
//! `mx_formal_fit` is a classed list made in R
//! (`structure(list(estimate = 2, se = 0.5), class = "mx_formal_fit")`); only
//! its methods live here. The Rust parameter names start with `fml_` so that
//! a test can tell that none of them reaches the R side.

use miniextendr_api::{
    BuiltDataFrame, DataFrameRow, IntoDataFrame, List, MatchArg, Missing, miniextendr,
};

// region: S3 methods of base generics with dotted arguments

#[derive(Clone, Debug, DataFrameRow)]
pub struct MxFormalRow {
    pub estimate: f64,
    pub row_name: Option<String>,
    pub optional: bool,
}

/// A fit as a one-row data frame, which records the `row.names` and
/// `optional` it was given.
///
/// @param x A `mx_formal_fit`.
/// @param row.names `NULL`, or the name of the one row.
/// @param optional Whether names are optional; recorded only.
/// @param ... Ignored.
/// @export
#[miniextendr(s3(generic = "as.data.frame", class = "mx_formal_fit"))]
pub fn as_data_frame_mx_formal_fit(
    x: List,
    #[miniextendr(formal = "row.names", default = "NULL")] fml_row_names: Option<String>,
    #[miniextendr(formal = "optional", default = "FALSE")] fml_optional: bool,
    _dots: ...
) -> BuiltDataFrame {
    let estimate: f64 = x.get_named("estimate").unwrap_or(f64::NAN);
    vec![MxFormalRow {
        estimate,
        row_name: fml_row_names,
        optional: fml_optional,
    }]
    .into_dataframe()
    .expect("one row")
}

/// The estimate of a fit, with its confidence interval when `conf.int`.
///
/// @param object A `mx_formal_fit`.
/// @param conf.int Whether to add the interval.
/// @param conf.level Its level.
/// @param ... Ignored.
/// @export
#[miniextendr(s3(generic = "summary", class = "mx_formal_fit"))]
pub fn summary_mx_formal_fit(
    object: List,
    #[miniextendr(formal = "conf.int", default = "FALSE")] fml_conf_int: bool,
    #[miniextendr(formal = "conf.level", default = "0.95", no_na)] fml_conf_level: f64,
    _dots: ...
) -> Vec<f64> {
    let estimate: f64 = object.get_named("estimate").unwrap_or(f64::NAN);
    if !fml_conf_int {
        return vec![estimate];
    }
    let se: f64 = object.get_named("se").unwrap_or(f64::NAN);
    // The normal quantile of the two levels the tests use.
    let z = if fml_conf_level >= 0.99 {
        2.575829
    } else {
        1.959964
    };
    vec![estimate, estimate - z * se, estimate + z * se]
}

// endregion

// region: standalone functions

/// The mean of `x`, dropping `NA` when `na.rm`.
///
/// @param x Numbers.
/// @param na.rm Whether to drop `NA`; `NA` itself is refused.
#[miniextendr(internal)]
pub fn mx_formal_mean(
    x: Vec<f64>,
    #[miniextendr(formal = "na.rm", default = "FALSE", no_na)] fml_na_rm: bool,
) -> f64 {
    let (sum, n) = x
        .into_iter()
        .filter(|v| !(fml_na_rm && v.is_nan()))
        .fold((0.0, 0.0), |(sum, n), v| (sum + v, n + 1.0));
    sum / n
}

/// `x` capped at `cap.at`, or unchanged when it is omitted. The default
/// only shows in the usage line (#2161): an omitted `cap.at` is
/// `Missing::Absent`.
///
/// @param x Numbers.
/// @param cap.at The cap.
#[miniextendr(internal)]
pub fn mx_formal_cap(
    x: Vec<f64>,
    #[miniextendr(formal = "cap.at", default = "Inf")] fml_cap_at: Missing<f64>,
) -> Vec<f64> {
    match fml_cap_at.into_option() {
        Some(cap) => x.into_iter().map(|v| v.min(cap)).collect(),
        None => x,
    }
}

/// How an interval is computed.
#[derive(Copy, Clone, Debug, PartialEq, MatchArg)]
#[match_arg(rename_all = "lower")]
pub enum MxFormalCiMethod {
    Wald,
    Profile,
}

/// The method `ci.method` was matched to, and the `out.type` chosen.
///
/// @param ci.method How the interval is computed.
/// @param out.type What to return.
#[miniextendr(internal)]
pub fn mx_formal_choice(
    #[miniextendr(formal = "ci.method", match_arg)] fml_ci_method: MxFormalCiMethod,
    #[miniextendr(formal = "out.type", choices("vector", "list"))] fml_out_type: String,
) -> String {
    format!("{fml_ci_method:?}/{fml_out_type}")
}

/// `n.boot` doubled; no R-side check, so a bad value is refused by the
/// conversion in Rust.
///
/// @param n.boot A count.
#[miniextendr(internal)]
pub fn mx_formal_unchecked(
    #[miniextendr(formal = "n.boot", no_preconditions)] fml_n_boot: i32,
) -> i32 {
    fml_n_boot.saturating_mul(2)
}

// endregion

// region: impl-block methods

/// A running total whose methods take dotted arguments.
#[derive(miniextendr_api::ExternalPtr)]
pub struct MxFormalTally {
    total: f64,
}

#[miniextendr(r6)]
impl MxFormalTally {
    /// @param start.value The initial total.
    #[miniextendr(formal(fml_start = "start.value"), defaults(fml_start = "0"))]
    pub fn new(fml_start: f64) -> Self {
        MxFormalTally { total: fml_start }
    }

    /// Add `by.value` (skipped when it is `NA` and `na.rm`); returns the total.
    /// @param by.value The amount.
    /// @param na.rm Whether an `NA` amount is skipped.
    #[miniextendr(
        formal(fml_by = "by.value", fml_na_rm = "na.rm"),
        defaults(fml_na_rm = "FALSE")
    )]
    pub fn add(&mut self, fml_by: f64, fml_na_rm: bool) -> f64 {
        if !(fml_na_rm && fml_by.is_nan()) {
            self.total += fml_by;
        }
        self.total
    }
}

/// An estimate with a standard error, as an S7 class.
#[derive(miniextendr_api::ExternalPtr)]
pub struct MxFormalEstimate {
    estimate: f64,
    se: f64,
}

#[miniextendr(s7)]
impl MxFormalEstimate {
    /// @param estimate The estimate.
    /// @param std.error Its standard error.
    #[miniextendr(formal(fml_se = "std.error"))]
    pub fn new(estimate: f64, fml_se: f64) -> Self {
        MxFormalEstimate {
            estimate,
            se: fml_se,
        }
    }

    /// The interval at `conf.level`.
    /// @param conf.level The level, 0.95 or 0.99.
    #[miniextendr(
        formal(fml_conf_level = "conf.level"),
        defaults(fml_conf_level = "0.95")
    )]
    pub fn formal_interval(&self, fml_conf_level: f64) -> Vec<f64> {
        let z = if fml_conf_level >= 0.99 {
            2.575829
        } else {
            1.959964
        };
        vec![self.estimate - z * self.se, self.estimate + z * self.se]
    }
}

// endregion
