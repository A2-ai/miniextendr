//! Side-by-side fixture for `docs/CALL_ATTRIBUTION.md`.
//!
//! Two functions raise the same error message. One goes through the standard
//! `#[miniextendr]` wrapper (which emits `.call = sys.call()`); the other is
//! `extern "C-unwind"`, which has no generated R wrapper and so no call slot.
//! The R-side error rendering is dramatically different.

use miniextendr_api::dots::Dots;
use miniextendr_api::prelude::SEXP;
use miniextendr_api::{Call, CallerCall, Missing, defer_warning, miniextendr};

use crate::match_arg_tests::Mode;

/// Wrapped path. The generated R wrapper passes `.call = sys.call()` into the
/// C entry; on panic, `Rf_errorcall(call, msg)` shows the user's call frame.
///
/// @param left Ignored.
/// @param right Ignored.
/// @noRd
#[miniextendr(noexport)]
pub fn call_attr_with(_left: i32, _right: i32) -> i32 {
    panic!("left + right is too risky")
}

/// Unwrapped path. `extern "C-unwind"` bypasses the wrapper entirely — there is
/// no call slot and no `with_r_unwind_protect`. We raise an R error directly
/// with `Rf_error`, which carries no call attribution.
///
/// @param left Ignored.
/// @param right Ignored.
/// @noRd
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_call_attr_without(_left: SEXP, _right: SEXP) -> SEXP {
    unsafe {
        ::miniextendr_api::sys::Rf_error(c"%s".as_ptr(), c"left + right is too risky".as_ptr()) // mxl::allow(MXL300)
    }
}

// region: call = caller — internal entry points behind a hand-written R function (#1450)

/// Internal entry point that attributes conditions to its caller. The
/// hand-written `call_attr_caller()` in `R/call_attribution.R` delegates here,
/// so `conditionCall(e)` names that public function, as written.
///
/// @param x Must be positive.
/// @noRd
#[miniextendr(noexport, call = caller)]
pub fn call_attr_caller_impl(x: i32) -> Result<i32, String> {
    if x <= 0 {
        return Err(format!("x must be positive, got {x}"));
    }
    Ok(x)
}

/// Internal entry point whose R-side checks are attributed to the caller too
/// (#1548): a scalar `match_arg` choice, a `several_ok` list, an optional
/// `choices` parameter and a typed precondition. The hand-written
/// `call_attr_checked()` in `R/call_attribution.R` delegates here, so a bad
/// choice or a non-integer `n` surfaces as `Error in call_attr_checked(...)`
/// naming the argument, the same way a Rust-side error does.
///
/// @param mode One of the modes.
/// @param metrics One or more of the metrics.
/// @param n An integer scalar.
/// @param level An optional level.
/// @noRd
#[miniextendr(noexport, call = caller)]
pub fn call_attr_checked_impl(
    #[miniextendr(match_arg)] mode: Mode,
    #[miniextendr(choices("mean", "median", "sd"), several_ok)] metrics: Vec<String>,
    n: i32,
    #[miniextendr(choices("low", "high"))] level: Option<String>,
) -> String {
    format!(
        "{mode:?}:{}:{n}:{}",
        metrics.join("+"),
        level.as_deref().unwrap_or("none")
    )
}

/// Internal entry point with an omittable choice (#1551) under `call = caller`:
/// the omission guard wraps the same caller-attributed check. The
/// hand-written `call_attr_omitted()` in `R/call_attribution.R` forwards its
/// argument without a default, so an omitted argument stays missing here.
///
/// @param mode One of the modes, NULL, or omitted.
/// @noRd
#[miniextendr(noexport, call = caller)]
pub fn call_attr_omitted_impl(#[miniextendr(match_arg)] mode: Missing<Option<Mode>>) -> String {
    match mode {
        Missing::Absent => "absent".to_string(),
        Missing::Present(None) => "null".to_string(),
        Missing::Present(Some(mode)) => format!("{mode:?}"),
    }
}

/// Default attribution for comparison: the same shape without `call = caller`
/// reports its own wrapper call (`call_attr_self_impl(value)`).
///
/// @param x Must be positive.
/// @noRd
#[miniextendr(noexport)]
pub fn call_attr_self_impl(x: i32) -> Result<i32, String> {
    if x <= 0 {
        return Err(format!("x must be positive, got {x}"));
    }
    Ok(x)
}

// endregion

// region: the three spellings of the attribution (#1566)

/// `Call` marker: the type-level spelling of `wrapper` attribution. The
/// marker is not an R formal (the wrapper takes `x` only); the C wrapper binds
/// it from its hidden call slot, so the body sees the wrapper's own
/// `sys.call()`, here returned to R for the test to compare.
///
/// @param x Ignored.
/// @noRd
#[miniextendr(noexport)]
pub fn call_marker_wrapper_impl(_x: i32, call: Call) -> SEXP {
    call.sexp()
}

