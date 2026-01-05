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

// =============================================================================
// CHARSXP to &str conversion helpers
// =============================================================================

/// Convert CHARSXP to `&str` using LENGTH (O(1)) instead of strlen (O(n)).
///
/// # Encoding Assumption
///
/// This function assumes the CHARSXP contains valid UTF-8 or ASCII bytes.
/// Modern R (4.2+) with UTF-8 locale support typically ensures this, but R can
/// store strings in other encodings (latin1, native, bytes).
///
/// **If you receive data from external sources that may not be UTF-8**, consider:
/// - Using `Rf_translateCharUTF8()` to convert to UTF-8 first
/// - Validating with `std::str::from_utf8()` instead of `from_utf8_unchecked()`
///
/// # Safety
///
/// - `charsxp` must be a valid CHARSXP (not NA_STRING, not null).
/// - The CHARSXP must contain valid UTF-8 bytes (CE_UTF8, CE_ASCII, or compatible).
/// - The returned `&str` is only valid as long as R doesn't GC the CHARSXP.
#[inline]
pub(crate) unsafe fn charsxp_to_str(charsxp: SEXP) -> &'static str {
    unsafe {
        let ptr = crate::ffi::R_CHAR(charsxp);
        let len = crate::ffi::LENGTH(charsxp) as usize;
        let bytes = std::slice::from_raw_parts(ptr.cast::<u8>(), len);
        // R's CE_UTF8 strings are guaranteed valid UTF-8, so skip validation
        std::str::from_utf8_unchecked(bytes)
    }
}

/// Unchecked version of [`charsxp_to_str`].
#[inline]
unsafe fn charsxp_to_str_unchecked(charsxp: SEXP) -> &'static str {
    unsafe {
        let ptr = crate::ffi::R_CHAR_unchecked(charsxp);
        // LENGTH is a simple macro, no thread check needed
        let len = crate::ffi::LENGTH(charsxp) as usize;
        let bytes = std::slice::from_raw_parts(ptr.cast::<u8>(), len);
        std::str::from_utf8_unchecked(bytes)
    }
}

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

#[derive(Debug, Clone)]
pub enum SexpError {
    Type(SexpTypeError),
    Length(SexpLengthError),
    Na(SexpNaError),
    /// Value is syntactically valid but semantically invalid (e.g. parse error).
    InvalidValue(String),
    /// A required field was missing from a named list.
    MissingField(String),
    /// A named list has duplicate non-empty names.
    DuplicateName(String),
    /// Failed to convert to `Either<L, R>` - both branches failed.
    ///
    /// Contains the error messages from attempting both conversions.
    #[cfg(feature = "either")]
    EitherConversion {
        /// Error from attempting to convert to the Left type
        left_error: String,
        /// Error from attempting to convert to the Right type
        right_error: String,
    },
}

impl std::fmt::Display for SexpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SexpError::Type(e) => write!(f, "{}", e),
            SexpError::Length(e) => write!(f, "{}", e),
            SexpError::Na(e) => write!(f, "{}", e),
            SexpError::InvalidValue(msg) => write!(f, "invalid value: {}", msg),
            SexpError::MissingField(name) => write!(f, "missing field: {}", name),
            SexpError::DuplicateName(name) => write!(f, "duplicate name in list: {:?}", name),
            #[cfg(feature = "either")]
            SexpError::EitherConversion {
                left_error,
                right_error,
            } => write!(
                f,
                "failed to convert to Either: Left failed ({}), Right failed ({})",
                left_error, right_error
            ),
        }
    }
}

impl std::error::Error for SexpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SexpError::Type(e) => Some(e),
            SexpError::Length(e) => Some(e),
            SexpError::Na(e) => Some(e),
            SexpError::InvalidValue(_) => None,
            SexpError::MissingField(_) => None,
            SexpError::DuplicateName(_) => None,
            #[cfg(feature = "either")]
            SexpError::EitherConversion { .. } => None,
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

