//! Conversions for types parsed from an R string: [`ParseRStr`],
//! [`ParsedRStr`] and [`try_from_sexp_via_str_parse!`](crate::try_from_sexp_via_str_parse).
//!
//! A string-parsed type `T` gets its four conversions (`T`, `Option<T>`,
//! `Vec<T>`, `Vec<Option<T>>`) from the [`ParsedRStr<T>`] impls in this module.
//! The container shapes reach `T` through the
//! [`TryFromSexpElement`](crate::TryFromSexpElement) blankets, with
//! `ParsedRStr<T>` as the inner type, so they compile in any crate (#1766).

use super::{BatchedErrors, SexpError, SexpNaError, TryFromSexp};
use crate::{SEXP, SEXPTYPE};

/// Parse hook for a type read from an R character vector.
///
/// Implemented by [`try_from_sexp_via_str_parse!`](crate::try_from_sexp_via_str_parse),
/// together with the `TryFromSexp` and
/// [`TryFromSexpElement`](crate::TryFromSexpElement) impls that give the type
/// its four conversions. Use the macro rather than implementing this by hand.
pub trait ParseRStr: Sized {
    /// What a failed parse calls the value: `invalid <LABEL>: <error>`.
    const LABEL: &'static str;

    /// Parse one string (never `NA`).
    ///
    /// The error is formatted only when it is reported, so a long vector
    /// that fails everywhere builds at most the ten messages a batched error
    /// lists.
    fn parse_r_str(s: &str) -> Result<Self, impl std::fmt::Display>;
}

/// A value of the string-parsed type `T`: the inner type of `T`'s
/// [`TryFromSexpElement`](crate::TryFromSexpElement) impl.
///
/// `Vec<T>`, `Option<T>` and `Vec<Option<T>>` read the same container of
/// `ParsedRStr<T>`, whose conversions here apply the NA policy and batch the
/// element errors. Plumbing for
/// [`try_from_sexp_via_str_parse!`](crate::try_from_sexp_via_str_parse).
pub struct ParsedRStr<T>(pub T);

/// `invalid <label>: <error>`, the reason a parse failure gives.
fn invalid<T: ParseRStr>(err: impl std::fmt::Display) -> String {
    format!("invalid {}: {err}", T::LABEL)
}

/// `NA_character_` / `NULL` → `SexpError::Na`; a parse failure →
/// `InvalidValue("invalid <label>: <error>")`.
impl<T: ParseRStr> TryFromSexp for ParsedRStr<T> {
    type Error = SexpError;
    const CHARACTER_ONLY: bool = true;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        let s: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
        let s = s.ok_or(SexpError::Na(SexpNaError {
            sexp_type: SEXPTYPE::STRSXP,
        }))?;
        T::parse_r_str(&s)
            .map(ParsedRStr)
            .map_err(|e| SexpError::InvalidValue(invalid::<T>(e)))
    }
}

/// `NA_character_` / `NULL` → `None`; a parse failure as for the scalar.
impl<T: ParseRStr> TryFromSexp for Option<ParsedRStr<T>> {
    type Error = SexpError;
    const CHARACTER_ONLY: bool = true;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        let s: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
        match s {
            None => Ok(None),
            Some(s) => T::parse_r_str(&s)
                .map(|v| Some(ParsedRStr(v)))
                .map_err(|e| SexpError::InvalidValue(invalid::<T>(e))),
        }
    }
}

/// `NA` elements and parse failures are collected across the whole vector
/// into one batched `InvalidValue`, each reason followed by the
/// 1-based positions that failed with it:
/// `NA is not allowed (element 2); invalid <label>: <error> (elements 3, 5)`.
impl<T: ParseRStr> TryFromSexp for Vec<ParsedRStr<T>> {
    type Error = SexpError;
    const CHARACTER_ONLY: bool = true;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        let values: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
        let mut result = Vec::with_capacity(values.len());
        let mut errors = BatchedErrors::default();
        for (i, s) in values.into_iter().enumerate() {
            match s {
                None => errors.push(i, || "NA is not allowed".to_string()),
                Some(s) => match T::parse_r_str(&s) {
                    Ok(v) => result.push(ParsedRStr(v)),
                    Err(e) => errors.push(i, || invalid::<T>(e)),
                },
            }
        }
        if errors.is_empty() {
            Ok(result)
        } else {
            Err(errors.into_element_error())
        }
    }
}

