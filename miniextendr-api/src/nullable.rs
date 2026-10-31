//! [`Nullable<T>`]: an optional value whose `None` is `NULL` in R.
//!
//! An `Option<T>` return converts through `Option<T>`'s own [`IntoR`]: `None`
//! is `NULL` for a vector, list or class, but `NA` for a scalar
//! (`Option<i32>` gives `NA_integer_`), because R stores a missing scalar as
//! `NA`. Where `None` means that there is no value at all, whatever `T` is,
//! return `Nullable<T>`: `Some(v)` converts as `v` does, and `None` is `NULL`.
//! [`RIterator`] uses it for the end of iteration, which an `NA` item would be
//! indistinguishable from.
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
//! It reads from R as `Option<T>` does ([`TryFromSexp`]): `NULL` and `NA`
//! are `None`. That makes it an `#[r_data]` field type of an
//! `#[derive(ExternalPtr)]` struct, whose getter reads `NULL` for `None` and
//! whose setter stores `None` for `NULL` (#2270); an `Option<T>` field reads
//! `NA` for a scalar `None`, unless it is marked `#[r_data(null_if_none)]`.
//! An optional argument is usually spelled `Option<T>`, which reads the same.
//!
//! With the `serde` feature, `Nullable<T>` serializes and deserializes as
//! `Option<T>` does, so a serde struct can hold one.

use crate::SEXP;
use crate::from_r::{NativeBorrow, TryFromSexp};
use crate::into_r::IntoR;

/// An optional value that R receives as `NULL` when `None`, whatever `T` is,
/// and that reads `NULL` or `NA` from R as `None`. See the
/// [module docs](self).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Nullable<T>(pub Option<T>);

impl<T> From<Option<T>> for Nullable<T> {
    #[inline]
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

// region: IntoR

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
// endregion

// region: TryFromSexp

/// Reads as `Option<T>` does: `NULL` and `NA` are `None` (#2270).
impl<T> TryFromSexp for Nullable<T>
where
    Option<T>: TryFromSexp,
{
    const NATIVE_BORROW: Option<NativeBorrow> = <Option<T> as TryFromSexp>::NATIVE_BORROW;
    const CHARACTER_ONLY: bool = <Option<T> as TryFromSexp>::CHARACTER_ONLY;
    #[doc(hidden)]
    const __MX_EXPECTATION: Option<&'static str> = <Option<T> as TryFromSexp>::__MX_EXPECTATION;

    type Error = <Option<T> as TryFromSexp>::Error;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        <Option<T> as TryFromSexp>::try_from_sexp(sexp).map(Self)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { <Option<T> as TryFromSexp>::try_from_sexp_unchecked(sexp) }.map(Self)
    }

    #[inline]
    unsafe fn __mx_try_from_rooted(root: crate::Root<'_>) -> Result<Self, Self::Error> {
        unsafe { <Option<T> as TryFromSexp>::__mx_try_from_rooted(root) }.map(Self)
    }

    #[inline]
    fn __mx_has_na(&self) -> bool {
        self.0.__mx_has_na()
    }

    #[inline]
    fn __mx_input_has_na(&self, input: SEXP) -> bool {
        self.0.__mx_input_has_na(input)
    }
}
// endregion

// region: serde

/// Serializes as `Option<T>` does.
#[cfg(feature = "serde")]
impl<T: ::serde::Serialize> ::serde::Serialize for Nullable<T> {
    #[inline]
    fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

/// Deserializes as `Option<T>` does.
#[cfg(feature = "serde")]
impl<'de, T: ::serde::Deserialize<'de>> ::serde::Deserialize<'de> for Nullable<T> {
    #[inline]
    fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<T>::deserialize(deserializer).map(Self)
    }
}
// endregion
