//! Conversions from R SEXP to Rust types.
//!
//! This module provides [`TryFromSexp`] implementations for converting R values to Rust types:
//!
//! | R Type | Rust Type | Access Method |
//! |--------|-----------|---------------|
//! | INTSXP | `i32`, `&[i32]` | `INTEGER()` / `DATAPTR_RO` |
//! | REALSXP | `f64`, `&[f64]` | `REAL()` / `DATAPTR_RO` |
//! | LGLSXP | `RLogical`, `&[RLogical]` | `LOGICAL()` / `DATAPTR_RO` |
//! | RAWSXP | `u8`, `&[u8]` | `RAW()` / `DATAPTR_RO` |
//! | STRSXP | `&str`, `String` | `STRING_ELT()` + `R_CHAR()` / `Rf_translateCharUTF8()` |
//!
//! # Thread Safety
//!
//! The trait provides two methods:
//! - [`TryFromSexp::try_from_sexp`] - checked version with debug thread assertions
//! - [`TryFromSexp::try_from_sexp_unchecked`] - unchecked version for performance-critical paths
//!
//! Use `try_from_sexp_unchecked` when you're certain you're on the main thread:
//! - Inside ALTREP callbacks
//! - Inside `#[miniextendr(unsafe(main_thread))]` functions
//! - Inside `extern "C-unwind"` functions called directly by R

use crate::ffi::{RLogical, RNativeType, Rboolean, SEXP, SEXPTYPE, SexpExt};

#[derive(Debug, Clone, Copy)]
pub struct SexpTypeError {
    pub expected: SEXPTYPE,
    pub actual: SEXPTYPE,
}

impl std::fmt::Display for SexpTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "type mismatch: expected {:?}, got {:?}",
            self.expected, self.actual
        )
    }
}

impl std::error::Error for SexpTypeError {}

#[derive(Debug, Clone, Copy)]
pub struct SexpLengthError {
    pub expected: usize,
    pub actual: usize,
}

impl std::fmt::Display for SexpLengthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "length mismatch: expected {}, got {}",
            self.expected, self.actual
        )
    }
}

impl std::error::Error for SexpLengthError {}

#[derive(Debug, Clone, Copy)]
pub struct SexpNaError {
    pub sexp_type: SEXPTYPE,
}

impl std::fmt::Display for SexpNaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unexpected NA value in {:?}", self.sexp_type)
    }
}

impl std::error::Error for SexpNaError {}

#[derive(Debug, Clone, Copy)]
pub enum SexpError {
    Type(SexpTypeError),
    Length(SexpLengthError),
    Na(SexpNaError),
}

impl std::fmt::Display for SexpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SexpError::Type(e) => write!(f, "{}", e),
            SexpError::Length(e) => write!(f, "{}", e),
            SexpError::Na(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for SexpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SexpError::Type(e) => Some(e),
            SexpError::Length(e) => Some(e),
            SexpError::Na(e) => Some(e),
        }
    }
}

impl From<SexpTypeError> for SexpError {
    fn from(e: SexpTypeError) -> Self {
        SexpError::Type(e)
    }
}

impl From<SexpLengthError> for SexpError {
    fn from(e: SexpLengthError) -> Self {
        SexpError::Length(e)
    }
}

impl From<SexpNaError> for SexpError {
    fn from(e: SexpNaError) -> Self {
        SexpError::Na(e)
    }
}

/// TryFrom-style trait for converting SEXP to Rust types.
pub trait TryFromSexp: Sized {
    /// The error type returned when conversion fails.
    type Error;

    /// Attempt to convert an R SEXP to this Rust type.
    ///
    /// In debug builds, may assert that we're on R's main thread.
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error>;

    /// Convert from SEXP without thread safety checks.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread. In debug builds, this still
    /// calls the checked version by default, but implementations may
    /// skip thread assertions for performance.
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        // Default: just call the checked version
        Self::try_from_sexp(sexp)
    }
}

