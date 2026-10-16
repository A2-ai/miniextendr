//! [`Nullable<T>`]: an optional return whose `None` is `NULL` in R.
//!
//! An `Option<T>` return converts through `Option<T>`'s own [`IntoR`]: `None`
//! is `NULL` for a vector, list or class, but `NA` for a scalar
//! (`Option<i32>` gives `NA_integer_`), because R stores a missing scalar as
//! `NA`. Where `None` means that there is no value at all, whatever `T` is,
//! return `Nullable<T>`: `Some(v)` converts as `v` does, and `None` is `NULL`.
//! [`RIterator`](crate::adapter_traits::RIterator) uses it for the end of
//! iteration, which an `NA` item would be indistinguishable from.
//!
//! ```ignore
//! use miniextendr_api::{miniextendr, Nullable};
//!
//! #[miniextendr]
//! pub fn first_even(xs: Vec<i32>) -> Nullable<i32> {
//!     Nullable(xs.into_iter().find(|x| x % 2 == 0))
//! }
//! // R: first_even(c(1L, 4L)) → 4L
//! //    first_even(c(1L, 3L)) → NULL  (an `Option<i32>` return gives NA_integer_)
//! ```
//!
//! `Nullable<T>` is a return type only; an optional argument is `Option<T>`,
//! which already reads `NULL` as `None`.

use crate::SEXP;
use crate::into_r::IntoR;

/// An optional return value that R receives as `NULL` when `None`, whatever
/// `T` is. See the [module docs](self).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Nullable<T>(pub Option<T>);

impl<T> From<Option<T>> for Nullable<T> {
    #[inline]
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

impl<T: IntoR> IntoR for Nullable<T> {
    type Error = T::Error;
    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        match self.0 {
            Some(v) => v.try_into_sexp(),
            None => Ok(SEXP::nil()),
        }
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        match self.0 {
            Some(v) => unsafe { v.try_into_sexp_unchecked() },
            None => Ok(SEXP::nil()),
        }
    }
    #[inline]
    fn into_sexp(self) -> SEXP {
        match self.0 {
            Some(v) => v.into_sexp(),
            None => SEXP::nil(),
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> SEXP {
        match self.0 {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => SEXP::nil(),
        }
    }
}
