//! Condition-call markers: [`Call`] and [`CallerCall`].
//!
//! Every generated R wrapper hands its C entry point a call object in a hidden
//! first slot, `.Call(C_pkg_f, .call = <call>, ...)`. Conditions raised from
//! Rust (a panic, an `Err`, `error!()` & co.) and, for `caller` attribution,
//! the wrapper's own R-side checks are attributed to that call, so
//! `conditionCall()` names the function the user wrote with its formals
//! matched. Which call it is comes from the wrapper's *attribution*:
//!
//! | attribution | `.call =`                 | names                                   |
//! |-------------|---------------------------|-----------------------------------------|
//! | `wrapper`   | `match.call()`            | the wrapper's own call (the default)    |
//! | `caller`    | `.mx_call` (caller frame) | the hand-written R function delegating  |
//! | `none`      | `NULL`                    | nothing; R falls back to `sys.call()`   |
//!
//! A `#[miniextendr]` function chooses its attribution in one of three
//! equivalent spellings, most specific first: a **marker parameter** on the
//! signature, the **`call = none | wrapper | caller`** attribute, or the
//! crate-wide default in `Cargo.toml`
//! (`[package.metadata.miniextendr] call_attribution = "..."`). The framework
//! default is `wrapper`.
//!
//! ```ignore
//! use miniextendr_api::{Call, CallerCall, miniextendr};
//!
//! /// `.call = match.call()`; `call` is that call, available to the body.
//! #[miniextendr]
//! pub fn scale(x: f64, call: Call) -> f64 { let _ = call.sexp(); x * 2.0 }
//!
//! /// Internal entry point behind a hand-written `scale2()` in R/: the caller's
//! /// call, with the caller's formals matched, reaches Rust as `call`.
//! #[miniextendr(noexport)]
//! pub fn scale2_impl(x: f64, _call: CallerCall) -> f64 { x * 2.0 }
//! ```
//!
//! The marker parameter is not an R argument: the R wrapper's formals are the
//! other parameters, and the C wrapper binds the marker from its hidden call
//! slot. `Call` selects `wrapper` attribution, `CallerCall` selects `caller`
//! (and, like the attribute, needs `noexport` or `internal`: an exported
//! function's caller is arbitrary user code). A marker and a `call = ...`
//! attribute that disagree are a compile error, as are two markers on one
//! function, a marker on an `extern "C-unwind"` function (no generated call
//! slot) and a marker on a class method (methods keep the wrapper's own call).
//! A function taking a marker runs on R's main thread, like one taking `SEXP`.
//!
//! Both types are transparent newtypes over [`SEXP`]: the language object R
//! matched (`is.call()`), or `R_NilValue` when the wrapper attributed nothing.
//! They are handles for inspection, `deparse()` in a log line say, never
//! roots: the object lives only as long as the `.Call()` frame that carried
//! it, so it must not outlive the function call it was passed to.

use crate::SEXP;
use crate::sexp_ext::SexpExt;

/// The call this wrapper attributes conditions to, as R matched it
/// (`match.call()` in the generated wrapper). Taking it selects `wrapper`
/// attribution. See the [module docs](self).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Call(SEXP);

/// The call of the R function that called this wrapper, with that function's
/// formals matched (`.miniextendr_caller_call()` in the generated wrapper).
/// Taking it selects `caller` attribution; the function must be `noexport` or
/// `internal`. See the [module docs](self).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallerCall(SEXP);

macro_rules! call_marker_impls {
    ($name:ident) => {
        impl $name {
            /// Wrap the call object a generated C wrapper received in its
            /// hidden `.call` slot. Generated code calls this; user code has no
            /// reason to.
            #[doc(hidden)]
            #[inline]
            pub const fn from_sexp(call: SEXP) -> Self {
                $name(call)
            }

            /// The call as an R language object, or `R_NilValue` when the
            /// wrapper attributed nothing.
            #[inline]
            pub const fn sexp(self) -> SEXP {
                self.0
            }

            /// Whether the wrapper handed over `NULL` instead of a call.
            #[inline]
            pub fn is_nil(self) -> bool {
                self.0.is_nil()
            }
        }

        impl ::std::ops::Deref for $name {
            type Target = SEXP;

            #[inline]
            fn deref(&self) -> &SEXP {
                &self.0
            }
        }

        impl From<$name> for SEXP {
            #[inline]
            fn from(call: $name) -> SEXP {
                call.0
            }
        }
    };
}

call_marker_impls!(Call);
call_marker_impls!(CallerCall);

/// A caller's call is a call: hand it to an API that takes [`Call`].
impl From<CallerCall> for Call {
    #[inline]
    fn from(call: CallerCall) -> Call {
        Call(call.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_are_transparent_over_sexp() {
        assert_eq!(std::mem::size_of::<Call>(), std::mem::size_of::<SEXP>());
        assert_eq!(
            std::mem::size_of::<CallerCall>(),
            std::mem::size_of::<SEXP>()
        );
        let raw = SEXP::null();
        let call = Call::from_sexp(raw);
        assert_eq!(call.sexp(), raw);
        assert_eq!(SEXP::from(call), raw);
        let caller = CallerCall::from_sexp(raw);
        assert_eq!(Call::from(caller), call);
        assert_eq!(*caller, raw);
    }
}
