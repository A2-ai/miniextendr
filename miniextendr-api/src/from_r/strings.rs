//! String conversions — STRSXP requires special handling via `STRING_ELT`.
//!
//! R stores strings as STRSXP (vector of CHARSXP). Each element requires
//! `STRING_ELT` + `R_CHAR` to extract, unlike numeric vectors which expose
//! a contiguous data pointer.
//!
//! Covers: `&str`, `String`, `char`, `Option<&str>`, `Option<String>`,
//! `Vec<String>`, `Vec<&str>`, `Box<[String]>`.

use crate::ffi::{SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{
    SexpError, SexpLengthError, SexpTypeError, TryFromSexp, charsxp_to_str,
    charsxp_to_str_unchecked,
};

/// Convert R character vector (STRSXP) to Rust &str.
///
/// Extracts the first element of the character vector and returns it as a UTF-8 string.
/// The returned string has static lifetime because it points to R's internal string pool.
///
/// # NA Handling
///
/// **Warning:** `NA_character_` is converted to empty string `""`. This is lossy!
/// If you need to distinguish between NA and empty strings, use `Option<String>` instead:
///
/// ```ignore
/// let maybe_str: Option<String> = sexp.try_into()?;
/// ```
///
/// # Safety
/// The returned &str is only valid as long as R doesn't garbage collect the CHARSXP.
/// In practice, this is safe within a single .Call invocation.
impl TryFromSexp for &'static str {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }

        // Get the CHARSXP at index 0
        let charsxp = sexp.string_elt(0);

        // Check for NA_STRING or R_BlankString
        if charsxp == SEXP::na_string() {
            return Ok("");
        }
        if charsxp == SEXP::blank_string() {
            return Ok("");
        }

        // Use LENGTH-based conversion (O(1)) instead of CStr::from_ptr (O(n) strlen)
        Ok(unsafe { charsxp_to_str(charsxp) })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = unsafe { sexp.len_unchecked() };
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }

        // Get the CHARSXP at index 0
        let charsxp = unsafe { sexp.string_elt_unchecked(0) };

        // Check for NA_STRING or R_BlankString
        if charsxp == SEXP::na_string() {
            return Ok("");
        }
        if charsxp == SEXP::blank_string() {
            return Ok("");
        }

        // Use LENGTH-based conversion (O(1)) instead of CStr::from_ptr (O(n) strlen)
        Ok(unsafe { charsxp_to_str_unchecked(charsxp) })
    }
}

impl TryFromSexp for Option<&'static str> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }

        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }

        let charsxp = sexp.string_elt(0);
        if charsxp == SEXP::na_string() {
            return Ok(None);
        }
        if charsxp == SEXP::blank_string() {
            return Ok(Some(""));
        }

        Ok(Some(unsafe { charsxp_to_str(charsxp) }))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }

        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = unsafe { sexp.len_unchecked() };
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }

        let charsxp = unsafe { sexp.string_elt_unchecked(0) };
        if charsxp == SEXP::na_string() {
            return Ok(None);
        }
        if charsxp == SEXP::blank_string() {
            return Ok(Some(""));
        }

        Ok(Some(unsafe { charsxp_to_str_unchecked(charsxp) }))
    }
}

/// Convert R character vector (STRSXP) to Rust char.
///
/// Extracts the first character of the first element of the character vector.
/// Returns an error if the string is empty, NA, or has more than one character.
impl TryFromSexp for char {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: &str = TryFromSexp::try_from_sexp(sexp)?;
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => Ok(c),
            (None, _) => Err(SexpError::InvalidValue(
                "empty string cannot be converted to char".to_string(),
            )),
            (Some(_), Some(_)) => Err(SexpError::InvalidValue(
                "string has more than one character".to_string(),
            )),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: &str = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => Ok(c),
            (None, _) => Err(SexpError::InvalidValue(
                "empty string cannot be converted to char".to_string(),
            )),
            (Some(_), Some(_)) => Err(SexpError::InvalidValue(
                "string has more than one character".to_string(),
            )),
        }
    }
}

/// Convert R character vector (STRSXP) to owned Rust String.
///
/// Extracts the first element and creates an owned copy.
///
/// # NA Handling
///
/// **Warning:** `NA_character_` is converted to empty string `""`. This is lossy!
/// If you need to distinguish between NA and empty strings, use `Option<String>` instead:
///
/// ```ignore
/// let maybe_str: Option<String> = sexp.try_into()?;
/// ```
impl TryFromSexp for String {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::Rf_translateCharUTF8;

        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }

        // Get the CHARSXP at index 0
        let charsxp = sexp.string_elt(0);

        // Check for NA_STRING
        if charsxp == SEXP::na_string() {
            return Ok(String::new());
        }

        // Translate to UTF-8 in an R-managed buffer, then copy to an owned Rust String.
        let c_str = unsafe { Rf_translateCharUTF8(charsxp) };
        if c_str.is_null() {
            return Ok(String::new());
        }

        let rust_str = unsafe { std::ffi::CStr::from_ptr(c_str) };
        rust_str.to_str().map(|s| s.to_owned()).map_err(|_| {
            SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual: SEXPTYPE::STRSXP,
            }
            .into()
        })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::Rf_translateCharUTF8_unchecked;

        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = unsafe { sexp.len_unchecked() };
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }

        // Get the CHARSXP at index 0
        let charsxp = unsafe { sexp.string_elt_unchecked(0) };

        // Check for NA_STRING
        if charsxp == SEXP::na_string() {
            return Ok(String::new());
        }

        // Translate to UTF-8 in an R-managed buffer, then copy to an owned Rust String.
        let c_str = unsafe { Rf_translateCharUTF8_unchecked(charsxp) };
        if c_str.is_null() {
            return Ok(String::new());
        }

        let rust_str = unsafe { std::ffi::CStr::from_ptr(c_str) };
        rust_str.to_str().map(|s| s.to_owned()).map_err(|_| {
            SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual: SEXPTYPE::STRSXP,
            }
            .into()
        })
    }
}

/// NA-aware string conversion: returns `None` for `NA_character_`.
///
/// Use this when you need to distinguish between NA and empty strings:
/// ```ignore
/// let maybe_str: Option<String> = sexp.try_into()?;
/// match maybe_str {
///     Some(s) => println!("Got string: {}", s),
///     None => println!("Got NA"),
/// }
/// ```
impl TryFromSexp for Option<String> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::Rf_translateCharUTF8;

        let actual = sexp.type_of();
        // NULL -> None
        if actual == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }

        let charsxp = sexp.string_elt(0);

        // Return None for NA_STRING
        if charsxp == SEXP::na_string() {
            return Ok(None);
        }

        let c_str = unsafe { Rf_translateCharUTF8(charsxp) };
        if c_str.is_null() {
            return Ok(Some(String::new()));
        }

        let rust_str = unsafe { std::ffi::CStr::from_ptr(c_str) };
        rust_str.to_str().map(|s| Some(s.to_owned())).map_err(|_| {
            SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual: SEXPTYPE::STRSXP,
            }
            .into()
        })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        // For Option<String>, unchecked is same as checked (NA check is semantic, not safety)
        Self::try_from_sexp(sexp)
    }
}
// endregion
