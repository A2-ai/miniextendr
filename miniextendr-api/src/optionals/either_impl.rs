//! Integration with the `either` crate.
//!
//! This module provides [`TryFromSexp`] and [`IntoR`] implementations for
//! [`Either<L, R>`], allowing sum types to be passed between R and Rust.
//!
//! # Features
//!
//! Enable this module with the `either` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["either"] }
//! ```
//!
//! # Conversion Behavior
//!
//! ## From R to Rust (`TryFromSexp`)
//!
//! When converting an R value to `Either<L, R>`:
//! 1. First attempts to convert to `L`
//! 2. If that fails, attempts to convert to `R`
//! 3. If both fail, returns an error containing both failure reasons
//!
//! This "try left first" strategy means the order of type parameters matters!
//!
//! `Option<Either<L, R>>` reads `NULL` as `None` before either arm is tried;
//! any other value converts as `Either<L, R>` (same arm order, same error)
//! and becomes `Some`. A `no_na` parameter of that type is checked by the arm
//! the value converted to, like a bare `Either`, and `None` passes.
//!
//! A `match_arg` / `choices` parameter typed `Either<T, R>` (or, with
//! `several_ok`, `Either<Vec<T>, R>` / `Either<Box<[T]>, R>`) does not use
//! this impl: its C wrapper decodes with
//! [`match_arg_either_or`](crate::match_arg::match_arg_either_or), which sends
//! character and factor input to the choice side and everything else to `R`,
//! the same split the R wrapper's `match.arg()` check makes.
//!
//! ## Native borrows
//!
//! An arm that borrows R's native storage (`&mut [f64]`, `&i32`, ...) makes
//! the `Either` borrow it, but only when that arm converts. A generated
//! wrapper's alias check takes both arms; when one of them meets another
//! argument, the wrapper prepares the `Either` to learn the arm its conversion
//! takes (left first, as below) and checks that arm alone, before any
//! reference exists (#1504). So `Either<i32, &mut [f64]>` next to a `&[f64]`
//! of the same vector is refused only when the `i32` arm does not take the
//! vector. Under `Option`, `Missing`, `Result<_, ()>`, `Nullable` or a
//! derived newtype the arm is still learned. Inside a list, tuple or map
//! the check takes both arms (#2340), and so it does for an `Option` of a
//! derived newtype and for any type whose hand-written `TryFromSexp` doesn't
//! forward the hidden preparation hooks: refused more often than needed,
//! never less.
//!
//! ## From Rust to R (`IntoR`)
//!
//! Each variant is converted to R using its inner type's `IntoR` implementation.
//!
//! # Avoiding Ambiguous Eithers
//!
//! The order of type parameters affects conversion behavior. For best results:
//!
//! - **Put the more specific type first** (as `L`)
//! - **Put the more general type second** (as `R`)
//!
//! ## Good patterns
//!
//! ```ignore
//! // Integer vs String - disjoint R types, unambiguous
//! Either<i32, String>
//!
//! // ExternalPtr vs primitive - ExternalPtr is checked first
//! Either<ExternalPtr<MyType>, i32>
//! ```
//!
//! ## Potentially problematic patterns
//!
//! ```ignore
//! // AsNumeric vs i32 - AsNumeric reads R integers too!
//! // If you pass an integer, it converts to Left(AsNumeric) not Right(i32)
//! Either<AsNumeric, i32>  // Consider: Either<i32, AsNumeric> instead
//!
//! // i64 vs bool - i64 reads R logicals too
//! Either<i64, bool>  // Consider: Either<bool, i64>
//! ```
//!
//! **Rule of thumb**: If `L` can successfully parse values intended for `R`,
//! swap the order so the more restrictive type is tried first.
//!
//! Bare `f64`, `i32` and `bool` (and their `Vec`s) are strict: each reads one
//! R type (`f64` only doubles, `i32` only integers), so they never overlap.
//! Only a widening arm does: [`AsNumeric`](crate::AsNumeric) /
//! [`AsNumericVec`](crate::AsNumericVec), or a scalar `i64`, `u32`, `f32` and
//! the like, which read integer, double, logical and raw input.
//! `#[miniextendr(coerce)]` / `coerce-default` do not widen an `Either`'s arms.
//!
//! ## Ambiguous Pairs Quick Reference
//!
//! | Ambiguous (avoid) | Safe Alternative | Why |
//! |-------------------|------------------|-----|
//! | `Either<f64, i32>` | OK as-is | Disjoint R types: a bare `f64` reads only doubles |
//! | `Either<Vec<f64>, Vec<i32>>` | OK as-is | Disjoint R types |
//! | `Either<f64, bool>` | OK as-is | Disjoint R types: a bare `f64` refuses logicals |
//! | `Either<i32, bool>` | OK as-is | Disjoint R types: a bare `i32` refuses logicals |
//! | `Either<AsNumeric, i32>` | `Either<i32, AsNumeric>` | `AsNumeric` reads integers too |
//! | `Either<i64, bool>` | `Either<bool, i64>` | `i64` reads logicals too |
//! | `Either<String, i32>` | OK as-is | Disjoint R types |
//! | `Either<Vec<T>, T>` | `Either<T, Vec<T>>` | Scalar matches length-1 vector |
//!
//! # Example
//!
//! ```ignore
//! use either::Either;
//!
//! #[miniextendr]
//! fn process_input(val: Either<i32, String>) -> String {
//!     match val {
//!         Either::Left(n) => format!("Got integer: {}", n),
//!         Either::Right(s) => format!("Got string: {}", s),
//!     }
//! }
//! ```

