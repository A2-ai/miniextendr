//! Cow, PathBuf, OsString, and string collection conversions.
//!
//! - `Cow<'static, [T]>` — zero-copy borrow of R native vectors
//! - `Cow<'static, str>` — zero-copy borrow of R character scalars
//! - `PathBuf` / `OsString` — from STRSXP via `String` intermediary
//! - `HashSet<String>` / `BTreeSet<String>` — string set conversions

use std::borrow::Cow;
use std::collections::{BTreeSet, HashSet};
use std::ffi::OsString;
use std::path::PathBuf;

use crate::ffi::{SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpTypeError, TryFromSexp, charsxp_to_str};

/// Blanket impl: Convert R vector to `Cow<'static, [T]>` where T: RNativeType.
///
/// Returns `Cow::Borrowed` — the slice points directly into R's SEXP data with
/// no copy. The `'static` lifetime is valid for the duration of the `.Call`
/// invocation (R protects the SEXP from GC while Rust code is running).
impl<T> TryFromSexp for Cow<'static, [T]>
where
    T: crate::ffi::RNativeType + Copy + Clone,
{
    type Error = SexpTypeError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        Ok(Cow::Borrowed(slice))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(Cow::Borrowed(slice))
    }
}

/// Convert R character scalar to `Cow<'static, str>`.
///
/// Returns `Cow::Borrowed` — the `&str` points directly into R's CHARSXP data
/// via `R_CHAR` + `LENGTH` (O(1), no strlen). No allocation or copy occurs.
/// The `'static` lifetime is valid for the duration of the `.Call` invocation.
///
/// This delegates to the `&'static str` impl (which uses `charsxp_to_str`),
/// giving the same zero-copy behavior. Use `Cow` when your code may need to
/// mutate the string later — `to_mut()` will copy-on-write at that point.
impl TryFromSexp for Cow<'static, str> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: &'static str = TryFromSexp::try_from_sexp(sexp)?;
        Ok(Cow::Borrowed(s))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: &'static str = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(Cow::Borrowed(s))
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

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::{Rf_translateCharUTF8, STRING_ELT};

        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let mut result = Vec::with_capacity(len);

        for i in 0..len {
            let charsxp = unsafe { STRING_ELT(sexp, i as crate::ffi::R_xlen_t) };
            let s = if charsxp == unsafe { crate::ffi::R_NaString } {
                String::new()
            } else {
                let c_str = unsafe { Rf_translateCharUTF8(charsxp) };
                if c_str.is_null() {
                    String::new()
                } else {
                    unsafe { std::ffi::CStr::from_ptr(c_str) }
                        .to_str()
                        .unwrap_or("")
                        .to_owned()
                }
            };
            result.push(s);
        }

        Ok(result)
    }
}

/// Convert R character vector to `Box<[String]>`.
///
/// **Warning:** `NA_character_` values are converted to empty string `""`.
impl TryFromSexp for Box<[String]> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let vec: Vec<String> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(vec.into_boxed_slice())
    }
}

/// Convert R character vector to `Vec<&str>`.
///
/// **Warning:** `NA_character_` values are converted to empty string `""`.
impl TryFromSexp for Vec<&'static str> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::STRING_ELT;

        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let mut result = Vec::with_capacity(len);

        for i in 0..len {
            let charsxp = unsafe { STRING_ELT(sexp, i as crate::ffi::R_xlen_t) };
            if charsxp == unsafe { crate::ffi::R_NaString } {
                result.push("");
                continue;
            }
            if charsxp == unsafe { crate::ffi::R_BlankString } {
                result.push("");
                continue;
            }
            result.push(unsafe { charsxp_to_str(charsxp) });
        }

        Ok(result)
    }
}

/// Convert R character vector to `Vec<Option<&str>>`.
impl TryFromSexp for Vec<Option<&'static str>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::STRING_ELT;

        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let mut result = Vec::with_capacity(len);

        for i in 0..len {
            let charsxp = unsafe { STRING_ELT(sexp, i as crate::ffi::R_xlen_t) };
            if charsxp == unsafe { crate::ffi::R_NaString } {
                result.push(None);
                continue;
            }
            if charsxp == unsafe { crate::ffi::R_BlankString } {
                result.push(Some(""));
                continue;
            }
            result.push(Some(unsafe { charsxp_to_str(charsxp) }));
        }

        Ok(result)
    }
}

macro_rules! impl_set_string_try_from_sexp {
    ($(#[$meta:meta])* $set_ty:ident) => {
        $(#[$meta])*
        impl TryFromSexp for $set_ty<String> {
            type Error = SexpError;

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
/// Vec, Vec<Option>). Used for PathBuf and OsString which delegate to String conversion.
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
