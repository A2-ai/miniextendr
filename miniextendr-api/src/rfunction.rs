//! Borrowed, validated R callbacks.

use crate::{IntoR, RCall, Root, SEXP, SexpClassError, SexpExt, TryFromRoot};

/// An R function borrowed for the lifetime of its root.
///
/// Accepts closures, builtins and specials, as R's `is.function()` does.
/// A `#[miniextendr]` parameter uses the call's root; `Option<RFunction<'_>>`
/// accepts `NULL` as `None`. This handle adds no owned GC root and is
/// neither `Send` nor `Sync`. To store the function beyond the call, explicitly
/// preserve its [`SEXP`] with [`crate::Preserved`].
///
/// [`Self::call`] starts the existing [`RCall`] builder, which owns a root for
/// the callable and its arguments. Choose that builder's evaluator according
/// to the desired handler behavior. An evaluator that preserves R's handlers
/// turns an R exit into a Rust unwind: it must reach a miniextendr boundary
/// through Rust frames only. Do not call it from a foreign callback, catch
/// and discard its unwind, or call it from a destructor.
///
/// ```no_run
/// use miniextendr_api::{RFunction, SEXP, miniextendr};
///
/// #[miniextendr]
/// fn apply(f: RFunction<'_>, x: SEXP) -> SEXP {
///     // The generated wrapper runs on R's main thread under an unwind boundary.
///     unsafe { f.call().arg(x).eval_with_handlers(miniextendr_api::sys::R_GlobalEnv) }
/// }
/// ```
///
/// ```compile_fail
/// use miniextendr_api::{RFunction, Root, TryFromRoot};
/// fn escape(root: Root<'_>) -> RFunction<'static> {
///     RFunction::try_from_root(root).unwrap()
/// }
/// ```
///
/// ```compile_fail
/// use miniextendr_api::RFunction;
/// fn require_send<T: Send>() {}
/// require_send::<RFunction<'static>>();
/// ```
#[derive(Clone, Copy)]
pub struct RFunction<'a>(Root<'a>);

impl std::fmt::Debug for RFunction<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RFunction").field(&self.as_sexp()).finish()
    }
}

impl RFunction<'_> {
    /// The underlying function. It remains reachable while the root lives.
    #[inline]
    pub fn as_sexp(self) -> SEXP {
        self.0.get()
    }

    /// Start a call builder that owns its own root for this function.
    #[inline]
    pub fn call(self) -> RCall {
        // The root retains a validated callable on R's main thread.
        unsafe { RCall::from_sexp(self.as_sexp()) }
    }
}

impl<'a> TryFromRoot<'a> for RFunction<'a> {
    type Error = SexpClassError;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        if root.get().is_function() {
            Ok(Self(root))
        } else {
            Err(SexpClassError {
                expected: "a function",
                actual: root.get().type_of(),
            })
        }
    }
}

impl<'a> TryFromRoot<'a> for Option<RFunction<'a>> {
    type Error = SexpClassError;

    fn try_from_root(root: Root<'a>) -> Result<Self, Self::Error> {
        if root.get().is_null_or_nil() {
            Ok(None)
        } else {
            RFunction::try_from_root(root).map(Some)
        }
    }
}

impl IntoR for RFunction<'_> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.as_sexp())
    }
}