/// Pass-through conversion for raw SEXP values.
///
/// This allows SEXP to be used directly in `#[miniextendr]` function signatures.
/// A blanket impl `impl<T: From<SEXP>> TryFromSexp for T` would conflict with
/// the existing `impl<T: RNativeType> TryFromSexp for T`, so we use an explicit impl.
///
/// # Safety
///
/// SEXP handles are only valid on R's main thread. Use with
/// `#[miniextendr(unsafe(main_thread))]` functions.
impl TryFromSexp for SEXP {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        Ok(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Ok(sexp)
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

impl TryFromSexp for Option<i32> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
        if value == crate::altrep_traits::NA_INTEGER {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let value: i32 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        if value == crate::altrep_traits::NA_INTEGER {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }
}

impl TryFromSexp for Option<f64> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
        if value.is_nan() {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let value: f64 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        if value.is_nan() {
            Ok(None)
        } else {
            Ok(Some(value))
        }
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
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }

        // Get the CHARSXP at index 0
        let charsxp = unsafe { STRING_ELT(sexp, 0) };

        // Check for NA_STRING or R_BlankString
        if charsxp == unsafe { crate::ffi::R_NaString } {
            return Ok("");
        }
        if charsxp == unsafe { crate::ffi::R_BlankString } {
            return Ok("");
        }

        // Use LENGTH-based conversion (O(1)) instead of CStr::from_ptr (O(n) strlen)
        Ok(unsafe { charsxp_to_str(charsxp) })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::STRING_ELT_unchecked;

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

        // Check for NA_STRING or R_BlankString
        if charsxp == unsafe { crate::ffi::R_NaString } {
            return Ok("");
        }
        if charsxp == unsafe { crate::ffi::R_BlankString } {
            return Ok("");
        }

        // Use LENGTH-based conversion (O(1)) instead of CStr::from_ptr (O(n) strlen)
        Ok(unsafe { charsxp_to_str_unchecked(charsxp) })
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

/// Convert R logical vector (LGLSXP) to `Vec<Option<bool>>` with NA support.
impl TryFromSexp for Vec<Option<bool>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::LGLSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::LGLSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let ptr = unsafe { crate::ffi::LOGICAL(sexp) };
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };

        Ok(slice
            .iter()
            .map(|&v| RLogical::from_i32(v).to_option_bool())
            .collect())
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::LGLSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::LGLSXP,
                actual,
            }
            .into());
        }

        let len = unsafe { sexp.len_unchecked() };
        let ptr = unsafe { crate::ffi::LOGICAL(sexp) };
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };

        Ok(slice
            .iter()
            .map(|&v| RLogical::from_i32(v).to_option_bool())
            .collect())
    }
}

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
///
/// See [`named_list_to_map`] for NA/empty name handling (elements with NA/empty
/// names map to key `""` and may silently overwrite each other).
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
///
/// See [`named_list_to_map`] for NA/empty name handling (elements with NA/empty
/// names map to key `""` and may silently overwrite each other).
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
///
/// Returns an error if the list has duplicate non-empty, non-NA names.
///
/// # NA and Empty Name Handling
///
/// **Warning:** Elements with NA or empty names are converted with key `""`:
/// - `NA` names become empty string key `""`
/// - Empty string names `""` stay as `""`
/// - If multiple elements have NA/empty names, later ones **silently overwrite** earlier ones
///
/// This means data loss can occur without error if your list has multiple
/// unnamed or NA-named elements.
///
/// **Example of silent data loss:**
/// ```r
/// # In R:
/// x <- list(a = 1, 2, 3)  # Elements 2 and 3 have empty names
/// # After conversion, only one of them survives under key ""
/// ```
///
/// If you need all elements regardless of names, use `Vec<(String, V)>` instead,
/// or convert the list to a vector first.
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

    // Check for duplicate non-empty names before conversion
    if has_names {
        let mut seen = HashSet::new();
        for i in 0..len {
            let charsxp = unsafe { STRING_ELT(names, i as crate::ffi::R_xlen_t) };
            // Skip NA names
            if charsxp == unsafe { crate::ffi::R_NaString } {
                continue;
            }
            let c_str = unsafe { Rf_translateCharUTF8(charsxp) };
            if c_str.is_null() {
                continue;
            }
            let name = unsafe { std::ffi::CStr::from_ptr(c_str) }
                .to_str()
                .unwrap_or("");
            // Skip empty names
            if name.is_empty() {
                continue;
            }
            if !seen.insert(name) {
                return Err(SexpError::DuplicateName(name.to_string()));
            }
        }
    }

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
    <R as TryFromSexp>::Error: Into<SexpError>,
    <R as TryCoerce<T>>::Error: std::fmt::Debug,
{
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let r_val: R = R::try_from_sexp(sexp).map_err(Into::into)?;
        let value: T = r_val
            .try_coerce()
            .map_err(|e| SexpError::InvalidValue(format!("{e:?}")))?;
        Ok(Coerced::new(value))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let r_val: R = unsafe { R::try_from_sexp_unchecked(sexp).map_err(Into::into)? };
        let value: T = r_val
            .try_coerce()
            .map_err(|e| SexpError::InvalidValue(format!("{e:?}")))?;
        Ok(Coerced::new(value))
    }
}

// =============================================================================
// Direct Vec coercion conversions
// =============================================================================
//
// These provide direct `TryFromSexp for Vec<T>` where T is not an R native type
// but can be coerced from one. This mirrors the `impl_into_r_via_coerce!` pattern
// in into_r.rs for the reverse direction.

/// Helper to coerce a slice element-wise into a Vec.
#[inline]
fn coerce_slice_to_vec<R, T>(slice: &[R]) -> Result<Vec<T>, SexpError>
where
    R: Copy + TryCoerce<T>,
    <R as TryCoerce<T>>::Error: std::fmt::Debug,
{
    slice
        .iter()
        .copied()
        .map(|v| {
            v.try_coerce()
                .map_err(|e| SexpError::InvalidValue(format!("{e:?}")))
        })
        .collect()
}

