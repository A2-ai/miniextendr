//! Subscript forms in S3 `[` methods: `i` / `j` / `drop` as `Missing`
//! parameters, and the call's argument count through `NArgs` (#1860).
//!
//! R tells `x[1:3]` from `x[1:3, ]` only through `nargs()` (2 against 3): both
//! give `i`, and `j` is missing in both. A method declares the subscripts it
//! takes as `Missing` parameters, so the empty argument of `x[i, ]` binds to
//! `j` instead of failing in the wrapper, and takes an `NArgs` parameter,
//! which the wrapper fills with `nargs()`.
//!
//! - `mx_vec1` (a classed double vector built in R) refuses every form but
//!   the one-subscript ones, `x[i]` and `x[]`, with a classed error.
//! - `mx_frame` (a data frame with that class in front) forwards every form to
//!   `[.data.frame` exactly as typed, so its results are `identical()` to
//!   `[.data.frame`'s.
//! - `mx_nargs` (a classed list) records the count of its replacement calls,
//!   `x[i] <- v` (3) and `x[i, ] <- v` (4).
//! - `MxNargsProbe` takes `NArgs` on an env-class trait method.
//!
//! The impl-block `[[` of `MxBagHandle` (`s3_nonsyntactic_tests.rs`) refuses
//! `h[[i, ]]` the same way. Tests: `rpkg/tests/testthat/test-s3-subscript.R`.

use miniextendr_api::condition::RConditionError;
use miniextendr_api::expression::RCall;
use miniextendr_api::prelude::{ProtectScope, SexpExt};
use miniextendr_api::{Missing, NArgs, SEXP, miniextendr};

// region: a method that refuses the two-subscript forms

/// An `mx_vec1` subscript other than `x[i]` / `x[]`.
#[derive(Debug, RConditionError)]
#[condition(
    class = "mx_vec1_subscript_error",
    message = "an mx_vec1 has one dimension: subset it as x[i], not with {subscripts} subscripts"
)]
pub struct Vec1SubscriptError {
    /// The subscripts written, empty ones included: `nargs()` without `x`
    /// and `drop`.
    subscripts: i32,
}

/// Subset an `mx_vec1` by 1-based integer positions, `x[i]`; `x[]` keeps
/// every value. Every two-subscript form (`x[i, ]`, `x[, j]`, `x[i, j]`,
/// `x[i, j, drop = FALSE]`) is refused with an `mx_vec1_subscript_error`.
///
/// @param x An `mx_vec1`.
/// @param i Integer positions to keep.
/// @param j Never accepted: declared so that `x[i, ]` reaches the method.
/// @param drop Never accepted: declared so that `x[i, j, drop = FALSE]`
///   reaches the method.
/// @export
#[miniextendr(s3(generic = "[", class = "mx_vec1"))]
pub fn mx_vec1_subset(
    x: Vec<f64>,
    i: Missing<Vec<i32>>,
    _j: Missing<SEXP>,
    drop: Missing<SEXP>,
    nargs: NArgs,
) -> Result<Vec<f64>, Vec1SubscriptError> {
    let subscripts = nargs.get() - 1 - usize::from(drop.is_present());
    if subscripts > 1 {
        return Err(Vec1SubscriptError {
            subscripts: i32::try_from(subscripts).unwrap_or(i32::MAX),
        });
    }
    let Some(i) = i.into_option() else {
        return Ok(x);
    };
    Ok(i.iter()
        .filter_map(|&k| {
            usize::try_from(k)
                .ok()
                .and_then(|k| k.checked_sub(1))
                .and_then(|k| x.get(k).copied())
        })
        .collect())
}

// endregion

// region: a method that forwards every form to `[.data.frame`

