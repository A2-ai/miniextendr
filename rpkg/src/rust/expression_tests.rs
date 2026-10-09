//! Fixtures for the expression subsystem (`RCall`, `REnv`, `r_eval_str`).
//!
//! Standalone `#[miniextendr]` functions run on the R main thread by default,
//! which is exactly what the expression module requires. `RCall::eval` and
//! `r_eval_str` catch an R error as `Err(REvalError)`, R's condition, never a
//! longjmp through Rust frames. Returned from a function, the `Err` is raised
//! again with the caught message and classes.
//!
//! See docs/EXPRESSION_EVAL.md.

use miniextendr_api::expression::{RCall, REnv, REvalError, r_eval_str, r_eval_str_global};
use miniextendr_api::into_r::IntoR;
use miniextendr_api::prelude::{OwnedProtect, SEXP, SexpExt};
use miniextendr_api::{List, ProtectScope, miniextendr, rust_error};

/// Parse and evaluate R source in the global environment; returns the value
/// of the last top-level expression (NULL for empty input).
/// @param code Character scalar of R source.
#[miniextendr]
pub fn expr_eval_str(code: &str) -> Result<SEXP, REvalError> {
    unsafe { r_eval_str_global(code) }
}

/// Build and evaluate `sum(x, na.rm = TRUE)` via the RCall builder
/// (positional + named argument paths).
/// @param x Numeric vector (may contain NA).
#[miniextendr]
pub fn expr_call_builder(x: SEXP) -> Result<SEXP, REvalError> {
    unsafe {
        let na_rm = OwnedProtect::new(SEXP::scalar_logical(true));
        RCall::new("sum")
            .arg(x)
            .named_arg("na.rm", na_rm.get())
            .eval_base()
    }
}

/// Build `c(1L, ..., 8L)` with freshly allocated inline arguments.
///
/// This exercises the ergonomic builder form shown in the public docs: each
/// argument must remain reachable while later scalar allocations occur.
#[miniextendr(noexport)]
pub fn expr_call_inline_arguments() -> Result<SEXP, REvalError> {
    unsafe {
        RCall::new("c")
            .arg(SEXP::scalar_integer(1))
            .arg(SEXP::scalar_integer(2))
            .arg(SEXP::scalar_integer(3))
            .arg(SEXP::scalar_integer(4))
            .arg(SEXP::scalar_integer(5))
            .arg(SEXP::scalar_integer(6))
            .arg(SEXP::scalar_integer(7))
            .arg(SEXP::scalar_integer(8))
            .eval_base()
    }
}

/// Resolve `name` in the base namespace and report whether it is a function.
/// Errors if the name does not resolve.
/// @param name Character scalar name to look up.
#[miniextendr]
pub fn expr_env_lookup(name: &str) -> Result<bool, REvalError> {
    unsafe {
        let env = REnv::base_namespace();
        let value = r_eval_str(name, env.as_sexp())?;
        Ok(value.is_function())
    }
}

/// Call `f()` through `RCall::eval_base`: `list(value = <value>)`, or, when
/// it raises an R error, the caught `REvalError`'s parts:
/// `list(message, classes, specific_classes, call, condition)`.
#[miniextendr(noexport)]
pub fn expr_catch_error(f: SEXP) -> SEXP {
    // SAFETY: R's main thread; `f` is the rooted `.Call()` argument.
    caught_parts(unsafe { RCall::from_sexp(f).eval_base() })
}

/// [`expr_catch_error`] for R source evaluated by `r_eval_str` in the global
/// environment.
/// @param code Character scalar of R source.
#[miniextendr(noexport)]
pub fn expr_catch_error_str(code: &str) -> SEXP {
    // SAFETY: R's main thread.
    caught_parts(unsafe { r_eval_str_global(code) })
}

/// `list(value = <value>)`, or the error's parts as an R list.
pub(crate) fn caught_parts(result: Result<SEXP, REvalError>) -> SEXP {
    // SAFETY: R's main thread; every part is protected (or rooted by `error`)
    // before the next allocation.
    unsafe {
        let error = match result {
            Ok(value) => {
                let value = OwnedProtect::new(value);
                return List::from_raw_pairs(vec![("value", value.get())]).into_sexp();
            }
            Err(error) => error,
        };
        let scope = ProtectScope::new();
        let message = scope.protect_raw(SEXP::scalar_string_from_str(error.message()));
        let classes = scope.protect_raw(strings(error.classes()));
        let specific = scope.protect_raw(strings(error.specific_classes()));
        List::from_raw_pairs(vec![
            ("message", message),
            ("classes", classes),
            ("specific_classes", specific),
            ("call", error.call().unwrap_or(SEXP::nil())),
            ("condition", error.condition()),
        ])
        .into_sexp()
    }
}

/// A character vector of `values`, unprotected.
fn strings(values: &[String]) -> SEXP {
    values.to_vec().into_sexp()
}

/// Call `f()` through `RCall::eval_base` and, when it raises an R error, raise
/// the package's own `mx_reraised` error with R's message, the caught classes
/// kept after it (`REvalError::reraise_class`).
#[miniextendr(noexport)]
pub fn expr_reraise(f: SEXP) -> SEXP {
    // SAFETY: R's main thread; `f` is the rooted `.Call()` argument.
    match unsafe { RCall::from_sexp(f).eval_base() } {
        Ok(value) => value,
        Err(e) => rust_error!(class = e.reraise_class("mx_reraised"), "{e}"),
    }
}