pub use either::{Either, Left, Right};

use crate::from_r::prepared::{Arms, Prepared};
use crate::from_r::{NativeBorrow, SexpError, TryFromSexp};
use crate::into_r::IntoR;
use crate::{SEXP, SEXPTYPE, SexpExt};

/// Implements `TryFromSexp` for `Either<L, R>`.
///
/// Attempts to parse as `L` first, falling back to `R` if that fails.
/// If both conversions fail, returns an error containing both failure reasons.
impl<L, R> TryFromSexp for Either<L, R>
where
    L: TryFromSexp,
    R: TryFromSexp,
    L::Error: Into<SexpError>,
    R::Error: Into<SexpError>,
{
    type Error = SexpError;
    // Either arm may borrow: the alias check takes both, then the arm the
    // conversion takes when it matters (#1504).
    const NATIVE_BORROW: Option<NativeBorrow> =
        NativeBorrow::either(&[L::NATIVE_BORROW, R::NATIVE_BORROW]);
    // Only character input when both sides read only character input.
    const CHARACTER_ONLY: bool = L::CHARACTER_ONLY && R::CHARACTER_ONLY;
    // `no_na` asks whichever side the value converted to.
    #[inline]
    fn __mx_has_na(&self) -> bool {
        match self {
            Either::Left(l) => L::__mx_has_na(l),
            Either::Right(r) => R::__mx_has_na(r),
        }
    }

    // An `Either` parameter has no R guard: the side the value converted to
    // reads the input, so a data frame arm keeps its NA cells.
    #[inline]
    fn __mx_input_has_na(&self, input: SEXP) -> bool {
        match self {
            Either::Left(l) => L::__mx_input_has_na(l, input),
            Either::Right(r) => R::__mx_input_has_na(r, input),
        }
    }

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // Try L first
        let left_err = match L::try_from_sexp(sexp) {
            Ok(left) => return Ok(Either::Left(left)),
            Err(e) => e.into(),
        };

        // Fall back to R
        match R::try_from_sexp(sexp) {
            Ok(right) => Ok(Either::Right(right)),
            Err(right_err) => Err(SexpError::EitherConversion {
                left_error: Box::new(left_err),
                right_error: Box::new(right_err.into()),
            }),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        // Try L first
        let left_err = match unsafe { L::try_from_sexp_unchecked(sexp) } {
            Ok(left) => return Ok(Either::Left(left)),
            Err(e) => e.into(),
        };

        // Fall back to R
        match unsafe { R::try_from_sexp_unchecked(sexp) } {
            Ok(right) => Ok(Either::Right(right)),
            Err(right_err) => Err(SexpError::EitherConversion {
                left_error: Box::new(left_err),
                right_error: Box::new(right_err.into()),
            }),
        }
    }

    // Learn the arm the conversion takes without building a reference: left
    // first, as `try_from_sexp`. An owned arm's value is kept, so it converts
    // once; a borrowing arm is deferred with its path. An arm that can't tell
    // leaves the `Either` undecided, checked against every arm it may take.
    fn __mx_prepare(sexp: SEXP) -> Prepared<Self> {
        let left_error = match L::__mx_prepare(sexp) {
            Prepared::Failed(e) => e.into(),
            left @ (Prepared::Ready(_) | Prepared::Deferred { .. }) => {
                return left.arm(false, Either::Left, Self::NATIVE_BORROW);
            }
            Prepared::Unprepared | Prepared::Undecided(_) => {
                return Prepared::Undecided(Self::NATIVE_BORROW);
            }
        };
        // The left arm failed. A decided right arm is materialized on its
        // own, so the left arm never runs again.
        match R::__mx_prepare(sexp) {
            Prepared::Failed(right_error) => Prepared::Failed(SexpError::EitherConversion {
                left_error: Box::new(left_error),
                right_error: Box::new(right_error.into()),
            }),
            right @ (Prepared::Ready(_) | Prepared::Deferred { .. }) => {
                right.arm(true, Either::Right, Self::NATIVE_BORROW)
            }
            // An undecided right arm converts the whole `Either` again, left
            // arm first, and a check in user code (a newtype's `validate`)
            // may pass the second time: both arms stay in the check.
            Prepared::Unprepared | Prepared::Undecided(_) => {
                Prepared::Undecided(Self::NATIVE_BORROW)
            }
        }
    }

    fn __mx_materialize(sexp: SEXP, arms: Arms) -> Result<Self, Self::Error> {
        match arms.pop() {
            (false, arms) => L::__mx_materialize(sexp, arms)
                .map(Either::Left)
                .map_err(Into::into),
            (true, arms) => R::__mx_materialize(sexp, arms)
                .map(Either::Right)
                .map_err(Into::into),
        }
    }
}