/// `NA` elements → `None`; parse failures batch as for `Vec<T>`.
impl<T: ParseRStr> TryFromSexp for Vec<Option<ParsedRStr<T>>> {
    type Error = SexpError;
    const CHARACTER_ONLY: bool = true;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        let values: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
        let mut result = Vec::with_capacity(values.len());
        let mut errors = BatchedErrors::default();
        for (i, s) in values.into_iter().enumerate() {
            match s {
                None => result.push(None),
                Some(s) => match T::parse_r_str(&s) {
                    Ok(v) => result.push(Some(ParsedRStr(v))),
                    Err(e) => errors.push(i, || invalid::<T>(e)),
                },
            }
        }
        if errors.is_empty() {
            Ok(result)
        } else {
            Err(errors.into_element_error())
        }
    }
}

/// Give a type parsed from an R string its four `TryFromSexp` conversions:
/// `T`, `Option<T>`, `Vec<T>` and `Vec<Option<T>>`.
///
/// Expands in any crate. It implements
/// [`ParseRStr`] (the parse step),
/// `TryFromSexp` for the type, and
/// [`TryFromSexpElement`](crate::TryFromSexpElement), through which the
/// container blankets in `miniextendr-api` convert `Option<T>`, `Vec<T>` and
/// `Vec<Option<T>>`. These are all impls on your own type, so the orphan rule
/// allows them; the container impls themselves can only live in
/// `miniextendr-api` (#1766).
///
/// The built-in uuid, url, regex and num-bigint conversions use this macro, so
/// a type given it reads R input the same way:
///
/// - `T`: `NA_character_` / `NULL` → `SexpError::Na`; a parse failure →
///   `InvalidValue("invalid <label>: <err>")`.
/// - `Option<T>`: `NA_character_` / `NULL` → `None`.
/// - `Vec<T>`: `NA` elements and parse failures are collected across the
///   whole vector into one batched `InvalidValue`, each reason followed by
///   the 1-based positions that failed with it:
///   `NA is not allowed (element 2); invalid <label>: <err> (elements 3, 5)`.
///   The first 10 failures are listed and the remainder is summarized as
///   `"and N more"`.
/// - `Vec<Option<T>>`: `NA` elements → `None`; parse failures batch as above.
///
/// All four read only character input (`TryFromSexp::CHARACTER_ONLY`).
///
/// The parse body is a closure-style `|s| expr` where `s: &str`, returning
/// `Result<T, E>` with `E: Display`.
///
/// ```ignore
/// pub struct Slug(String);
///
/// impl std::str::FromStr for Slug {
///     type Err = &'static str;
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         if s.is_empty() || !s.chars().all(|c| c.is_ascii_lowercase() || c == '-') {
///             return Err("expected lowercase letters and '-'");
///         }
///         Ok(Slug(s.to_owned()))
///     }
/// }
///
/// miniextendr_api::try_from_sexp_via_str_parse!(Slug, "slug", |s| s.parse::<Slug>());
///
/// #[miniextendr]
/// pub fn slugs(x: Vec<Option<Slug>>) -> i32 { /* ... */ }
/// ```
///
/// The type must not be generic, and the macro must not be used on a type
/// that also derives `TryFromSexp`: both implement `TryFromSexpElement`.
#[macro_export]
macro_rules! try_from_sexp_via_str_parse {
    ($ty:ty, $label:literal, |$s:ident| $parse:expr) => {
        impl $crate::from_r::ParseRStr for $ty {
            const LABEL: &'static str = $label;

            fn parse_r_str($s: &str) -> ::core::result::Result<$ty, impl ::core::fmt::Display> {
                $parse
            }
        }

        impl $crate::from_r::TryFromSexp for $ty {
            type Error = $crate::from_r::SexpError;
            const CHARACTER_ONLY: bool = true;

            fn try_from_sexp(
                sexp: $crate::SEXP,
            ) -> ::core::result::Result<$ty, $crate::from_r::SexpError> {
                <$crate::from_r::ParsedRStr<$ty> as $crate::from_r::TryFromSexp>::try_from_sexp(
                    sexp,
                )
                .map(|parsed| parsed.0)
            }
        }

        impl $crate::TryFromSexpElement for $ty {
            type Inner = $crate::from_r::ParsedRStr<$ty>;

            #[inline]
            fn from_inner(inner: $crate::from_r::ParsedRStr<$ty>) -> $ty {
                inner.0
            }
        }
    };
}
