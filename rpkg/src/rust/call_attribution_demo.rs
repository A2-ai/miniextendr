//! Side-by-side fixture for `docs/CALL_ATTRIBUTION.md`.
//!
//! Two functions raise the same error message. One goes through the standard
//! `#[miniextendr]` wrapper (which emits `.call = match.call()`); the other is
//! `extern "C-unwind"`, which has no generated R wrapper and so no call slot.
//! The R-side error rendering is dramatically different.

use miniextendr_api::miniextendr;
use miniextendr_api::prelude::SEXP;
use miniextendr_api::{Call, CallerCall};

use crate::match_arg_tests::Mode;

/// Wrapped path. The generated R wrapper passes `.call = match.call()` into the
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
/// so `conditionCall(e)` names that public function with its formals matched.
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

/// Default attribution for comparison: the same shape without `call = caller`
/// reports its own wrapper call (`call_attr_self_impl(x = value)`).
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
/// `match.call()`, here returned to R for the test to compare.
///
/// @param x Ignored.
/// @noRd
#[miniextendr(noexport)]
pub fn call_marker_wrapper_impl(_x: i32, call: Call) -> SEXP {
    call.sexp()
}

/// `CallerCall` marker: the type-level spelling of `call = caller`. Behind the
/// hand-written `call_marker_caller()` in `R/call_attribution.R` the body sees
/// that function's matched call; called directly it sees its own.
///
/// @param x Ignored.
/// @noRd
#[miniextendr(noexport)]
pub fn call_marker_caller_impl(_x: i32, call: CallerCall) -> SEXP {
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

/// `call = none`: the attribute spelling of `no_call_attribution`. The wrapper
/// passes `.call = NULL`; on error R's `sys.call()` fallback still names the
/// wrapper, but without the formals matched.
///
/// @param x Must be positive.
/// @noRd
#[miniextendr(noexport, call = none)]
pub fn call_attr_none_impl(x: i32) -> Result<i32, String> {
    if x <= 0 {
        return Err(format!("x must be positive, got {x}"));
    }
    Ok(x)
}

// endregion