// Blanket implementation for scalar R native types
impl<T: RNativeType> TryFromSexp for T {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
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
        // SAFETY: sexp is a .Call argument, protected by R's calling convention
        unsafe { sexp.as_slice::<T>() }
            .first()
            .cloned()
            .ok_or_else(|| {
                SexpLengthError {
                    expected: 1,
                    actual: 0,
                }
                .into()
            })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
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
        unsafe { sexp.as_slice_unchecked::<T>() }
            .first()
            .cloned()
            .ok_or_else(|| {
                SexpLengthError {
                    expected: 1,
                    actual: 0,
                }
                .into()
            })
    }
}

// =============================================================================
// Logical conversions
// =============================================================================

impl TryFromSexp for Rboolean {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let raw: RLogical = TryFromSexp::try_from_sexp(sexp)?;
        match raw.to_option_bool() {
            Some(false) => Ok(Rboolean::FALSE),
            Some(true) => Ok(Rboolean::TRUE),
            None => Err(SexpNaError {
                sexp_type: SEXPTYPE::LGLSXP,
            }
            .into()),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let raw: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        match raw.to_option_bool() {
            Some(false) => Ok(Rboolean::FALSE),
            Some(true) => Ok(Rboolean::TRUE),
            None => Err(SexpNaError {
                sexp_type: SEXPTYPE::LGLSXP,
            }
            .into()),
        }
    }
}

impl TryFromSexp for bool {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let raw: RLogical = TryFromSexp::try_from_sexp(sexp)?;
        raw.to_option_bool().ok_or_else(|| {
            SexpNaError {
                sexp_type: SEXPTYPE::LGLSXP,
            }
            .into()
        })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let raw: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        raw.to_option_bool().ok_or_else(|| {
            SexpNaError {
                sexp_type: SEXPTYPE::LGLSXP,
            }
            .into()
        })
    }
}

impl TryFromSexp for Option<bool> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let raw: RLogical = TryFromSexp::try_from_sexp(sexp)?;
        Ok(raw.to_option_bool())
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let raw: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(raw.to_option_bool())
    }
}

// Blanket implementation for slices of R native types
impl<T: RNativeType> TryFromSexp for &'static [T] {
    type Error = SexpTypeError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            });
        }
        // SAFETY: sexp is a .Call argument, protected by R's calling convention
        Ok(unsafe { sexp.as_slice::<T>() })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            });
        }
        Ok(unsafe { sexp.as_slice_unchecked::<T>() })
    }
}

