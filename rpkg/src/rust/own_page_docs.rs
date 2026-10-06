//! Free functions of one file that each document their own help page
//! (#1289).
//!
//! A free function without `@rdname`, `@describeIn` or `@name` documents its
//! own page, as in roxygen2: `man/own_page_round.Rd` and
//! `man/own_page_truncate.Rd`. Both carry their own `@title`, and the second
//! inherits a section from the first with `@inheritSection`. Were both on one
//! `own_page_docs.Rd`, the inherited section would resolve to that merged page
//! itself: roxygen2 copies it once per block, already inherited copies
//! included, which grows a family of ten such functions into a page of
//! megabytes and stalls `R CMD INSTALL`. Grouping is opt-in through an
//! explicit `@rdname` (see `stem_page_docs.rs`).

use miniextendr_api::miniextendr;

/// @title Round values to a step
/// @description `own_page_round()` rounds each value to the nearest multiple
/// of `step`.
/// @section Step rules:
/// `step` must be positive. The result keeps the length of `values`.
/// @param values Numbers to round.
/// @param step Positive step.
/// @return A numeric vector the length of `values`.
#[miniextendr]
pub fn own_page_round(values: Vec<f64>, step: f64) -> Vec<f64> {
    values
        .into_iter()
        .map(|value| (value / step).round() * step)
        .collect()
}

/// @title Truncate values to a step
/// @description `own_page_truncate()` moves each value towards zero, onto a
/// multiple of `step`.
/// @inheritSection own_page_round Step rules
/// @param values Numbers to truncate.
/// @param step Positive step.
/// @return A numeric vector the length of `values`.
#[miniextendr]
pub fn own_page_truncate(values: Vec<f64>, step: f64) -> Vec<f64> {
    values
        .into_iter()
        .map(|value| (value / step).trunc() * step)
        .collect()
}
