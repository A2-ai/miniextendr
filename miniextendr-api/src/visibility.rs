//! Return-visibility markers: [`Invisible<T>`] and [`Visible<T>`].
//!
//! An R function's result is either printed at the console when the call is
//! the top-level expression (visible) or not (invisible, as `invisible(x)`
//! returns it). `#[miniextendr]` wrappers decide this per function or method:
//!
//! - A bare function whose R value is `NULL` (no return type, `-> ()`,
//!   `Option<()>`, `Result<(), E>` on success) returns it invisibly, so a
//!   side-effect call does not print `NULL`.
//! - Every other return is visible, including a method that hands back its
//!   receiver for chaining (`&mut self -> ()`, `&mut self -> &mut Self`,
//!   `self -> Self`): `counter$increment()` prints the counter, exactly as an
//!   R function returning `self` would.
//!
//! Wrapping the return type changes that decision without touching the
//! value: `Invisible<T>` makes the wrapper return `invisible(...)`,
//! `Visible<T>` forces a visible return where the default is invisible (a
//! visible `NULL`, say). The attribute spelling, `#[miniextendr(invisible)]`
//! / `#[miniextendr(visible)]` on a function or `r6(invisible)` & co. on a
//! method, does the identical thing; use whichever reads better. A marker and
//! an attribute that disagree are a compile error.
//!
//! ```ignore
//! use miniextendr_api::{Invisible, miniextendr};
//!
//! #[derive(miniextendr_api::ExternalPtr)]
//! pub struct Counter { n: i32 }
//!
//! #[miniextendr(r6)]
//! impl Counter {
//!     pub fn new() -> Self { Counter { n: 0 } }
//!     /// Chainable and silent: `c$bump()` prints nothing, `c$bump()$bump()` works.
//!     pub fn bump(&mut self) -> Invisible<()> { self.n += 1; Invisible(()) }
//!     /// Chainable and visible (the default): `c$tick()` prints the counter.
//!     pub fn tick(&mut self) { self.n += 1; }
//!     /// A value returned invisibly, `withVisible(c$peek())$visible` is `FALSE`.
//!     pub fn peek(&self) -> Invisible<i32> { Invisible(self.n) }
//! }
//! ```
//!
//! The marker is transparent everywhere else: `Invisible<Option<i32>>` keeps
//! `Option`'s `None`-raises semantics, `Invisible<Self>` still wraps the
//! returned handle in the class, `Invisible<Result<T, E>>` still raises on
//! `Err`. Both types implement [`IntoR`] by forwarding to `T`, so a value that
//! reaches a conversion without the macro having peeled the marker (a type
//! alias, a `use Invisible as Quiet` rename) still converts correctly; only
//! the visibility falls back to the default. Markers are return-position only:
//! `Invisible<T>` as a parameter type is a compile error, as is nesting one
//! marker inside another.

use crate::into_r::IntoR;

/// Return-position marker: the R wrapper returns this value with
/// `invisible()`. See the [module docs](self).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Invisible<T>(pub T);

/// Return-position marker: the R wrapper returns this value visibly even where
/// the default is invisible (a `NULL` from a unit-returning function). See the
/// [module docs](self).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Visible<T>(pub T);

macro_rules! marker_impls {
    ($name:ident) => {
        impl<T> $name<T> {
            /// Wrap a value.
            #[inline]
            pub const fn new(value: T) -> Self {
                $name(value)
            }

            /// Unwrap the value.
            #[inline]
            pub fn into_inner(self) -> T {
                self.0
            }
        }

        impl<T> From<T> for $name<T> {
            #[inline]
            fn from(value: T) -> Self {
                $name(value)
            }
        }

        impl<T> ::std::ops::Deref for $name<T> {
            type Target = T;

            #[inline]
            fn deref(&self) -> &T {
                &self.0
            }
        }

        impl<T> ::std::ops::DerefMut for $name<T> {
            #[inline]
            fn deref_mut(&mut self) -> &mut T {
                &mut self.0
            }
        }

        /// Forwards to `T`: the marker only informs the generated R wrapper.
        impl<T: IntoR> IntoR for $name<T> {
            type Error = T::Error;

            #[inline]
            fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
                self.0.try_into_sexp()
            }

            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
                // SAFETY: same contract as the caller's.
                unsafe { self.0.try_into_sexp_unchecked() }
            }
        }
    };
}

marker_impls!(Invisible);
marker_impls!(Visible);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_are_transparent_wrappers() {
        let i = Invisible::new(41).into_inner() + 1;
        assert_eq!(i, 42);
        let mut v: Visible<Vec<i32>> = Visible::from(vec![1]);
        v.push(2);
        assert_eq!(v.len(), 2);
        assert_eq!(*v, vec![1, 2]);
        assert_eq!(Invisible(()), Invisible::default());
        assert_eq!(
            std::mem::size_of::<Invisible<u64>>(),
            std::mem::size_of::<u64>()
        );
    }
}