// =============================================================================
// String conversions - STRSXP requires special handling via STRING_ELT
// =============================================================================

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
        use crate::ffi::{R_CHAR, STRING_ELT};

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
        let charsxp = unsafe { STRING_ELT(sexp, 0) };

        // Check for NA_STRING
        if charsxp == unsafe { crate::ffi::R_NaString } {
            // Return empty string for NA (or we could return an error)
            return Ok("");
        }

        // Get the C string pointer - R_CHAR returns UTF-8 for ASCII/UTF-8 strings
        let c_str = unsafe { R_CHAR(charsxp) };
        if c_str.is_null() {
            return Ok("");
        }

        // Convert to Rust str
        let rust_str = unsafe { std::ffi::CStr::from_ptr(c_str) };
        rust_str.to_str().map_err(|_| {
            // If not valid UTF-8, this is an error
            // In practice, R strings should be valid after R_CHAR
            SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual: SEXPTYPE::STRSXP,
            }
            .into()
        })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::{R_CHAR_unchecked, STRING_ELT_unchecked};

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
        let charsxp = unsafe { STRING_ELT_unchecked(sexp, 0) };

        // Check for NA_STRING
        if charsxp == unsafe { crate::ffi::R_NaString } {
            return Ok("");
        }

        // Get the C string pointer
        let c_str = unsafe { R_CHAR_unchecked(charsxp) };
        if c_str.is_null() {
            return Ok("");
        }

        // Convert to Rust str
        let rust_str = unsafe { std::ffi::CStr::from_ptr(c_str) };
        rust_str.to_str().map_err(|_| {
            SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual: SEXPTYPE::STRSXP,
            }
            .into()
        })
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
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }

        // Get the CHARSXP at index 0
        let charsxp = unsafe { STRING_ELT(sexp, 0) };

        // Check for NA_STRING
        if charsxp == unsafe { crate::ffi::R_NaString } {
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
        use crate::ffi::{Rf_translateCharUTF8_unchecked, STRING_ELT_unchecked};

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
        let charsxp = unsafe { STRING_ELT_unchecked(sexp, 0) };

        // Check for NA_STRING
        if charsxp == unsafe { crate::ffi::R_NaString } {
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
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }

        let charsxp = unsafe { STRING_ELT(sexp, 0) };

        // Return None for NA_STRING
        if charsxp == unsafe { crate::ffi::R_NaString } {
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

// =============================================================================
// NA-aware vector conversions
// =============================================================================

/// Macro for NA-aware `R vector → Vec<Option<T>>` conversions.
macro_rules! impl_vec_option_try_from_sexp {
    ($t:ty, $sexptype:ident, $dataptr:ident, $is_na:expr) => {
        impl TryFromSexp for Vec<Option<$t>> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::$sexptype {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::$sexptype,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let ptr = unsafe { crate::ffi::$dataptr(sexp) };
                let slice = unsafe { std::slice::from_raw_parts(ptr, len) };

                Ok(slice
                    .iter()
                    .map(|&v| if $is_na(v) { None } else { Some(v) })
                    .collect())
            }
        }
    };
}

impl_vec_option_try_from_sexp!(f64, REALSXP, REAL, |v: f64| v.is_nan());
impl_vec_option_try_from_sexp!(i32, INTSXP, INTEGER, |v: i32| v == i32::MIN);

/// Convert R character vector (STRSXP) to `Vec<Option<String>>` with NA support.
///
/// `NA_character_` elements are converted to `None`.
impl TryFromSexp for Vec<Option<String>> {
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

            if charsxp == unsafe { crate::ffi::R_NaString } {
                result.push(None);
            } else {
                let c_str = unsafe { Rf_translateCharUTF8(charsxp) };
                if c_str.is_null() {
                    result.push(Some(String::new()));
                } else {
                    let rust_str = unsafe { std::ffi::CStr::from_ptr(c_str) };
                    result.push(Some(rust_str.to_str().map(|s| s.to_owned()).map_err(
                        |_| SexpTypeError {
                            expected: SEXPTYPE::STRSXP,
                            actual: SEXPTYPE::STRSXP,
                        },
                    )?));
                }
            }
        }

        Ok(result)
    }
}

// =============================================================================
// Collection conversions (HashMap, BTreeMap, HashSet, BTreeSet)
// =============================================================================

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

/// Convert R named list (VECSXP) to HashMap<String, V>.
impl<V: TryFromSexp> TryFromSexp for HashMap<String, V>
where
    V::Error: Into<SexpError>,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        named_list_to_map(sexp, HashMap::with_capacity)
    }
}

/// Convert R named list (VECSXP) to BTreeMap<String, V>.
impl<V: TryFromSexp> TryFromSexp for BTreeMap<String, V>
where
    V::Error: Into<SexpError>,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        named_list_to_map(sexp, |_| BTreeMap::new())
    }
}

/// Helper to convert R named list to a map type.
fn named_list_to_map<V, M, F>(sexp: SEXP, create_map: F) -> Result<M, SexpError>
where
    V: TryFromSexp,
    V::Error: Into<SexpError>,
    M: Extend<(String, V)>,
    F: FnOnce(usize) -> M,
{
    use crate::ffi::{Rf_getAttrib, Rf_translateCharUTF8, STRING_ELT, VECTOR_ELT};

    let actual = sexp.type_of();
    if actual != SEXPTYPE::VECSXP {
        return Err(SexpTypeError {
            expected: SEXPTYPE::VECSXP,
            actual,
        }
        .into());
    }

    let len = sexp.len();
    let mut map = create_map(len);

    // Get names attribute
    let names = unsafe { Rf_getAttrib(sexp, crate::ffi::R_NamesSymbol) };
    let has_names = names.type_of() == SEXPTYPE::STRSXP && names.len() == len;

    for i in 0..len {
        let key = if has_names {
            let charsxp = unsafe { STRING_ELT(names, i as crate::ffi::R_xlen_t) };
            if charsxp == unsafe { crate::ffi::R_NaString } {
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
            }
        } else {
            // Use index as key if no names
            i.to_string()
        };

        let elem = unsafe { VECTOR_ELT(sexp, i as crate::ffi::R_xlen_t) };
        let value = V::try_from_sexp(elem).map_err(|e| e.into())?;
        map.extend(std::iter::once((key, value)));
    }

    Ok(map)
}