/// Subset an `mx_frame` the way a data frame is: every form, `x[i]`,
/// `x[i, ]`, `x[, j]`, `x[i, j]`, `x[i, j, drop = FALSE]` and `x[]`, is
/// forwarded to `[.data.frame` as typed.
///
/// @param x An `mx_frame`.
/// @param i,j Row and column subscripts, either may be empty.
/// @param drop Whether to drop to a vector, as for a data frame.
/// @export
#[miniextendr(s3(generic = "[", class = "mx_frame"))]
pub fn mx_frame_subset(
    x: SEXP,
    i: Missing<SEXP>,
    j: Missing<SEXP>,
    drop: Missing<SEXP>,
    nargs: NArgs,
) -> SEXP {
    // The subscript slots as typed: `nargs()` without `x` and a named `drop`.
    // `[.data.frame` reads the same count back (`nargs() - !missing(drop)`),
    // so one slot is list-style subscripting (`x[i]`, `x[]`) and two are
    // matrix-style (`x[i, ]`, `x[, j]`, `x[i, j]`).
    let slots = nargs.get() - 1 - usize::from(drop.is_present());
    // SAFETY: R's main thread. `RCall` roots each argument; the empty
    // argument `R_MissingArg` is a symbol and needs no root. The head is the
    // symbol `[.data.frame`, looked up in base: its list-style branch calls
    // `NextMethod()`, which needs to know the method by name.
    unsafe {
        let mut call = RCall::new("[.data.frame").quoted_arg(x);
        if slots >= 1 {
            call = subscript(call, i);
        }
        if slots >= 2 {
            call = subscript(call, j);
        }
        if let Missing::Present(drop) = drop {
            call = call.named_quoted_arg("drop", drop);
        }
        call.eval_with_handlers(miniextendr_api::sys::R_BaseEnv)
    }
}

/// Add a subscript as written: a given one as its value, an empty one as an
/// empty argument (`R_MissingArg`).
fn subscript(call: RCall, arg: Missing<SEXP>) -> RCall {
    match arg {
        Missing::Present(value) => call.quoted_arg(value),
        Missing::Absent => call.arg(SEXP::missing_arg()),
    }
}

// endregion

// region: the count of a replacement call

/// Record the argument count of a `[<-` call on an `mx_nargs` in the copy's
/// `nargs` attribute: 3 for `x[i] <- v`, 4 for `x[i, ] <- v`. `value` comes
/// before the `NArgs` parameter, which is no formal, so it stays the last.
///
/// @param x An `mx_nargs`.
/// @param i,j Subscripts, ignored.
/// @param value The new value, ignored.
/// @return `x`, with its `nargs` attribute set.
/// @export
#[miniextendr(s3(generic = "[<-", class = "mx_nargs"))]
pub fn mx_nargs_assign(
    x: SEXP,
    _i: Missing<SEXP>,
    _j: Missing<SEXP>,
    _value: SEXP,
    nargs: NArgs,
) -> SEXP {
    let count = i32::try_from(nargs.get()).unwrap_or(i32::MAX);
    // SAFETY: R's main thread; the copy is rooted across the attribute
    // value's allocation.
    let scope = unsafe { ProtectScope::new() };
    let copy = unsafe { scope.protect_raw(x.shallow_duplicate()) };
    copy.set_attr(SEXP::symbol("nargs"), SEXP::scalar_integer(count));
    copy
}

// endregion

// region: NArgs on a trait method

/// A probe whose trait method reports its call's argument count.
#[derive(miniextendr_api::ExternalPtr)]
pub struct MxNargsProbe;

#[miniextendr(env)]
impl MxNargsProbe {
    /// Create a probe.
    pub fn new() -> Self {
        MxNargsProbe
    }
}

/// Report a call's argument count from a trait method.
#[miniextendr]
pub trait ArgCount {
    /// The `nargs()` of the method's call; `a` is ignored.
    fn count_args(&self, a: i32, nargs: NArgs) -> i32;
}

#[miniextendr(env)]
impl ArgCount for MxNargsProbe {
    fn count_args(&self, _a: i32, nargs: NArgs) -> i32 {
        i32::try_from(nargs.get()).unwrap_or(i32::MAX)
    }
}

// endregion
