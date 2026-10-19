//! `#[miniextendr(dots = empty | empty(allow = [..]) | forward(allow = [..]))]`
//! (#2157): the generated wrapper refuses what `...` doesn't take before it
//! evaluates anything, with a "did you mean" suggestion.
//!
//! `mx_dots_fit` is a classed list made in R
//! (`structure(list(estimate = 2), class = "mx_dots_fit")`); only its methods
//! live here.

use miniextendr_api::dots::Dots;
use miniextendr_api::{BuiltDataFrame, DataFrameRow, IntoDataFrame, List, miniextendr};

// region: standalone functions

/// Format a fit's estimate.
///
/// @param x A `mx_dots_fit`.
/// @param digits Significant digits.
/// @param ... Must be empty.
/// @export
#[miniextendr(s3(generic = "format", class = "mx_dots_fit"), dots = empty)]
pub fn format_mx_dots_fit(
    x: List,
    #[miniextendr(default = "3L")] digits: i32,
    _dots: ...
) -> String {
    let estimate: f64 = x.get_named("estimate").unwrap_or(f64::NAN);
    let digits = usize::try_from(digits).unwrap_or(0);
    format!("estimate {estimate:.digits$}")
}

#[derive(Clone, Debug, DataFrameRow)]
pub struct MxDotsRow {
    pub estimate: f64,
}

/// A fit as a one-row data frame. `data.frame()` calls this with
/// `optional = TRUE, stringsAsFactors = FALSE`, so `...` takes those and
/// `row.names`, and ignores them.
///
/// @param x A `mx_dots_fit`.
/// @param ... `row.names`, `optional` and `stringsAsFactors`, ignored.
/// @export
#[miniextendr(
    s3(generic = "as.data.frame", class = "mx_dots_fit"),
    dots = empty(allow = ["row.names", "optional", "stringsAsFactors"])
)]
pub fn as_data_frame_mx_dots_fit(x: List, _dots: ...) -> BuiltDataFrame {
    let estimate: f64 = x.get_named("estimate").unwrap_or(f64::NAN);
    vec![MxDotsRow { estimate }]
        .into_dataframe()
        .expect("one row")
}

/// Values at `newdata`: a required argument after the fit, so a misspelt
/// `newdata` must be reported as such, not as the missing `newdata`.
///
/// @param object A `mx_dots_fit`.
/// @param newdata Points to predict at.
/// @param ... Must be empty.
/// @export
#[miniextendr(dots = empty)]
pub fn mx_dots_predict(object: List, newdata: Vec<f64>, _dots: ...) -> Vec<f64> {
    let estimate: f64 = object.get_named("estimate").unwrap_or(f64::NAN);
    newdata.iter().map(|x| x * estimate).collect()
}

/// `x * scale + shift`, with `scale` and `shift` passed in `...`; the
/// function reads them, the refusal class is the package's own.
///
/// @param x A number.
/// @param ... `scale` (default 1) and `shift` (default 0).
/// @export
#[miniextendr(dots = forward(allow = ["scale", "shift"], class = "mx_dots_unused"))]
pub fn mx_dots_forward(x: f64, rest: &Dots) -> f64 {
    let args = rest.as_list();
    let scale: f64 = args.get_named("scale").unwrap_or(1.0);
    let shift: f64 = args.get_named("shift").unwrap_or(0.0);
    x * scale + shift
}

/// The names `...` passed to the function: `dots = data` checks nothing.
///
/// @param ... Anything.
/// @export
#[miniextendr(dots = data)]
pub fn mx_dots_data(rest: &Dots) -> i32 {
    i32::try_from(rest.len()).unwrap_or(i32::MAX)
}

// endregion

// region: impl-block methods

/// A counter with an S3 `format()` method whose `...` is the generic's
/// dispatch `...`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct MxDotsCounter {
    n: i32,
}

#[miniextendr(s3)]
impl MxDotsCounter {
    /// @param n The count.
    pub fn new(n: i32) -> Self {
        MxDotsCounter { n }
    }

    /// The count, padded to `width`.
    /// @param width Field width; required.
    #[miniextendr(s3(generic = "format"), dots = empty)]
    pub fn format(&self, width: i32) -> String {
        let width = usize::try_from(width).unwrap_or(0);
        format!("{:>width$}", self.n)
    }
}

/// A running total whose `add()` reads `by` and `times` from `...`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct MxDotsTally {
    total: f64,
}

#[miniextendr(r6)]
impl MxDotsTally {
    pub fn new() -> Self {
        MxDotsTally { total: 0.0 }
    }

    /// Add `by` (default 1) `times` times (default 1); returns the total.
    /// @param ... `by` and `times`.
    #[miniextendr(dots = forward(allow = ["by", "times"]))]
    pub fn add(&mut self, rest: &Dots) -> f64 {
        let args = rest.as_list();
        let by: f64 = args.get_named("by").unwrap_or(1.0);
        let times: f64 = args.get_named("times").unwrap_or(1.0);
        self.total += by * times;
        self.total
    }
}

// endregion
