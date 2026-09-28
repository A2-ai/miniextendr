//! Parameters named like a base function the generated wrapper calls:
//! `length` (type guards), `c` (a choice formal's default), `missing` and
//! `quote` (`Missing<T>` forwarding), `attr` (the check after `.Call()`) and
//! `list` (dots forwarding). The wrapper writes those calls
//! `base::name(...)`, so omitting such an argument, or passing a function
//! there, affects only that argument. Driven by
//! `tests/testthat/test-shadowed-formals.R`.
//!
//! Each colliding parameter that targets a site other than a type guard is a
//! `Missing<SEXP>`: it has no R type guard of its own, so a function passed
//! there reaches the site instead of being refused first.

use miniextendr_api::dots::Dots;
use miniextendr_api::{ExternalPtr, MatchArg, Missing, SEXP, miniextendr};

/// Returns `length`. The type guards of both parameters call `length()`, and
/// `overwrite` comes first.
/// @param overwrite A flag, unused.
/// @param length A number, returned.
#[miniextendr(noexport)]
pub fn shadowed_length(#[miniextendr(default = "FALSE")] overwrite: bool, length: f64) -> f64 {
    let _ = overwrite;
    length
}

/// Speed setting for [`shadowed_c`].
#[derive(Copy, Clone, Debug, PartialEq, MatchArg)]
#[match_arg(rename_all = "lower")]
pub enum ShadowMode {
    Fast,
    Slow,
}

/// Returns the chosen mode. Its formal default is `c("fast", "slow")`, next to
/// a parameter named `c`.
/// @param mode The mode.
/// @param c Anything, unused.
#[miniextendr(noexport)]
pub fn shadowed_c(#[miniextendr(match_arg)] mode: ShadowMode, c: Missing<SEXP>) -> String {
    let _ = c;
    match mode {
        ShadowMode::Fast => "fast",
        ShadowMode::Slow => "slow",
    }
    .to_string()
}

/// Whether `x` was given. Its `no_na` check calls `missing(x)`, and the
/// forwarding of all three arguments calls `missing()` and `quote()`.
/// @param x A non-NA double, or omitted.
/// @param missing Anything, unused.
/// @param quote Anything, unused.
#[miniextendr(noexport)]
pub fn shadowed_missing(
    #[miniextendr(no_na)] x: Missing<f64>,
    missing: Missing<SEXP>,
    quote: Missing<SEXP>,
) -> bool {
    let _ = (missing, quote);
    x.is_present()
}

/// Returns `1`, or fails when `fail` is `TRUE`. The check after `.Call()`
/// calls `attr()` on the failure.
/// @param fail Whether to fail.
/// @param attr Anything, unused.
#[miniextendr(noexport)]
pub fn shadowed_attr(fail: bool, attr: Missing<SEXP>) -> Result<f64, String> {
    let _ = attr;
    if fail {
        Err("shadowed_attr failed".to_string())
    } else {
        Ok(1.0)
    }
}

/// The number of arguments in the dots, which the wrapper forwards as
/// `list(...)`.
/// @param list Anything, unused.
/// @param ... Counted.
#[miniextendr(noexport)]
pub fn shadowed_list(list: Missing<SEXP>, dots: &Dots) -> i32 {
    let _ = list;
    i32::try_from(dots.len()).expect("dots length fits i32")
}

/// Holds a size; its method's type guards call `length()` next to a
/// parameter named `length`.
#[derive(ExternalPtr)]
pub struct ShadowedHolder {
    size: f64,
}

/// An R6 class whose method has a parameter named `length`.
#[miniextendr(r6, noexport)]
impl ShadowedHolder {
    /// Creates a holder of size 0.
    pub fn new() -> Self {
        ShadowedHolder { size: 0.0 }
    }

    /// Sets the size to `length` and returns it.
    /// @param overwrite A flag, unused.
    /// @param length The new size.
    #[miniextendr(defaults(overwrite = "FALSE"))]
    pub fn resize(&mut self, overwrite: bool, length: f64) -> f64 {
        let _ = overwrite;
        self.size = length;
        self.size
    }
}