/// Implement `TryFromSexp for Vec<$target>` by reading R's native `$source` type and coercing.
macro_rules! impl_vec_try_from_sexp_coerce {
    ($source:ty => $target:ty) => {
        impl TryFromSexp for Vec<$target> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$source as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$source as RNativeType>::SEXP_TYPE,
                        actual,
                    }
                    .into());
                }
                let slice: &[$source] = unsafe { sexp.as_slice() };
                coerce_slice_to_vec(slice)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                Self::try_from_sexp(sexp)
            }
        }
    };
}

// Integer coercions: R integer (i32) -> various Rust integer types
impl_vec_try_from_sexp_coerce!(i32 => i8);
impl_vec_try_from_sexp_coerce!(i32 => i16);
impl_vec_try_from_sexp_coerce!(i32 => i64);
impl_vec_try_from_sexp_coerce!(i32 => isize);
impl_vec_try_from_sexp_coerce!(i32 => u16);
impl_vec_try_from_sexp_coerce!(i32 => u32);
impl_vec_try_from_sexp_coerce!(i32 => u64);
impl_vec_try_from_sexp_coerce!(i32 => usize);

// Float coercions: R numeric (f64) -> f32
impl_vec_try_from_sexp_coerce!(f64 => f32);

// Logical coercions: R logical (RLogical) -> bool
impl_vec_try_from_sexp_coerce!(RLogical => bool);

// =============================================================================
// Direct HashSet coercion conversions
// =============================================================================

/// Implement `TryFromSexp for HashSet<$target>` by reading R's native `$source` type and coercing.
macro_rules! impl_hashset_try_from_sexp_coerce {
    ($source:ty => $target:ty) => {
        impl TryFromSexp for HashSet<$target> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$source as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$source as RNativeType>::SEXP_TYPE,
                        actual,
                    }
                    .into());
                }
                let slice: &[$source] = unsafe { sexp.as_slice() };
                slice
                    .iter()
                    .copied()
                    .map(|v| {
                        v.try_coerce()
                            .map_err(|e| SexpError::InvalidValue(format!("{e:?}")))
                    })
                    .collect()
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                Self::try_from_sexp(sexp)
            }
        }
    };
}

// Integer coercions: R integer (i32) -> various Rust integer types
impl_hashset_try_from_sexp_coerce!(i32 => i8);
impl_hashset_try_from_sexp_coerce!(i32 => i16);
impl_hashset_try_from_sexp_coerce!(i32 => i64);
impl_hashset_try_from_sexp_coerce!(i32 => isize);
impl_hashset_try_from_sexp_coerce!(i32 => u16);
impl_hashset_try_from_sexp_coerce!(i32 => u32);
impl_hashset_try_from_sexp_coerce!(i32 => u64);
impl_hashset_try_from_sexp_coerce!(i32 => usize);

// Float coercions: R numeric (f64) -> f32
// Note: f32 doesn't implement Hash, so no HashSet<f32>

// Logical coercions: R logical (RLogical) -> bool
impl_hashset_try_from_sexp_coerce!(RLogical => bool);

// =============================================================================
// Direct BTreeSet coercion conversions
// =============================================================================

/// Implement `TryFromSexp for BTreeSet<$target>` by reading R's native `$source` type and coercing.
macro_rules! impl_btreeset_try_from_sexp_coerce {
    ($source:ty => $target:ty) => {
        impl TryFromSexp for BTreeSet<$target> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$source as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$source as RNativeType>::SEXP_TYPE,
                        actual,
                    }
                    .into());
                }
                let slice: &[$source] = unsafe { sexp.as_slice() };
                slice
                    .iter()
                    .copied()
                    .map(|v| {
                        v.try_coerce()
                            .map_err(|e| SexpError::InvalidValue(format!("{e:?}")))
                    })
                    .collect()
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                Self::try_from_sexp(sexp)
            }
        }
    };
}

// Integer coercions: R integer (i32) -> various Rust integer types
impl_btreeset_try_from_sexp_coerce!(i32 => i8);
impl_btreeset_try_from_sexp_coerce!(i32 => i16);
impl_btreeset_try_from_sexp_coerce!(i32 => i64);
impl_btreeset_try_from_sexp_coerce!(i32 => isize);
impl_btreeset_try_from_sexp_coerce!(i32 => u16);
impl_btreeset_try_from_sexp_coerce!(i32 => u32);
impl_btreeset_try_from_sexp_coerce!(i32 => u64);
impl_btreeset_try_from_sexp_coerce!(i32 => usize);

// Float coercions: R numeric (f64) -> f32
// Note: f32 doesn't implement Ord (only PartialOrd due to NaN), so no BTreeSet<f32>

// Logical coercions: R logical (RLogical) -> bool
impl_btreeset_try_from_sexp_coerce!(RLogical => bool);
