//! Fixtures for `RFunction` (#1947): an R function as an argument, under
//! `Option`, returned, and called with the user's handlers in place.

use miniextendr_api::expression::REvalError;
use miniextendr_api::sys::R_BaseEnv;
use miniextendr_api::{RFunction, SEXP, SEXPTYPE, SexpExt, Unchecked, miniextendr};

/// The kind of function received.
/// @param f A function.
#[miniextendr(noexport)]
pub fn rfunction_kind(f: RFunction) -> &'static str {
    match f.as_sexp().type_of() {
        SEXPTYPE::CLOSXP => "closure",
        SEXPTYPE::BUILTINSXP => "builtin",
        SEXPTYPE::SPECIALSXP => "special",
        other => panic!("RFunction holds a {other:?}"),
    }
}

/// The same function, through the Rust conversion only: no R-side check.
/// @param f A function.
#[miniextendr(noexport)]
pub fn rfunction_kind_unchecked(f: Unchecked<RFunction>) -> &'static str {
    rfunction_kind(*f)
}

/// Whether a function was given (`NULL` reads as `None`).
/// @param f `NULL` or a function.
#[miniextendr(noexport)]
pub fn rfunction_is_some(f: Option<RFunction>) -> bool {
    f.is_some()
}

/// The function, returned.
/// @param f A function.
#[miniextendr(noexport)]
pub fn rfunction_identity(f: RFunction) -> RFunction {
    f
}

/// `f(x)`; an error in `f` is raised with its own message and classes.
/// @param f A function of one argument.
/// @param x Its argument.
#[miniextendr(noexport)]
pub fn rfunction_call_on(f: RFunction, x: f64) -> Result<SEXP, REvalError> {
    unsafe { f.call().arg_value(x).try_eval_with_handlers(R_BaseEnv) }
}