/// `CallerCall` marker: the type-level spelling of `call = caller`. Behind the
/// hand-written `call_marker_caller()` in `R/call_attribution.R` the body sees
/// that function's call as written; called directly it sees its own.
///
/// @param x Ignored.
/// @noRd
#[miniextendr(noexport)]
pub fn call_marker_caller_impl(_x: i32, call: CallerCall) -> SEXP {
    call.sexp()
}

/// `Call` marker on a standalone S3 method: the marker is accepted there, and
/// the body sees the call the method's conditions would report. Under
/// `UseMethod()` dispatch that is the generic's call, `format(obj)` (#1851:
/// the method frame's own `sys.call()` would name the method,
/// `format.mx_call_marker(obj)`); called directly, the method's own. Here it
/// is returned to R for the test to compare.
///
/// @param x An object of class `mx_call_marker`.
/// @param ... Ignored.
/// @return The call the method was handed.
#[miniextendr(s3(generic = "format", class = "mx_call_marker"))]
pub fn format_mx_call_marker(_x: SEXP, _dots: &Dots, call: Call) -> SEXP {
    call.sexp()
}

/// `Call` marker together with an `Err`: the marker changes what the body can
/// see, not how conditions are attributed, so this reports its own call like
/// `call_attr_self_impl` does.
///
/// @param x Must be positive.
/// @noRd
#[miniextendr(noexport)]
pub fn call_marker_checked_impl(x: i32, _call: Call) -> Result<i32, String> {
    if x <= 0 {
        return Err(format!("x must be positive, got {x}"));
    }
    Ok(x)
}

// endregion

// region: a helper in between passes on the call to report (#1613)

/// A `call = caller` entry point that renders a page (`internal`), so
/// `R CMD check` compares its usage and arguments with the wrapper, whose
/// formals end with `.call = NULL`.
///
/// @param x Must be positive.
#[miniextendr(internal, call = caller)]
pub fn call_attr_internal_impl(x: i32) -> Result<i32, String> {
    if x <= 0 {
        return Err(format!("x must be positive, got {x}"));
    }
    Ok(x)
}

/// A `call = caller` entry point taking `...`: the `.call` formal follows the
/// dots (`function(x, ..., .call = NULL)`), so positional extras land in the
/// dots and `.call` is matched by name only. Returns `x` plus the number of
/// extras.
///
/// @param x Must be non-negative.
/// @param ... Counted.
/// @noRd
#[miniextendr(noexport, call = caller)]
pub fn call_attr_dots_impl(x: i32, dots: &Dots) -> Result<i32, String> {
    if x < 0 {
        return Err(format!("x must be non-negative, got {x}"));
    }
    let extras = i32::try_from(dots.len()).map_err(|e| e.to_string())?;
    Ok(x + extras)
}

// endregion

// region: call_arg — an exported function takes the call to report (#1834)

/// An exported function with a `.call` argument: its conditions name the call
/// passed there, or its own call when `.call` is `NULL`. An R function
/// composing it, through `do.call()` say, passes `.call = environment()`.
/// Each kind of condition goes through `.call`: an error from Rust (a negative
/// `x`), the R-side checks of `x` (an integer scalar) and `mode` (a choice),
/// and a warning deferred from Rust (`x` is zero).
///
/// @param x A count: negative is an error, zero a warning.
/// @param mode One of the modes.
/// @return `x`.
#[miniextendr(call_arg)]
pub fn call_arg_verb(x: i32, #[miniextendr(match_arg)] mode: Mode) -> Result<i32, String> {
    let _ = mode;
    if x < 0 {
        return Err(format!("x must be non-negative, got {x}"));
    }
    if x == 0 {
        defer_warning!("x is zero");
    }
    Ok(x)
}

/// A page for [call_arg_joined()] to join: a plain exported function, so the
/// page's own block has no `.call` line.
///
/// @param x Returned.
/// @return `x`.
#[miniextendr]
pub fn call_arg_topic(x: i32) -> i32 {
    x
}

/// A `call_arg` function on a page defined by another block (`@rdname`). The
/// generated `@param .call` line is kept there, so the shared page documents
/// the `.call` in its usage.
///
/// @rdname call_arg_topic
#[miniextendr(call_arg)]
pub fn call_arg_joined(x: i32) -> Result<i32, String> {
    if x < 0 {
        return Err(format!("x must be non-negative, got {x}"));
    }
    Ok(x)
}

/// A `Call` marker with `call_arg`: the body receives the call `.call`
/// resolves to, here returned to R for the test to compare.
///
/// @param x Ignored.
/// @noRd
#[miniextendr(noexport, call_arg)]
pub fn call_arg_marker_impl(_x: i32, call: Call) -> SEXP {
    call.sexp()
}

// endregion
