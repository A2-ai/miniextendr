//! Fixtures for `BuiltList`, the rooted plain-list finish of
//! `NamedListBuilder`: a classed result that R reads while Rust is still
//! building it, then returned.
//!
//! See `tests/testthat/test-built-list.R`.

use miniextendr_api::expression::{RCall, REvalError};
use miniextendr_api::{BuiltList, NamedListBuilder, miniextendr};

/// A classed summary of `x`: `list(n, mean, values)` with class
/// `c("summary_of", "list")`, and a `fields` attribute that R computes from
/// the list (`names()`) before it is returned. `mean` is `NA` for no values.
/// @param x Numeric vector.
#[miniextendr(noexport)]
pub fn built_list_summary(x: Vec<f64>) -> Result<BuiltList, REvalError> {
    let n = i32::try_from(x.len()).expect("a numeric vector has at most i32::MAX values here");
    let mean = (n > 0).then(|| x.iter().sum::<f64>() / f64::from(n));
    let mut parts = NamedListBuilder::with_capacity(3);
    parts.push("n", n);
    parts.push("mean", mean);
    parts.push("values", x);
    let summary: BuiltList = parts.into_list().set_class(&["summary_of", "list"]);

    // R reads the list while the handle roots it.
    // SAFETY: R's main thread, inside this call's boundary; the list is
    // rooted by `summary`.
    let fields: Vec<String> = unsafe { RCall::new("names").arg(summary.as_sexp()).eval_base_as() }?;
    Ok(summary.set_attr("fields", fields.join(",")))
}
