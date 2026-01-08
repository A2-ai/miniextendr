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
//! // f64 vs i32 - R integers can coerce to doubles!
//! // If you pass an integer, it converts to Left(f64) not Right(i32)
//! Either<f64, i32>  // Consider: Either<i32, f64> instead
//!
//! // Vec<f64> vs Vec<i32> - same issue with coercion
//! Either<Vec<f64>, Vec<i32>>  // Consider: Either<Vec<i32>, Vec<f64>>
//! ```
//!
//! **Rule of thumb**: If `L` can successfully parse values intended for `R`,
//! swap the order so the more restrictive type is tried first.
//!
//! ## Ambiguous Pairs Quick Reference
//!
//! | Ambiguous (avoid) | Safe Alternative | Why |
//! |-------------------|------------------|-----|
//! | `Either<f64, i32>` | `Either<i32, f64>` | R integers coerce to doubles |
//! | `Either<Vec<f64>, Vec<i32>>` | `Either<Vec<i32>, Vec<f64>>` | Same coercion issue |
//! | `Either<f64, bool>` | `Either<bool, f64>` | Booleans coerce to numeric |
//! | `Either<i32, bool>` | `Either<bool, i32>` | Booleans coerce to integer |
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

use crate::ffi::SEXP;
use crate::from_r::{SexpError, TryFromSexp};
use crate::into_r::IntoR;

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
                left_error: left_err.to_string(),
                right_error: right_err.into().to_string(),
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
                left_error: left_err.to_string(),
                right_error: right_err.into().to_string(),
            }),
        }
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
    #[inline]
    fn into_sexp(self) -> SEXP {
        match self {
            Either::Left(l) => l.into_sexp(),
            Either::Right(r) => r.into_sexp(),
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> SEXP {
        match self {
            Either::Left(l) => unsafe { l.into_sexp_unchecked() },
            Either::Right(r) => unsafe { r.into_sexp_unchecked() },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn either_can_be_created() {
        let _left: Either<i32, String> = Left(42);
        let _right: Either<i32, String> = Right("hello".to_string());
    }
}