/// Convert R vector to `HashSet<T>`.
impl<T> TryFromSexp for HashSet<T>
where
    T: RNativeType + Eq + Hash,
{
    type Error = SexpTypeError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        Ok(slice.iter().copied().collect())
    }
}

/// Convert R vector to `BTreeSet<T>`.
impl<T> TryFromSexp for BTreeSet<T>
where
    T: RNativeType + Ord,
{
    type Error = SexpTypeError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        Ok(slice.iter().copied().collect())
    }
}

/// Convert R character vector to `Vec<String>`.
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

/// Convert R character vector to `HashSet<String>`.
impl TryFromSexp for HashSet<String> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let vec: Vec<String> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(vec.into_iter().collect())
    }
}

/// Convert R character vector to `BTreeSet<String>`.
impl TryFromSexp for BTreeSet<String> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let vec: Vec<String> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(vec.into_iter().collect())
    }
}

// =============================================================================
// Coerced wrapper - bridge between TryFromSexp and TryCoerce
// =============================================================================

use crate::coerce::{Coerced, TryCoerce};

/// Error type for coerced SEXP conversions.
#[derive(Debug, Clone)]
pub enum CoercedSexpError {
    /// Error from the underlying SEXP conversion
    Sexp(SexpError),
    /// Error from the coercion step
    Coerce(String),
}

impl std::fmt::Display for CoercedSexpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoercedSexpError::Sexp(e) => write!(f, "SEXP conversion failed: {}", e),
            CoercedSexpError::Coerce(msg) => write!(f, "coercion failed: {}", msg),
        }
    }
}

impl std::error::Error for CoercedSexpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CoercedSexpError::Sexp(e) => Some(e),
            CoercedSexpError::Coerce(_) => None,
        }
    }
}

impl From<SexpError> for CoercedSexpError {
    fn from(e: SexpError) -> Self {
        CoercedSexpError::Sexp(e)
    }
}

impl From<SexpTypeError> for CoercedSexpError {
    fn from(e: SexpTypeError) -> Self {
        CoercedSexpError::Sexp(e.into())
    }
}

/// Convert R value to `Coerced<T, R>` by reading `R` and coercing to `T`.
///
/// This enables reading non-native Rust types from R with coercion:
///
/// ```ignore
/// // Read i64 from R integer (i32)
/// let val: Coerced<i64, i32> = TryFromSexp::try_from_sexp(sexp)?;
/// let i64_val: i64 = val.into_inner();
///
/// // Works with collections too:
/// let vec: Vec<Coerced<i64, i32>> = ...;
/// let set: HashSet<Coerced<NonZeroU32, i32>> = ...;
/// ```
impl<T, R> TryFromSexp for Coerced<T, R>
where
    R: TryFromSexp,
    R: TryCoerce<T>,
    <R as TryFromSexp>::Error: Into<CoercedSexpError>,
    <R as TryCoerce<T>>::Error: std::fmt::Debug,
{
    type Error = CoercedSexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let r_val: R = R::try_from_sexp(sexp).map_err(Into::into)?;
        let value: T = r_val
            .try_coerce()
            .map_err(|e| CoercedSexpError::Coerce(format!("{e:?}")))?;
        Ok(Coerced::new(value))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let r_val: R = unsafe { R::try_from_sexp_unchecked(sexp).map_err(Into::into)? };
        let value: T = r_val
            .try_coerce()
            .map_err(|e| CoercedSexpError::Coerce(format!("{e:?}")))?;
        Ok(Coerced::new(value))
    }
}
