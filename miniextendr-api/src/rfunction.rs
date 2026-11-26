//! [`RFunction`]: an R function as a `#[miniextendr]` argument (#1947).

use std::marker::PhantomData;

use crate::expression::RCall;
use crate::from_r::{SexpClassError, SexpError, TryFromSexp};
use crate::into_r::IntoR;
use crate::{SEXP, SEXPTYPE, SexpExt};

/// An R function: a closure, a builtin (`sum`) or a special (`quote`).
///
/// As a `#[miniextendr]` parameter it accepts exactly what `is.function()`
/// accepts and refuses anything else, `'f' must be a function` (an R-side
/// check, and the Rust conversion's `'f' must be a function: got integer`
/// when that check is off). `Option<RFunction>` reads `NULL` as `None`.
/// Returned, it gives the function back.
///
/// ```ignore
/// use miniextendr_api::expression::REvalError;
/// use miniextendr_api::{RFunction, SEXP, miniextendr, sys::R_BaseEnv};
///
/// /// `f(x)`; an error in `f` is raised with its own message and classes.
/// #[miniextendr]
/// pub fn call_on(f: RFunction, x: f64) -> Result<SEXP, REvalError> {
///     unsafe { f.call().arg_value(x).try_eval_with_handlers(R_BaseEnv) }
/// }
/// ```
///
/// # Calling it
///
/// [`call`](Self::call) starts an [`RCall`] with the function as its head;
/// add arguments and evaluate it as any other call. Evaluate a callback with
/// [`RCall::eval_with_handlers`], or with
/// [`RCall::try_eval_with_handlers`] to get its error back as `Err`: the
/// user's handlers then see what the function signals, with its classes.
/// [`RCall::eval`] runs it at top level, where the user's handlers don't
/// reach.
///
/// An R error or other exit from the function unwinds through Rust frames,
/// which the evaluator handles. It cannot unwind through foreign (C, C++,
/// Fortran) frames: to call the function from a callback that a foreign
/// library invokes, catch the exit there with
/// [`call_unwinding`](crate::unwind_protect::call_unwinding) and resume it
/// after the library returns.
///
/// # Lifetime
///
/// A view of the argument, like [`List`](crate::list::List): the `.Call()`
/// that received it keeps the function alive for that call, and nothing
/// roots it after. To keep it past the call (in a struct, a registry, a
/// closure stored for later), root it with
/// [`Preserved::new(f.as_sexp())`](crate::Preserved::new). It is not `Send`.
#[derive(Clone, Copy, Debug)]
pub struct RFunction {
    sexp: SEXP,
    /// `SEXP` is `Send`; a function is only ever used on R's main thread.
    _not_send: PhantomData<*const ()>,
}

impl RFunction {
    /// The function itself.
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.sexp
    }

    /// Start a call of this function: add arguments to the returned
    /// [`RCall`] and evaluate it (see [Calling it](Self#calling-it)).
    ///
    /// # Safety
    ///
    /// R's main thread, while the function is alive (during the `.Call()`
    /// that received it, or while rooted).
    #[inline]
    pub unsafe fn call(&self) -> RCall {
        unsafe { RCall::from_sexp(self.sexp) }
    }
}

impl TryFromSexp for RFunction {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.is_function() {
            Ok(RFunction {
                sexp,
                _not_send: PhantomData,
            })
        } else {
            Err(SexpClassError {
                expected: "a function",
                actual: sexp.type_of(),
            }
            .into())
        }
    }
}

/// `NULL` → `None`; any other value converts as an [`RFunction`] and becomes
/// `Some`.
///
/// Concrete rather than left to the newtype blanket `Option<T>` impl
/// (`newtype.rs`), which `RFunction` does not match.
impl TryFromSexp for Option<RFunction> {
    type Error = <RFunction as TryFromSexp>::Error;
    // `__MX_EXPECTATION` keeps its `None`: `#[miniextendr]` words an
    // `Option<_>` parameter itself (`NULL or a function`).

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            <RFunction as TryFromSexp>::try_from_sexp(sexp).map(Some)
        }
    }
}

impl IntoR for RFunction {
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.sexp)
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(self.sexp)
    }

    #[inline]
    fn into_sexp(self) -> SEXP {
        self.sexp
    }
}