/// Implements `TryFromSexp` for `Option<Either<L, R>>`.
///
/// `NULL` → `None`, before either arm is tried; any other value converts as
/// an [`Either<L, R>`] (left first, the same error when both arms fail) and
/// becomes `Some`. So an arm that would itself accept `NULL` never sees it.
///
/// Concrete rather than left to the newtype blanket `Option<T>` impl
/// (`newtype.rs`), which `Either` does not match: without this impl,
/// `Option<Either<L, R>>: TryFromSexp` fails with E0275 (overflow) instead
/// of converting.
impl<L, R> TryFromSexp for Option<Either<L, R>>
where
    Either<L, R>: TryFromSexp,
{
    type Error = <Either<L, R> as TryFromSexp>::Error;
    const NATIVE_BORROW: Option<crate::from_r::NativeBorrow> =
        <Either<L, R> as TryFromSexp>::NATIVE_BORROW;
    const CHARACTER_ONLY: bool = <Either<L, R> as TryFromSexp>::CHARACTER_ONLY;
    // `__MX_EXPECTATION` keeps its `None`: the inner text without `NULL or`
    // would be wrong, and `#[miniextendr]` words an `Option<_>` parameter or
    // arm itself (`NULL or <T>`).

    // `NULL` (`None`) is "not given" and passes `no_na`.
    #[inline]
    fn __mx_has_na(&self) -> bool {
        self.as_ref()
            .is_some_and(<Either<L, R> as TryFromSexp>::__mx_has_na)
    }

    // A given value is read by the arm it converted to (a data frame arm
    // keeps its `NA` cells); `NULL` holds no `NA`.
    #[inline]
    fn __mx_input_has_na(&self, input: SEXP) -> bool {
        match self {
            Some(either) => <Either<L, R> as TryFromSexp>::__mx_input_has_na(either, input),
            None => false,
        }
    }

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            <Either<L, R> as TryFromSexp>::try_from_sexp(sexp).map(Some)
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            unsafe { <Either<L, R> as TryFromSexp>::try_from_sexp_unchecked(sexp) }.map(Some)
        }
    }

    // `NULL` is `None` before either arm is tried; anything else prepares
    // as the `Either`.
    fn __mx_prepare(sexp: SEXP) -> Prepared<Self> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Prepared::Ready(None)
        } else {
            <Either<L, R> as TryFromSexp>::__mx_prepare(sexp).map(Some, |e| e)
        }
    }

    fn __mx_materialize(sexp: SEXP, arms: Arms) -> Result<Self, Self::Error> {
        <Either<L, R> as TryFromSexp>::__mx_materialize(sexp, arms).map(Some)
    }
}

/// Implements `IntoR` for `Either<L, R>`.
///
/// Converts the inner value to R using the appropriate variant's `IntoR` impl.
impl<L, R> IntoR for Either<L, R>
where
    L: IntoR,
    R: IntoR,
{
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(match self {
            Either::Left(l) => l.into_sexp(),
            Either::Right(r) => r.into_sexp(),
        })
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(match self {
            Either::Left(l) => unsafe { l.into_sexp_unchecked() },
            Either::Right(r) => unsafe { r.into_sexp_unchecked() },
        })
    }
}

#[cfg(test)]
mod tests;
