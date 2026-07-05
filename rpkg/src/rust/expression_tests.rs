//! Fixtures for the expression subsystem (`RCall`, `REnv`, `r_eval_str`).
//!
//! Standalone `#[miniextendr]` functions run on the R main thread by default,
//! which is exactly what the expression module requires. Error paths go
//! through `R_tryEvalSilent`, so R-level failures surface as `Err(String)`
//! (→ an R error via the tagged-condition transport), never a longjmp
//! through Rust frames.
//!
//! See docs/EXPRESSION_EVAL.md.

use miniextendr_api::expression::{RCall, REnv, r_eval_str, r_eval_str_global};
use miniextendr_api::miniextendr;
use miniextendr_api::prelude::{OwnedProtect, SEXP, SexpExt};

/// Parse and evaluate R source in the global environment; returns the value
/// of the last top-level expression (NULL for empty input).
/// @param code Character scalar of R source.
#[miniextendr]
pub fn expr_eval_str(code: &str) -> Result<SEXP, String> {
    unsafe { r_eval_str_global(code) }
}

/// Build and evaluate `sum(x, na.rm = TRUE)` via the RCall builder
/// (positional + named argument paths).
/// @param x Numeric vector (may contain NA).
#[miniextendr]
pub fn expr_call_builder(x: SEXP) -> Result<SEXP, String> {
    unsafe {
        let na_rm = OwnedProtect::new(SEXP::scalar_logical(true));
        RCall::new("sum")
            .arg(x)
            .named_arg("na.rm", na_rm.get())
            .eval_base()
    }
}

/// Resolve `name` in the base namespace and report whether it is a function.
/// Errors if the name does not resolve.
/// @param name Character scalar name to look up.
#[miniextendr]
pub fn expr_env_lookup(name: &str) -> Result<bool, String> {
    unsafe {
        let env = REnv::base_namespace();
        let value = r_eval_str(name, env.as_sexp())?;
        Ok(value.is_function())
    }
}
