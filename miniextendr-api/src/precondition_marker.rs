//! Precondition markers: [`Checked<T>`] and [`Unchecked<T>`].
//!
//! A generated R wrapper checks each argument's R type before the `.Call()`
//! (`is.integer(n)`, `length(n) == 1L`, ...), one guard per check, so a
//! wrong argument fails in R with a message naming it. These are the
//! *type-derived* checks: they come from the parameter's Rust type. Without
//! them the Rust conversion still refuses bad input, with the same
//! argument-error condition (`e$param`, `e$rust_type`, `kind =
//! "conversion"`); only the message is the conversion's. Dropping them saves
//! one `isTRUE()` guard per check on a hot path.
//!
//! Whether a parameter keeps them is decided per parameter, the most specific
//! spelling first:
//!
//! | rank | spelling | where |
//! |------|----------|-------|
//! | 1 | `x: Checked<T>` / `x: Unchecked<T>` | a parameter of a function or inherent-impl method |
//! | 1 | `#[miniextendr(preconditions)]` / `#[miniextendr(no_preconditions)]` on the parameter | a function parameter |
//! | 1 | `preconditions(x, y)` / `no_preconditions(x)` | a method attribute (inherent and trait impls) |
//! | 2 | `preconditions` / `no_preconditions` | the function or method attribute |
//! | 3 | `preconditions` / `no_preconditions` | the impl-block attribute |
//! | 4 | `preconditions = true \| false` | `[package.metadata.miniextendr]` in `Cargo.toml` |
//! | 5 | the `no-preconditions-default` cargo feature | the whole build |
//! | 6 | on | the framework default |
//!
//! ```ignore
//! use miniextendr_api::{Checked, List, Unchecked, miniextendr};
//!
//! /// `n_iter` keeps its guards even when the crate default drops them.
//! #[miniextendr(no_preconditions)]
//! pub fn fit(data: List, n_iter: Checked<i32>, tol: f64) -> f64 {
//!     let n = *n_iter; // or n_iter.into_inner()
//!     // ...
//! #   let _ = (data, n, tol); 0.0
//! }
//!
//! /// `xs` skips `is.double(xs)`; `factor` keeps its guards.
//! #[miniextendr]
//! pub fn scale_by(factor: f64, xs: Unchecked<Vec<f64>>) -> Vec<f64> {
//!     xs.into_inner().into_iter().map(|x| x * factor).collect()
//! }
//! ```
//!
//! The switch covers the type-derived checks only. The checks an author names
//! (`inherits`, `no_na`), the `match_arg` / `choices` validation and the Rust
//! conversion stay whatever the spelling. A marker and a keyword that
//! disagree on one parameter are a compile error; two keywords follow the
//! pair rule, the last one written wins.
//!
//! The marker goes outermost (`Checked<Option<i32>>`, not
//! `Option<Checked<i32>>`), at most one per parameter, and only on a
//! parameter that has a type-derived check to keep or drop (not `SEXP`,
//! `Missing<T>`, `ExternalPtr<T>`, `&Dots`, a choice parameter, ...). It
//! cannot appear in a `#[miniextendr]` trait's method signature (spell it on
//! the impl, `#[miniextendr(preconditions(x))]`) nor on an
//! `extern "C-unwind"` function, which has no generated conversion to unwrap
//! it.
//!
//! # Not a runtime type
//!
//! The guards are R text the macro writes before any type is resolved, so
//! only the syntax `Checked<..>` / `Unchecked<..>` on the parameter selects
//! them. The markers therefore implement neither `TryFromSexp` nor `IntoR`:
//! the macro converts the inner `T` and wraps it (`from_inner`) before the
//! call. A type alias (`type N = Checked<i32>`) or a rename is not seen as a
//! marker, and fails to compile (E0277, no `TryFromSexp`) instead of silently
//! following the crate default. The return-visibility markers
//! (`Invisible<T>`, `Visible<T>`) differ: they forward `IntoR`, because a
//! missed visibility marker still has a correct value to convert.
//!
//! # Name
//!
//! `Checked` means "the R-side guards are kept". It is not an
//! overflow-checked conversion, and not the thread-checked FFI variants
//! (`#[r_ffi_checked]`).

/// Parameter marker: keep this parameter's type-derived R-side checks,
/// whatever the function, impl or crate default says. Transparent over `T`;
/// see the [module docs](self).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Checked<T>(pub T);

/// Parameter marker: drop this parameter's type-derived R-side checks, and
/// let the Rust conversion word a bad argument. Transparent over `T`; see the
/// [module docs](self).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Unchecked<T>(pub T);

macro_rules! precondition_marker_impls {
    ($name:ident) => {
        impl<T> $name<T> {
            /// Wrap a converted argument. Generated code calls this after the
            /// conversion of the inner `T`; user code can build the marker
            /// directly (`Checked(x)`).
            #[doc(hidden)]
            #[inline]
            pub const fn from_inner(value: T) -> Self {
                $name(value)
            }

            /// The wrapped value.
            #[inline]
            pub fn into_inner(self) -> T {
                self.0
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
    };
}

precondition_marker_impls!(Checked);
precondition_marker_impls!(Unchecked);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_are_transparent_over_the_value() {
        assert_eq!(std::mem::size_of::<Checked<f64>>(), 8);
        assert_eq!(std::mem::size_of::<Unchecked<u16>>(), 2);
        let mut n = Checked::from_inner(3_i32);
        *n += 1;
        assert_eq!(*n, 4);
        assert_eq!(n.into_inner(), 4);
        let s = Unchecked::from_inner("a");
        assert_eq!(s.len(), 1);
        assert_eq!(s.into_inner(), "a");
    }
}
