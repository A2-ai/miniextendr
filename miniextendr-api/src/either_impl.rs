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
//! 3. If both fail, returns the error from attempting `R`
//!
//! This "try left first" strategy means the order of type parameters matters!
//!
//! ## From Rust to R (`IntoR`)
//!
//! Each variant is converted to R using its inner type's `IntoR` implementation.
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
        if let Ok(left) = L::try_from_sexp(sexp) {
            return Ok(Either::Left(left));
        }
        // Fall back to R
        R::try_from_sexp(sexp)
            .map(Either::Right)
            .map_err(Into::into)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        // Try L first
        if let Ok(left) = unsafe { L::try_from_sexp_unchecked(sexp) } {
            return Ok(Either::Left(left));
        }
        // Fall back to R
        unsafe { R::try_from_sexp_unchecked(sexp) }
            .map(Either::Right)
            .map_err(Into::into)
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
