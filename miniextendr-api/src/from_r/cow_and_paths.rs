//! Cow, PathBuf, OsString, and string collection conversions.
//!
//! - `Cow<'_, [T]>` — owned copy of R native vectors
//! - `Cow<'_, str>` — owned copy of R character scalars
//! - `PathBuf` / `OsString` — from STRSXP via `String` intermediary
//! - `HashSet<String>` / `BTreeSet<String>` — string set conversions
//!
//! # Tradeoff
//!
//! These [`TryFromSexp`] impls reject mismatched
//! [`SEXPTYPE`](crate::SEXPTYPE)s — there is no looser coercion path for `Cow` / `PathBuf` /
//! `OsString`. Raw `Cow` conversions own their data and may outlive the source.
//! [`super::TryFromRoot`] supplies the borrowing path with a root-bound lifetime;
//! generated wrappers use it for explicit `Cow` parameters.

use std::borrow::Cow;
use std::collections::{BTreeSet, HashSet};
use std::ffi::OsString;
use std::path::PathBuf;

use crate::SEXP;
use crate::from_r::{SexpError, SexpTypeError, TryFromSexp, charsxp_to_str, map_strsxp_with};

/// Convert an R native vector to an owned `Cow`.
///
/// A raw SEXP carries no borrow lifetime. Use [`super::TryFromRoot`] to borrow
/// without copying; generated wrappers select that path for `Cow<'_, [T]>`.
impl<T: crate::RNativeType> TryFromSexp for Cow<'_, [T]> {
    type Error = SexpTypeError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        Ok(Cow::Owned(slice.to_vec()))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(Cow::Owned(slice.to_vec()))
    }
}

/// Convert an R character scalar to an owned `Cow`.
///
/// The same scalar/NA semantics as `String`. Borrowing requires
/// [`super::TryFromRoot`], which generated wrappers use for `Cow<'_, str>`.
impl TryFromSexp for Cow<'_, str> {
    type Error = SexpError;
    const CHARACTER_ONLY: bool = true;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        String::try_from_sexp(sexp).map(Cow::Owned)
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { String::try_from_sexp_unchecked(sexp) }.map(Cow::Owned)
    }
}

/// Convert a character vector to owned strings; NA becomes an empty string.
/// Generated wrappers borrow instead through [`super::TryFromRoot`].
impl TryFromSexp for Vec<Cow<'_, str>> {
    type Error = SexpError;
    const CHARACTER_ONLY: bool = true;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        Vec::<String>::try_from_sexp(sexp)
            .map(|values| values.into_iter().map(Cow::Owned).collect())
    }
}

/// Convert a character vector to owned strings, preserving NA as `None`.
/// Generated wrappers borrow instead through [`super::TryFromRoot`].
impl TryFromSexp for Vec<Option<Cow<'_, str>>> {
    type Error = SexpError;
    const CHARACTER_ONLY: bool = true;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        Vec::<Option<String>>::try_from_sexp(sexp)
            .map(|values| values.into_iter().map(|v| v.map(Cow::Owned)).collect())
    }
}

/// Convert R character vector to `Vec<String>`.
///
/// # NA and Encoding Handling
///
/// **Warning:** This conversion is lossy for NA values and encoding failures:
/// - `NA_character_` values are converted to empty string `""`
/// - Encoding translation failures become empty string `""`
/// - Invalid UTF-8 (after translation) becomes empty string `""`
///
/// If you need to preserve NA semantics, use `Vec<Option<String>>` instead:
///
/// ```ignore
/// let strings: Vec<Option<String>> = sexp.try_into()?;
/// // NA values will be None, valid strings will be Some(s)
/// ```
///
/// This design choice prioritizes convenience over strict correctness for the
/// common case where strings are known to be non-NA and properly encoded.
impl TryFromSexp for Vec<String> {
    type Error = SexpError;
    const CHARACTER_ONLY: bool = true;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        map_strsxp_with(sexp, |charsxp, _i| {
            let s = if charsxp == SEXP::na_string() {
                String::new()
            } else {
                unsafe { charsxp_to_str(charsxp) }.to_owned()
            };
            Ok(s)
        })
    }
}

/// Convert R character vector to `Vec<&str>`.
///
/// **Warning:** `NA_character_` values are converted to empty string `""`.
impl TryFromSexp for Vec<&'static str> {
    type Error = SexpError;
    const CHARACTER_ONLY: bool = true;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        map_strsxp_with(sexp, |charsxp, _i| {
            if charsxp == SEXP::na_string() || charsxp == SEXP::blank_string() {
                return Ok("");
            }
            Ok(unsafe { charsxp_to_str(charsxp) })
        })
    }
}

/// Convert R character vector to `Vec<Option<&str>>`.
impl TryFromSexp for Vec<Option<&'static str>> {
    type Error = SexpError;
    const CHARACTER_ONLY: bool = true;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        map_strsxp_with(sexp, |charsxp, _i| {
            if charsxp == SEXP::na_string() {
                return Ok(None);
            }
            if charsxp == SEXP::blank_string() {
                return Ok(Some(""));
            }
            Ok(Some(unsafe { charsxp_to_str(charsxp) }))
        })
    }
}

macro_rules! impl_set_string_try_from_sexp {
    ($(#[$meta:meta])* $set_ty:ident) => {
        $(#[$meta])*
        impl TryFromSexp for $set_ty<String> {
            type Error = SexpError;
            const CHARACTER_ONLY: bool = true;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let vec: Vec<String> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(vec.into_iter().collect())
            }
        }
    };
}

impl_set_string_try_from_sexp!(
    /// Convert R character vector to `HashSet<String>`.
    HashSet
);
impl_set_string_try_from_sexp!(
    /// Convert R character vector to `BTreeSet<String>`.
    BTreeSet
);
// endregion

// region: String-wrapper type conversions (PathBuf, OsString)

/// Generate TryFromSexp impls for types that are `From<String>` (scalar, Option,
/// Vec, `Vec<Option>`). Used for PathBuf and OsString which delegate to String conversion.
macro_rules! impl_string_wrapper_try_from_sexp {
    (
        $(#[$scalar_meta:meta])*
        scalar: $ty:ty;
        $(#[$option_meta:meta])*
        option: $ty2:ty;
        $(#[$vec_meta:meta])*
        vec: $ty3:ty;
        $(#[$vec_option_meta:meta])*
        vec_option: $ty4:ty;
    ) => {
        $(#[$scalar_meta])*
        impl TryFromSexp for $ty {
            type Error = SexpError;
            const CHARACTER_ONLY: bool = true;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let s: String = TryFromSexp::try_from_sexp(sexp)?;
                Ok(<$ty>::from(s))
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let s: String = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(<$ty>::from(s))
            }
        }

        $(#[$option_meta])*
        impl TryFromSexp for Option<$ty> {
            type Error = SexpError;
            const CHARACTER_ONLY: bool = true;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let opt: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(opt.map(<$ty>::from))
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let opt: Option<String> = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(opt.map(<$ty>::from))
            }
        }

        $(#[$vec_meta])*
        impl TryFromSexp for Vec<$ty> {
            type Error = SexpError;
            const CHARACTER_ONLY: bool = true;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let vec: Vec<String> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(vec.into_iter().map(<$ty>::from).collect())
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let vec: Vec<String> = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(vec.into_iter().map(<$ty>::from).collect())
            }
        }

        $(#[$vec_option_meta])*
        impl TryFromSexp for Vec<Option<$ty>> {
            type Error = SexpError;
            const CHARACTER_ONLY: bool = true;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let vec: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(vec.into_iter().map(|opt| opt.map(<$ty>::from)).collect())
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let vec: Vec<Option<String>> = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(vec.into_iter().map(|opt| opt.map(<$ty>::from)).collect())
            }
        }
    };
}

impl_string_wrapper_try_from_sexp!(
    /// Convert R character scalar (STRSXP of length 1) to `PathBuf`.
    ///
    /// # NA Handling
    ///
    /// **Warning:** `NA_character_` is converted to empty path `""`. This is lossy!
    /// If you need to distinguish between NA and empty strings, use `Option<PathBuf>` instead.
    scalar: PathBuf;
    /// NA-aware PathBuf conversion: returns `None` for `NA_character_` or `NULL`.
    option: PathBuf;
    /// Convert R character vector (STRSXP) to `Vec<PathBuf>`.
    ///
    /// # NA Handling
    ///
    /// **Warning:** `NA_character_` elements are converted to empty paths.
    /// Use `Vec<Option<PathBuf>>` if you need to preserve NA values.
    vec: PathBuf;
    /// Convert R character vector (STRSXP) to `Vec<Option<PathBuf>>` with NA support.
    ///
    /// `NA_character_` elements are converted to `None`.
    vec_option: PathBuf;
);

impl_string_wrapper_try_from_sexp!(
    /// Convert R character scalar (STRSXP of length 1) to `OsString`.
    ///
    /// Since R strings are converted to UTF-8, the resulting `OsString` contains
    /// valid UTF-8 data.
    ///
    /// # NA Handling
    ///
    /// **Warning:** `NA_character_` is converted to empty string. This is lossy!
    /// If you need to distinguish between NA and empty strings, use `Option<OsString>` instead.
    scalar: OsString;
    /// NA-aware OsString conversion: returns `None` for `NA_character_` or `NULL`.
    option: OsString;
    /// Convert R character vector (STRSXP) to `Vec<OsString>`.
    ///
    /// # NA Handling
    ///
    /// **Warning:** `NA_character_` elements are converted to empty strings.
    /// Use `Vec<Option<OsString>>` if you need to preserve NA values.
    vec: OsString;
    /// Convert R character vector (STRSXP) to `Vec<Option<OsString>>` with NA support.
    ///
    /// `NA_character_` elements are converted to `None`.
    vec_option: OsString;
);
// endregion
