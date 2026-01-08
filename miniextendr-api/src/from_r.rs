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

use crate::altrep_traits::NA_REAL;
use crate::coerce::TryCoerce;
use crate::ffi::{RLogical, RNativeType, Rboolean, SEXP, SEXPTYPE, SexpExt};

/// Check if an f64 value is R's NA_real_ (a specific NaN bit pattern).
///
/// This is different from `f64::is_nan()` which returns true for ALL NaN values.
/// R's `NA_real_` is a specific NaN with a particular bit pattern, while regular
/// NaN values (e.g., from `0.0/0.0`) should be preserved as valid values.
#[inline]
fn is_na_real(value: f64) -> bool {
    value.to_bits() == NA_REAL.to_bits()
}

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

macro_rules! impl_try_from_sexp_scalar_native {
    ($t:ty, $sexptype:ident) => {
        impl TryFromSexp for $t {
            type Error = SexpError;

            #[inline]
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
                if len != 1 {
                    return Err(SexpLengthError {
                        expected: 1,
                        actual: len,
                    }
                    .into());
                }
                unsafe { sexp.as_slice::<$t>() }
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
                if actual != SEXPTYPE::$sexptype {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::$sexptype,
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
                unsafe { sexp.as_slice_unchecked::<$t>() }
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
    };
}

impl_try_from_sexp_scalar_native!(i32, INTSXP);
impl_try_from_sexp_scalar_native!(f64, REALSXP);
impl_try_from_sexp_scalar_native!(u8, RAWSXP);
impl_try_from_sexp_scalar_native!(RLogical, LGLSXP);
impl_try_from_sexp_scalar_native!(crate::ffi::Rcomplex, CPLXSXP);

/// Pass-through conversion for raw SEXP values.
///
/// This allows SEXP to be used directly in `#[miniextendr]` function signatures.
/// A blanket impl `impl<T: From<SEXP>> TryFromSexp for T` would conflict with
/// many explicit conversions in this module, so we use an explicit impl.
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

impl TryFromSexp for Option<SEXP> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            Ok(Some(sexp))
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
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

impl TryFromSexp for Option<Rboolean> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let raw: RLogical = TryFromSexp::try_from_sexp(sexp)?;
        match raw.to_option_bool() {
            Some(false) => Ok(Some(Rboolean::FALSE)),
            Some(true) => Ok(Some(Rboolean::TRUE)),
            None => Ok(None),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let raw: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        match raw.to_option_bool() {
            Some(false) => Ok(Some(Rboolean::FALSE)),
            Some(true) => Ok(Some(Rboolean::TRUE)),
            None => Ok(None),
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
        // NULL -> None
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let raw: RLogical = TryFromSexp::try_from_sexp(sexp)?;
        Ok(raw.to_option_bool())
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        // NULL -> None
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let raw: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(raw.to_option_bool())
    }
}

impl TryFromSexp for Option<RLogical> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let raw: RLogical = TryFromSexp::try_from_sexp(sexp)?;
        if raw.is_na() { Ok(None) } else { Ok(Some(raw)) }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let raw: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        if raw.is_na() { Ok(None) } else { Ok(Some(raw)) }
    }
}

impl TryFromSexp for Option<i32> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // NULL -> None
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
        if value == crate::altrep_traits::NA_INTEGER {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        // NULL -> None
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
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
        // NULL -> None
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
        if is_na_real(value) {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        // NULL -> None
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let value: f64 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        if is_na_real(value) {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }
}

impl TryFromSexp for Option<u8> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
        Ok(Some(value))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let value: u8 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(Some(value))
    }
}

impl TryFromSexp for Option<crate::ffi::Rcomplex> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::altrep_traits::NA_REAL;

        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let value: crate::ffi::Rcomplex = TryFromSexp::try_from_sexp(sexp)?;
        let na_bits = NA_REAL.to_bits();
        if value.r.to_bits() == na_bits || value.i.to_bits() == na_bits {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::altrep_traits::NA_REAL;

        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let value: crate::ffi::Rcomplex = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        let na_bits = NA_REAL.to_bits();
        if value.r.to_bits() == na_bits || value.i.to_bits() == na_bits {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }
}

// =============================================================================
// Coerced scalar conversions (multi-source numeric)
// =============================================================================

#[inline]
fn coerce_value<R, T>(value: R) -> Result<T, SexpError>
where
    R: TryCoerce<T>,
    <R as TryCoerce<T>>::Error: std::fmt::Debug,
{
    value
        .try_coerce()
        .map_err(|e| SexpError::InvalidValue(format!("{e:?}")))
}

#[inline]
fn try_from_sexp_numeric_scalar<T>(sexp: SEXP) -> Result<T, SexpError>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    u8: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Debug,
    <f64 as TryCoerce<T>>::Error: std::fmt::Debug,
    <u8 as TryCoerce<T>>::Error: std::fmt::Debug,
{
    let actual = sexp.type_of();
    match actual {
        SEXPTYPE::INTSXP => {
            let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
            coerce_value(value)
        }
        SEXPTYPE::REALSXP => {
            let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
            coerce_value(value)
        }
        SEXPTYPE::RAWSXP => {
            let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
            coerce_value(value)
        }
        SEXPTYPE::LGLSXP => {
            let value: RLogical = TryFromSexp::try_from_sexp(sexp)?;
            coerce_value(value.to_i32())
        }
        _ => Err(SexpError::InvalidValue(format!(
            "expected integer, numeric, logical, or raw; got {:?}",
            actual
        ))),
    }
}

#[inline]
unsafe fn try_from_sexp_numeric_scalar_unchecked<T>(sexp: SEXP) -> Result<T, SexpError>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    u8: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Debug,
    <f64 as TryCoerce<T>>::Error: std::fmt::Debug,
    <u8 as TryCoerce<T>>::Error: std::fmt::Debug,
{
    let actual = sexp.type_of();
    match actual {
        SEXPTYPE::INTSXP => {
            let value: i32 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
            coerce_value(value)
        }
        SEXPTYPE::REALSXP => {
            let value: f64 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
            coerce_value(value)
        }
        SEXPTYPE::RAWSXP => {
            let value: u8 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
            coerce_value(value)
        }
        SEXPTYPE::LGLSXP => {
            let value: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
            coerce_value(value.to_i32())
        }
        _ => Err(SexpError::InvalidValue(format!(
            "expected integer, numeric, logical, or raw; got {:?}",
            actual
        ))),
    }
}

#[inline]
fn try_from_sexp_numeric_option<T>(sexp: SEXP) -> Result<Option<T>, SexpError>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    u8: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Debug,
    <f64 as TryCoerce<T>>::Error: std::fmt::Debug,
    <u8 as TryCoerce<T>>::Error: std::fmt::Debug,
{
    if sexp.type_of() == SEXPTYPE::NILSXP {
        return Ok(None);
    }

    let actual = sexp.type_of();
    match actual {
        SEXPTYPE::INTSXP => {
            let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
            if value == crate::altrep_traits::NA_INTEGER {
                Ok(None)
            } else {
                coerce_value(value).map(Some)
            }
        }
        SEXPTYPE::REALSXP => {
            let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
            if is_na_real(value) {
                Ok(None)
            } else {
                coerce_value(value).map(Some)
            }
        }
        SEXPTYPE::RAWSXP => {
            let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
            coerce_value(value).map(Some)
        }
        SEXPTYPE::LGLSXP => {
            let value: RLogical = TryFromSexp::try_from_sexp(sexp)?;
            if value.is_na() {
                Ok(None)
            } else {
                coerce_value(value.to_i32()).map(Some)
            }
        }
        _ => Err(SexpError::InvalidValue(format!(
            "expected integer, numeric, logical, or raw; got {:?}",
            actual
        ))),
    }
}

#[inline]
unsafe fn try_from_sexp_numeric_option_unchecked<T>(sexp: SEXP) -> Result<Option<T>, SexpError>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    u8: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Debug,
    <f64 as TryCoerce<T>>::Error: std::fmt::Debug,
    <u8 as TryCoerce<T>>::Error: std::fmt::Debug,
{
    if sexp.type_of() == SEXPTYPE::NILSXP {
        return Ok(None);
    }

    let actual = sexp.type_of();
    match actual {
        SEXPTYPE::INTSXP => {
            let value: i32 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
            if value == crate::altrep_traits::NA_INTEGER {
                Ok(None)
            } else {
                coerce_value(value).map(Some)
            }
        }
        SEXPTYPE::REALSXP => {
            let value: f64 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
            if is_na_real(value) {
                Ok(None)
            } else {
                coerce_value(value).map(Some)
            }
        }
        SEXPTYPE::RAWSXP => {
            let value: u8 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
            coerce_value(value).map(Some)
        }
        SEXPTYPE::LGLSXP => {
            let value: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
            if value.is_na() {
                Ok(None)
            } else {
                coerce_value(value.to_i32()).map(Some)
            }
        }
        _ => Err(SexpError::InvalidValue(format!(
            "expected integer, numeric, logical, or raw; got {:?}",
            actual
        ))),
    }
}

impl TryFromSexp for i8 {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_scalar(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_scalar_unchecked(sexp) }
    }
}

impl TryFromSexp for i16 {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_scalar(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_scalar_unchecked(sexp) }
    }
}

impl TryFromSexp for u16 {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_scalar(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_scalar_unchecked(sexp) }
    }
}

impl TryFromSexp for u32 {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_scalar(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_scalar_unchecked(sexp) }
    }
}

impl TryFromSexp for f32 {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        match actual {
            SEXPTYPE::INTSXP => {
                let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(value as f32)
            }
            SEXPTYPE::REALSXP => {
                let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(value as f32)
            }
            SEXPTYPE::RAWSXP => {
                let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(value as f32)
            }
            SEXPTYPE::LGLSXP => {
                let value: RLogical = TryFromSexp::try_from_sexp(sexp)?;
                Ok(value.to_i32() as f32)
            }
            _ => Err(SexpError::InvalidValue(format!(
                "expected integer, numeric, logical, or raw; got {:?}",
                actual
            ))),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        match actual {
            SEXPTYPE::INTSXP => {
                let value: i32 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(value as f32)
            }
            SEXPTYPE::REALSXP => {
                let value: f64 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(value as f32)
            }
            SEXPTYPE::RAWSXP => {
                let value: u8 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(value as f32)
            }
            SEXPTYPE::LGLSXP => {
                let value: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(value.to_i32() as f32)
            }
            _ => Err(SexpError::InvalidValue(format!(
                "expected integer, numeric, logical, or raw; got {:?}",
                actual
            ))),
        }
    }
}

impl TryFromSexp for Option<i8> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_option(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_option_unchecked(sexp) }
    }
}

impl TryFromSexp for Option<i16> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_option(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_option_unchecked(sexp) }
    }
}

impl TryFromSexp for Option<u16> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_option(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_option_unchecked(sexp) }
    }
}

impl TryFromSexp for Option<u32> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_option(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_option_unchecked(sexp) }
    }
}

impl TryFromSexp for Option<f32> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let actual = sexp.type_of();
        match actual {
            SEXPTYPE::INTSXP => {
                let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
                if value == crate::altrep_traits::NA_INTEGER {
                    Ok(None)
                } else {
                    Ok(Some(value as f32))
                }
            }
            SEXPTYPE::REALSXP => {
                let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
                if is_na_real(value) {
                    Ok(None)
                } else {
                    Ok(Some(value as f32))
                }
            }
            SEXPTYPE::RAWSXP => {
                let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(Some(value as f32))
            }
            SEXPTYPE::LGLSXP => {
                let value: RLogical = TryFromSexp::try_from_sexp(sexp)?;
                if value.is_na() {
                    Ok(None)
                } else {
                    Ok(Some(value.to_i32() as f32))
                }
            }
            _ => Err(SexpError::InvalidValue(format!(
                "expected integer, numeric, logical, or raw; got {:?}",
                actual
            ))),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let actual = sexp.type_of();
        match actual {
            SEXPTYPE::INTSXP => {
                let value: i32 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                if value == crate::altrep_traits::NA_INTEGER {
                    Ok(None)
                } else {
                    Ok(Some(value as f32))
                }
            }
            SEXPTYPE::REALSXP => {
                let value: f64 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                if is_na_real(value) {
                    Ok(None)
                } else {
                    Ok(Some(value as f32))
                }
            }
            SEXPTYPE::RAWSXP => {
                let value: u8 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(Some(value as f32))
            }
            SEXPTYPE::LGLSXP => {
                let value: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                if value.is_na() {
                    Ok(None)
                } else {
                    Ok(Some(value.to_i32() as f32))
                }
            }
            _ => Err(SexpError::InvalidValue(format!(
                "expected integer, numeric, logical, or raw; got {:?}",
                actual
            ))),
        }
    }
}

// =============================================================================
// Large integer scalar conversions (via f64)
// =============================================================================
//
// R doesn't have native 64-bit integers, so these read from REALSXP (f64)
// and convert with range/precision checking.

impl TryFromSexp for i64 {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_scalar(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_scalar_unchecked(sexp) }
    }
}

impl TryFromSexp for u64 {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_scalar(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_scalar_unchecked(sexp) }
    }
}

impl TryFromSexp for Option<i64> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_option(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_option_unchecked(sexp) }
    }
}

impl TryFromSexp for Option<u64> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        try_from_sexp_numeric_option(sexp)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { try_from_sexp_numeric_option_unchecked(sexp) }
    }
}

impl TryFromSexp for usize {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // Try i32 first (more common), fall back to f64 for large values
        let actual = sexp.type_of();
        match actual {
            SEXPTYPE::INTSXP => {
                use crate::coerce::TryCoerce;
                let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
                value
                    .try_coerce()
                    .map_err(|e| SexpError::InvalidValue(format!("{e}")))
            }
            SEXPTYPE::REALSXP => {
                use crate::coerce::TryCoerce;
                let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
                let u: u64 = value
                    .try_coerce()
                    .map_err(|e| SexpError::InvalidValue(format!("{e}")))?;
                u.try_into()
                    .map_err(|_| SexpError::InvalidValue("value out of usize range".into()))
            }
            SEXPTYPE::RAWSXP => {
                use crate::coerce::Coerce;
                let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(value.coerce())
            }
            SEXPTYPE::LGLSXP => {
                use crate::coerce::TryCoerce;
                let value: RLogical = TryFromSexp::try_from_sexp(sexp)?;
                value
                    .to_i32()
                    .try_coerce()
                    .map_err(|e| SexpError::InvalidValue(format!("{e}")))
            }
            _ => Err(SexpError::InvalidValue(format!(
                "expected integer, numeric, logical, or raw; got {:?}",
                actual
            ))),
        }
    }
}

impl TryFromSexp for Option<usize> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let actual = sexp.type_of();
        match actual {
            SEXPTYPE::INTSXP => {
                use crate::coerce::TryCoerce;
                let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
                if value == crate::altrep_traits::NA_INTEGER {
                    Ok(None)
                } else {
                    value
                        .try_coerce()
                        .map(Some)
                        .map_err(|e| SexpError::InvalidValue(format!("{e}")))
                }
            }
            SEXPTYPE::REALSXP => {
                use crate::coerce::TryCoerce;
                let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
                if is_na_real(value) {
                    return Ok(None);
                }
                let u: u64 = value
                    .try_coerce()
                    .map_err(|e| SexpError::InvalidValue(format!("{e}")))?;
                u.try_into()
                    .map(Some)
                    .map_err(|_| SexpError::InvalidValue("value out of usize range".into()))
            }
            SEXPTYPE::RAWSXP => {
                use crate::coerce::Coerce;
                let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(Some(value.coerce()))
            }
            SEXPTYPE::LGLSXP => {
                use crate::coerce::TryCoerce;
                let value: RLogical = TryFromSexp::try_from_sexp(sexp)?;
                if value.is_na() {
                    Ok(None)
                } else {
                    value
                        .to_i32()
                        .try_coerce()
                        .map(Some)
                        .map_err(|e| SexpError::InvalidValue(format!("{e}")))
                }
            }
            _ => Err(SexpError::InvalidValue(format!(
                "expected integer, numeric, logical, or raw; got {:?}",
                actual
            ))),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

impl TryFromSexp for isize {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // Try i32 first (more common), fall back to f64 for large values
        let actual = sexp.type_of();
        match actual {
            SEXPTYPE::INTSXP => {
                use crate::coerce::Coerce;
                let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(value.coerce())
            }
            SEXPTYPE::REALSXP => {
                use crate::coerce::TryCoerce;
                let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
                let i: i64 = value
                    .try_coerce()
                    .map_err(|e| SexpError::InvalidValue(format!("{e}")))?;
                i.try_into()
                    .map_err(|_| SexpError::InvalidValue("value out of isize range".into()))
            }
            SEXPTYPE::RAWSXP => {
                use crate::coerce::Coerce;
                let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(value.coerce())
            }
            SEXPTYPE::LGLSXP => {
                use crate::coerce::Coerce;
                let value: RLogical = TryFromSexp::try_from_sexp(sexp)?;
                Ok(value.to_i32().coerce())
            }
            _ => Err(SexpError::InvalidValue(format!(
                "expected integer, numeric, logical, or raw; got {:?}",
                actual
            ))),
        }
    }
}

impl TryFromSexp for Option<isize> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let actual = sexp.type_of();
        match actual {
            SEXPTYPE::INTSXP => {
                use crate::coerce::Coerce;
                let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
                if value == crate::altrep_traits::NA_INTEGER {
                    Ok(None)
                } else {
                    Ok(Some(value.coerce()))
                }
            }
            SEXPTYPE::REALSXP => {
                use crate::coerce::TryCoerce;
                let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
                if is_na_real(value) {
                    return Ok(None);
                }
                let i: i64 = value
                    .try_coerce()
                    .map_err(|e| SexpError::InvalidValue(format!("{e}")))?;
                i.try_into()
                    .map(Some)
                    .map_err(|_| SexpError::InvalidValue("value out of isize range".into()))
            }
            SEXPTYPE::RAWSXP => {
                use crate::coerce::Coerce;
                let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
                Ok(Some(value.coerce()))
            }
            SEXPTYPE::LGLSXP => {
                use crate::coerce::Coerce;
                let value: RLogical = TryFromSexp::try_from_sexp(sexp)?;
                if value.is_na() {
                    Ok(None)
                } else {
                    Ok(Some(value.to_i32().coerce()))
                }
            }
            _ => Err(SexpError::InvalidValue(format!(
                "expected integer, numeric, logical, or raw; got {:?}",
                actual
            ))),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

// =============================================================================
// Reference conversions (borrowed views)
// =============================================================================

macro_rules! impl_ref_conversions_for {
    ($t:ty) => {
        impl TryFromSexp for &'static $t {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$t as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$t as RNativeType>::SEXP_TYPE,
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
                unsafe { sexp.as_slice::<$t>() }
                    .first()
                    .ok_or_else(|| SexpLengthError { expected: 1, actual: 0 }.into())
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$t as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$t as RNativeType>::SEXP_TYPE,
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
                unsafe { sexp.as_slice_unchecked::<$t>() }
                    .first()
                    .ok_or_else(|| SexpLengthError { expected: 1, actual: 0 }.into())
            }
        }

        impl TryFromSexp for &'static mut $t {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$t as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$t as RNativeType>::SEXP_TYPE,
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
                let ptr = unsafe { <$t as RNativeType>::dataptr_mut(sexp) };
                Ok(unsafe { &mut *ptr })
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$t as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$t as RNativeType>::SEXP_TYPE,
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
                let ptr = unsafe { <$t as RNativeType>::dataptr_mut(sexp) };
                Ok(unsafe { &mut *ptr })
            }
        }

        impl TryFromSexp for &'static [$t] {
            type Error = SexpTypeError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$t as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$t as RNativeType>::SEXP_TYPE,
                        actual,
                    });
                }
                Ok(unsafe { sexp.as_slice::<$t>() })
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$t as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$t as RNativeType>::SEXP_TYPE,
                        actual,
                    });
                }
                Ok(unsafe { sexp.as_slice_unchecked::<$t>() })
            }
        }

        impl TryFromSexp for &'static mut [$t] {
            type Error = SexpTypeError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$t as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$t as RNativeType>::SEXP_TYPE,
                        actual,
                    });
                }
                let len = sexp.len();
                let ptr = unsafe { <$t as RNativeType>::dataptr_mut(sexp) };
                Ok(unsafe { std::slice::from_raw_parts_mut(ptr, len) })
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$t as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$t as RNativeType>::SEXP_TYPE,
                        actual,
                    });
                }
                let len = unsafe { sexp.len_unchecked() };
                let ptr = unsafe { <$t as RNativeType>::dataptr_mut(sexp) };
                Ok(unsafe { std::slice::from_raw_parts_mut(ptr, len) })
            }
        }

        impl TryFromSexp for Option<&'static $t> {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                let value: &'static $t = TryFromSexp::try_from_sexp(sexp)?;
                Ok(Some(value))
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                let value: &'static $t = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(Some(value))
            }
        }

        impl TryFromSexp for Option<&'static mut $t> {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                let value: &'static mut $t = TryFromSexp::try_from_sexp(sexp)?;
                Ok(Some(value))
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                let value: &'static mut $t =
                    unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(Some(value))
            }
        }

        impl TryFromSexp for Option<&'static [$t]> {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                let slice: &'static [$t] =
                    TryFromSexp::try_from_sexp(sexp).map_err(SexpError::from)?;
                Ok(Some(slice))
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                let slice: &'static [$t] = unsafe {
                    TryFromSexp::try_from_sexp_unchecked(sexp).map_err(SexpError::from)?
                };
                Ok(Some(slice))
            }
        }

        impl TryFromSexp for Option<&'static mut [$t]> {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                let slice: &'static mut [$t] =
                    TryFromSexp::try_from_sexp(sexp).map_err(SexpError::from)?;
                Ok(Some(slice))
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                let slice: &'static mut [$t] = unsafe {
                    TryFromSexp::try_from_sexp_unchecked(sexp).map_err(SexpError::from)?
                };
                Ok(Some(slice))
            }
        }

        impl TryFromSexp for Vec<&'static $t> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut out = Vec::with_capacity(len);

                for i in 0..len {
                    let elem = unsafe { crate::ffi::VECTOR_ELT(sexp, i as crate::ffi::R_xlen_t) };
                    let value: &'static $t = TryFromSexp::try_from_sexp(elem)?;
                    out.push(value);
                }

                Ok(out)
            }
        }

        impl TryFromSexp for Vec<Option<&'static $t>> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut out = Vec::with_capacity(len);

                for i in 0..len {
                    let elem = unsafe { crate::ffi::VECTOR_ELT(sexp, i as crate::ffi::R_xlen_t) };
                    if elem.type_of() == SEXPTYPE::NILSXP {
                        out.push(None);
                    } else {
                        let value: &'static $t = TryFromSexp::try_from_sexp(elem)?;
                        out.push(Some(value));
                    }
                }

                Ok(out)
            }
        }

        impl TryFromSexp for Vec<&'static mut $t> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut out = Vec::with_capacity(len);
                let mut ptrs: Vec<*mut $t> = Vec::new();

                for i in 0..len {
                    let elem = unsafe { crate::ffi::VECTOR_ELT(sexp, i as crate::ffi::R_xlen_t) };
                    let value: &'static mut $t = TryFromSexp::try_from_sexp(elem)?;
                    let ptr = value as *mut $t;
                    if ptrs.iter().any(|&p| p == ptr) {
                        return Err(SexpError::InvalidValue(
                            "list contains duplicate elements; cannot create multiple mutable references"
                                .to_string(),
                        ));
                    }
                    ptrs.push(ptr);
                    out.push(value);
                }

                Ok(out)
            }
        }

        impl TryFromSexp for Vec<Option<&'static mut $t>> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut out = Vec::with_capacity(len);
                let mut ptrs: Vec<*mut $t> = Vec::new();

                for i in 0..len {
                    let elem = unsafe { crate::ffi::VECTOR_ELT(sexp, i as crate::ffi::R_xlen_t) };
                    if elem.type_of() == SEXPTYPE::NILSXP {
                        out.push(None);
                        continue;
                    }
                    let value: &'static mut $t = TryFromSexp::try_from_sexp(elem)?;
                    let ptr = value as *mut $t;
                    if ptrs.iter().any(|&p| p == ptr) {
                        return Err(SexpError::InvalidValue(
                            "list contains duplicate elements; cannot create multiple mutable references"
                                .to_string(),
                        ));
                    }
                    ptrs.push(ptr);
                    out.push(Some(value));
                }

                Ok(out)
            }
        }

        impl TryFromSexp for Vec<&'static [$t]> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut out = Vec::with_capacity(len);

                for i in 0..len {
                    let elem = unsafe { crate::ffi::VECTOR_ELT(sexp, i as crate::ffi::R_xlen_t) };
                    let slice: &'static [$t] =
                        TryFromSexp::try_from_sexp(elem).map_err(SexpError::from)?;
                    out.push(slice);
                }

                Ok(out)
            }
        }

        impl TryFromSexp for Vec<Option<&'static [$t]>> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut out = Vec::with_capacity(len);

                for i in 0..len {
                    let elem = unsafe { crate::ffi::VECTOR_ELT(sexp, i as crate::ffi::R_xlen_t) };
                    if elem.type_of() == SEXPTYPE::NILSXP {
                        out.push(None);
                    } else {
                        let slice: &'static [$t] =
                            TryFromSexp::try_from_sexp(elem).map_err(SexpError::from)?;
                        out.push(Some(slice));
                    }
                }

                Ok(out)
            }
        }

        impl TryFromSexp for Vec<&'static mut [$t]> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut out = Vec::with_capacity(len);
                let mut ptrs: Vec<*mut $t> = Vec::new();

                for i in 0..len {
                    let elem = unsafe { crate::ffi::VECTOR_ELT(sexp, i as crate::ffi::R_xlen_t) };
                    let slice: &'static mut [$t] =
                        TryFromSexp::try_from_sexp(elem).map_err(SexpError::from)?;
                    if !slice.is_empty() {
                        let ptr = slice.as_mut_ptr();
                        if ptrs.iter().any(|&p| p == ptr) {
                            return Err(SexpError::InvalidValue(
                                "list contains duplicate elements; cannot create multiple mutable references"
                                    .to_string(),
                            ));
                        }
                        ptrs.push(ptr);
                    }
                    out.push(slice);
                }

                Ok(out)
            }
        }

        impl TryFromSexp for Vec<Option<&'static mut [$t]>> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut out = Vec::with_capacity(len);
                let mut ptrs: Vec<*mut $t> = Vec::new();

                for i in 0..len {
                    let elem = unsafe { crate::ffi::VECTOR_ELT(sexp, i as crate::ffi::R_xlen_t) };
                    if elem.type_of() == SEXPTYPE::NILSXP {
                        out.push(None);
                        continue;
                    }
                    let slice: &'static mut [$t] =
                        TryFromSexp::try_from_sexp(elem).map_err(SexpError::from)?;
                    if !slice.is_empty() {
                        let ptr = slice.as_mut_ptr();
                        if ptrs.iter().any(|&p| p == ptr) {
                            return Err(SexpError::InvalidValue(
                                "list contains duplicate elements; cannot create multiple mutable references"
                                    .to_string(),
                            ));
                        }
                        ptrs.push(ptr);
                    }
                    out.push(Some(slice));
                }

                Ok(out)
            }
        }
    };
}

impl_ref_conversions_for!(i32);
impl_ref_conversions_for!(f64);
impl_ref_conversions_for!(u8);
impl_ref_conversions_for!(RLogical);
impl_ref_conversions_for!(crate::ffi::Rcomplex);

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

impl TryFromSexp for Option<&'static str> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::STRING_ELT;

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

        let charsxp = unsafe { STRING_ELT(sexp, 0) };
        if charsxp == unsafe { crate::ffi::R_NaString } {
            return Ok(None);
        }
        if charsxp == unsafe { crate::ffi::R_BlankString } {
            return Ok(Some(""));
        }

        Ok(Some(unsafe { charsxp_to_str(charsxp) }))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::STRING_ELT_unchecked;

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

        let charsxp = unsafe { STRING_ELT_unchecked(sexp, 0) };
        if charsxp == unsafe { crate::ffi::R_NaString } {
            return Ok(None);
        }
        if charsxp == unsafe { crate::ffi::R_BlankString } {
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
// Result conversions (NULL -> Err(()))
// =============================================================================

impl<T> TryFromSexp for Result<T, ()>
where
    T: TryFromSexp,
    T::Error: Into<SexpError>,
{
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(Err(()));
        }
        let value = T::try_from_sexp(sexp).map_err(Into::into)?;
        Ok(Ok(value))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(Err(()));
        }
        let value = unsafe { T::try_from_sexp_unchecked(sexp).map_err(Into::into)? };
        Ok(Ok(value))
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

impl_vec_option_try_from_sexp!(f64, REALSXP, REAL, is_na_real);
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

/// Convert R logical vector (LGLSXP) to `Vec<Rboolean>` (errors on NA).
impl TryFromSexp for Vec<Rboolean> {
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

        slice
            .iter()
            .map(|&v| {
                let raw = RLogical::from_i32(v);
                match raw.to_option_bool() {
                    Some(false) => Ok(Rboolean::FALSE),
                    Some(true) => Ok(Rboolean::TRUE),
                    None => Err(SexpNaError {
                        sexp_type: SEXPTYPE::LGLSXP,
                    }
                    .into()),
                }
            })
            .collect()
    }
}

/// Convert R logical vector (LGLSXP) to `Vec<Option<Rboolean>>` with NA support.
impl TryFromSexp for Vec<Option<Rboolean>> {
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
            .map(|&v| match RLogical::from_i32(v).to_option_bool() {
                Some(false) => Some(Rboolean::FALSE),
                Some(true) => Some(Rboolean::TRUE),
                None => None,
            })
            .collect())
    }
}

/// Convert R logical vector (LGLSXP) to `Vec<Logical>` (ALTREP-compatible).
///
/// This converts R's logical vector to a vector of [`Logical`] values,
/// which is the native representation used by ALTREP logical vectors.
/// Unlike `Vec<bool>`, this preserves NA values.
impl TryFromSexp for Vec<crate::altrep_data::Logical> {
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
            .map(|&v| crate::altrep_data::Logical::from_r_int(v))
            .collect())
    }
}

/// Convert R logical vector (LGLSXP) to `Vec<Option<RLogical>>` with NA support.
impl TryFromSexp for Vec<Option<RLogical>> {
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
            .map(|&v| {
                let raw = RLogical::from_i32(v);
                if raw.is_na() { None } else { Some(raw) }
            })
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

/// Convert R raw vector (RAWSXP) to `Vec<Option<u8>>`.
impl TryFromSexp for Vec<Option<u8>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::RAWSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::RAWSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let ptr = unsafe { crate::ffi::RAW(sexp) };
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };

        Ok(slice.iter().map(|&v| Some(v)).collect())
    }
}

#[inline]
fn try_from_sexp_numeric_option_vec<T>(sexp: SEXP) -> Result<Vec<Option<T>>, SexpError>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    u8: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Debug,
    <f64 as TryCoerce<T>>::Error: std::fmt::Debug,
    <u8 as TryCoerce<T>>::Error: std::fmt::Debug,
{
    let actual = sexp.type_of();
    match actual {
        SEXPTYPE::INTSXP => {
            let slice: &[i32] = unsafe { sexp.as_slice() };
            slice
                .iter()
                .map(|&v| {
                    if v == crate::altrep_traits::NA_INTEGER {
                        Ok(None)
                    } else {
                        coerce_value(v).map(Some)
                    }
                })
                .collect()
        }
        SEXPTYPE::REALSXP => {
            let slice: &[f64] = unsafe { sexp.as_slice() };
            slice
                .iter()
                .map(|&v| {
                    if is_na_real(v) {
                        Ok(None)
                    } else {
                        coerce_value(v).map(Some)
                    }
                })
                .collect()
        }
        SEXPTYPE::RAWSXP => {
            let slice: &[u8] = unsafe { sexp.as_slice() };
            slice.iter().map(|&v| coerce_value(v).map(Some)).collect()
        }
        SEXPTYPE::LGLSXP => {
            let slice: &[RLogical] = unsafe { sexp.as_slice() };
            slice
                .iter()
                .map(|&v| {
                    if v.is_na() {
                        Ok(None)
                    } else {
                        coerce_value(v.to_i32()).map(Some)
                    }
                })
                .collect()
        }
        _ => Err(SexpError::InvalidValue(format!(
            "expected integer, numeric, logical, or raw; got {:?}",
            actual
        ))),
    }
}

macro_rules! impl_vec_option_try_from_sexp_numeric {
    ($t:ty) => {
        impl TryFromSexp for Vec<Option<$t>> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                try_from_sexp_numeric_option_vec(sexp)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                try_from_sexp_numeric_option_vec(sexp)
            }
        }
    };
}

impl_vec_option_try_from_sexp_numeric!(i8);
impl_vec_option_try_from_sexp_numeric!(i16);
impl_vec_option_try_from_sexp_numeric!(u16);
impl_vec_option_try_from_sexp_numeric!(u32);
impl_vec_option_try_from_sexp_numeric!(i64);
impl_vec_option_try_from_sexp_numeric!(u64);
impl_vec_option_try_from_sexp_numeric!(isize);
impl_vec_option_try_from_sexp_numeric!(usize);
impl_vec_option_try_from_sexp_numeric!(f32);

// =============================================================================
// Collection conversions (HashMap, BTreeMap, HashSet, BTreeSet)
// =============================================================================

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// Convert R named list (VECSXP) to HashMap<String, V>.
///
/// See `named_list_to_map` for NA/empty name handling (elements with NA/empty
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
/// See `named_list_to_map` for NA/empty name handling (elements with NA/empty
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

macro_rules! impl_set_try_from_sexp_native {
    ($set:ident<$t:ty>) => {
        impl TryFromSexp for $set<$t> {
            type Error = SexpTypeError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let slice: &[$t] = TryFromSexp::try_from_sexp(sexp)?;
                Ok(slice.iter().copied().collect())
            }
        }
    };
}

impl_set_try_from_sexp_native!(HashSet<i32>);
impl_set_try_from_sexp_native!(HashSet<u8>);
impl_set_try_from_sexp_native!(HashSet<RLogical>);
impl_set_try_from_sexp_native!(BTreeSet<i32>);
impl_set_try_from_sexp_native!(BTreeSet<u8>);

macro_rules! impl_vec_try_from_sexp_native {
    ($t:ty) => {
        impl TryFromSexp for Vec<$t> {
            type Error = SexpTypeError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let slice: &[$t] = TryFromSexp::try_from_sexp(sexp)?;
                Ok(slice.to_vec())
            }
        }
    };
}

impl_vec_try_from_sexp_native!(i32);
impl_vec_try_from_sexp_native!(f64);
impl_vec_try_from_sexp_native!(u8);
impl_vec_try_from_sexp_native!(RLogical);
impl_vec_try_from_sexp_native!(crate::ffi::Rcomplex);

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
// Option<Collection> conversions
// =============================================================================
//
// These convert NULL → None, and non-NULL to Some(collection).
// This differs from Option<scalar> which converts NA → None.

/// Convert R value to `Option<Vec<T>>`: NULL → None, otherwise Some(vec).
impl<T> TryFromSexp for Option<Vec<T>>
where
    Vec<T>: TryFromSexp,
    <Vec<T> as TryFromSexp>::Error: Into<SexpError>,
{
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            Vec::<T>::try_from_sexp(sexp).map(Some).map_err(Into::into)
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            unsafe {
                Vec::<T>::try_from_sexp_unchecked(sexp)
                    .map(Some)
                    .map_err(Into::into)
            }
        }
    }
}

/// Convert R value to `Option<HashMap<String, V>>`: NULL → None, otherwise Some(map).
impl<V: TryFromSexp> TryFromSexp for Option<HashMap<String, V>>
where
    V::Error: Into<SexpError>,
{
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            HashMap::<String, V>::try_from_sexp(sexp).map(Some)
        }
    }
}

/// Convert R value to `Option<BTreeMap<String, V>>`: NULL → None, otherwise Some(map).
impl<V: TryFromSexp> TryFromSexp for Option<BTreeMap<String, V>>
where
    V::Error: Into<SexpError>,
{
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            BTreeMap::<String, V>::try_from_sexp(sexp).map(Some)
        }
    }
}

/// Convert R value to `Option<HashSet<T>>`: NULL → None, otherwise Some(set).
impl<T> TryFromSexp for Option<HashSet<T>>
where
    HashSet<T>: TryFromSexp,
    <HashSet<T> as TryFromSexp>::Error: Into<SexpError>,
{
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            HashSet::<T>::try_from_sexp(sexp)
                .map(Some)
                .map_err(Into::into)
        }
    }
}

/// Convert R value to `Option<BTreeSet<T>>`: NULL → None, otherwise Some(set).
impl<T> TryFromSexp for Option<BTreeSet<T>>
where
    BTreeSet<T>: TryFromSexp,
    <BTreeSet<T> as TryFromSexp>::Error: Into<SexpError>,
{
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            BTreeSet::<T>::try_from_sexp(sexp)
                .map(Some)
                .map_err(Into::into)
        }
    }
}

// =============================================================================
// Nested vector conversions (list of vectors)
// =============================================================================

/// Convert R list (VECSXP) to `Vec<Vec<T>>`.
///
/// Each element of the R list must be convertible to `Vec<T>`.
impl<T> TryFromSexp for Vec<Vec<T>>
where
    Vec<T>: TryFromSexp,
    <Vec<T> as TryFromSexp>::Error: Into<SexpError>,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::VECTOR_ELT;

        let actual = sexp.type_of();
        if actual != SEXPTYPE::VECSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::VECSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let mut result = Vec::with_capacity(len);

        for i in 0..len {
            let elem = unsafe { VECTOR_ELT(sexp, i as crate::ffi::R_xlen_t) };
            let inner: Vec<T> = Vec::<T>::try_from_sexp(elem).map_err(Into::into)?;
            result.push(inner);
        }

        Ok(result)
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::VECTOR_ELT;

        let actual = sexp.type_of();
        if actual != SEXPTYPE::VECSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::VECSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let mut result = Vec::with_capacity(len);

        for i in 0..len {
            let elem = unsafe { VECTOR_ELT(sexp, i as crate::ffi::R_xlen_t) };
            let inner: Vec<T> =
                unsafe { Vec::<T>::try_from_sexp_unchecked(elem).map_err(Into::into)? };
            result.push(inner);
        }

        Ok(result)
    }
}

// =============================================================================
// Coerced wrapper - bridge between TryFromSexp and TryCoerce
// =============================================================================

use crate::coerce::Coerced;

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

/// Convert numeric/logical/raw vectors to `Vec<T>` with element-wise coercion.
#[inline]
fn try_from_sexp_numeric_vec<T>(sexp: SEXP) -> Result<Vec<T>, SexpError>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    u8: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Debug,
    <f64 as TryCoerce<T>>::Error: std::fmt::Debug,
    <u8 as TryCoerce<T>>::Error: std::fmt::Debug,
{
    let actual = sexp.type_of();
    match actual {
        SEXPTYPE::INTSXP => {
            let slice: &[i32] = unsafe { sexp.as_slice() };
            coerce_slice_to_vec(slice)
        }
        SEXPTYPE::REALSXP => {
            let slice: &[f64] = unsafe { sexp.as_slice() };
            coerce_slice_to_vec(slice)
        }
        SEXPTYPE::RAWSXP => {
            let slice: &[u8] = unsafe { sexp.as_slice() };
            coerce_slice_to_vec(slice)
        }
        SEXPTYPE::LGLSXP => {
            let slice: &[RLogical] = unsafe { sexp.as_slice() };
            slice.iter().map(|v| coerce_value(v.to_i32())).collect()
        }
        _ => Err(SexpError::InvalidValue(format!(
            "expected integer, numeric, logical, or raw; got {:?}",
            actual
        ))),
    }
}

/// Implement `TryFromSexp for Vec<$target>` by coercing from integer/real/logical/raw.
macro_rules! impl_vec_try_from_sexp_numeric {
    ($target:ty) => {
        impl TryFromSexp for Vec<$target> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                try_from_sexp_numeric_vec(sexp)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                try_from_sexp_numeric_vec(sexp)
            }
        }
    };
}

impl_vec_try_from_sexp_numeric!(i8);
impl_vec_try_from_sexp_numeric!(i16);
impl_vec_try_from_sexp_numeric!(i64);
impl_vec_try_from_sexp_numeric!(isize);
impl_vec_try_from_sexp_numeric!(u16);
impl_vec_try_from_sexp_numeric!(u32);
impl_vec_try_from_sexp_numeric!(u64);
impl_vec_try_from_sexp_numeric!(usize);
impl_vec_try_from_sexp_numeric!(f32);

/// Convert R logical vector (LGLSXP) to `Vec<bool>` (errors on NA).
impl TryFromSexp for Vec<bool> {
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
        let slice: &[RLogical] = unsafe { sexp.as_slice() };
        coerce_slice_to_vec(slice)
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

// =============================================================================
// Direct HashSet coercion conversions
// =============================================================================

/// Convert numeric/logical/raw vectors to a set type with element-wise coercion.
#[inline]
fn try_from_sexp_numeric_set<T, S>(sexp: SEXP) -> Result<S, SexpError>
where
    S: std::iter::FromIterator<T>,
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    u8: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Debug,
    <f64 as TryCoerce<T>>::Error: std::fmt::Debug,
    <u8 as TryCoerce<T>>::Error: std::fmt::Debug,
{
    let vec = try_from_sexp_numeric_vec(sexp)?;
    Ok(vec.into_iter().collect())
}

macro_rules! impl_hashset_try_from_sexp_numeric {
    ($target:ty) => {
        impl TryFromSexp for HashSet<$target> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                try_from_sexp_numeric_set(sexp)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                try_from_sexp_numeric_set(sexp)
            }
        }
    };
}

impl_hashset_try_from_sexp_numeric!(i8);
impl_hashset_try_from_sexp_numeric!(i16);
impl_hashset_try_from_sexp_numeric!(i64);
impl_hashset_try_from_sexp_numeric!(isize);
impl_hashset_try_from_sexp_numeric!(u16);
impl_hashset_try_from_sexp_numeric!(u32);
impl_hashset_try_from_sexp_numeric!(u64);
impl_hashset_try_from_sexp_numeric!(usize);

impl TryFromSexp for HashSet<bool> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let vec: Vec<bool> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(vec.into_iter().collect())
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

// =============================================================================
// Direct BTreeSet coercion conversions
// =============================================================================

macro_rules! impl_btreeset_try_from_sexp_numeric {
    ($target:ty) => {
        impl TryFromSexp for BTreeSet<$target> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                try_from_sexp_numeric_set(sexp)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                try_from_sexp_numeric_set(sexp)
            }
        }
    };
}

impl_btreeset_try_from_sexp_numeric!(i8);
impl_btreeset_try_from_sexp_numeric!(i16);
impl_btreeset_try_from_sexp_numeric!(i64);
impl_btreeset_try_from_sexp_numeric!(isize);
impl_btreeset_try_from_sexp_numeric!(u16);
impl_btreeset_try_from_sexp_numeric!(u32);
impl_btreeset_try_from_sexp_numeric!(u64);
impl_btreeset_try_from_sexp_numeric!(usize);

impl TryFromSexp for BTreeSet<bool> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let vec: Vec<bool> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(vec.into_iter().collect())
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

// =============================================================================
// ExternalPtr conversions
// =============================================================================

use crate::externalptr::{ExternalPtr, TypeMismatchError, TypedExternal};

/// Convert R EXTPTRSXP to `ExternalPtr<T>`.
///
/// This enables using `ExternalPtr<T>` as parameter types in `#[miniextendr]` functions.
///
/// # Example
///
/// ```ignore
/// #[derive(ExternalPtr)]
/// struct MyData { value: i32 }
///
/// #[miniextendr]
/// fn process(data: ExternalPtr<MyData>) -> i32 {
///     data.value
/// }
/// ```
impl<T: TypedExternal + Send> TryFromSexp for ExternalPtr<T> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::EXTPTRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::EXTPTRSXP,
                actual,
            }
            .into());
        }

        // Use ExternalPtr's type-checked constructor
        unsafe { ExternalPtr::wrap_sexp_with_error(sexp) }.map_err(|e| match e {
            TypeMismatchError::NullPointer => {
                SexpError::InvalidValue("external pointer is null".to_string())
            }
            TypeMismatchError::InvalidTypeId => {
                SexpError::InvalidValue("external pointer has no valid type id".to_string())
            }
            TypeMismatchError::Mismatch { expected, found } => SexpError::InvalidValue(format!(
                "type mismatch: expected `{}`, found `{}`",
                expected, found
            )),
        })
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::EXTPTRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::EXTPTRSXP,
                actual,
            }
            .into());
        }

        // Use ExternalPtr's type-checked constructor (unchecked variant)
        unsafe { ExternalPtr::wrap_sexp_unchecked(sexp) }.ok_or_else(|| {
            SexpError::InvalidValue(
                "failed to convert external pointer: type mismatch or null pointer".to_string(),
            )
        })
    }
}

impl<T: TypedExternal + Send> TryFromSexp for Option<ExternalPtr<T>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let ptr: ExternalPtr<T> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(Some(ptr))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let ptr: ExternalPtr<T> = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(Some(ptr))
    }
}

// =============================================================================
// Helper macros for feature-gated modules
// =============================================================================

/// Implement `TryFromSexp for Option<T>` where T already implements TryFromSexp.
///
/// NULL → None, otherwise delegates to T::try_from_sexp and wraps in Some.
#[macro_export]
macro_rules! impl_option_try_from_sexp {
    ($t:ty) => {
        impl $crate::from_r::TryFromSexp for Option<$t> {
            type Error = $crate::from_r::SexpError;

            fn try_from_sexp(sexp: $crate::ffi::SEXP) -> Result<Self, Self::Error> {
                use $crate::ffi::{SEXPTYPE, SexpExt};
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                <$t as $crate::from_r::TryFromSexp>::try_from_sexp(sexp).map(Some)
            }

            unsafe fn try_from_sexp_unchecked(
                sexp: $crate::ffi::SEXP,
            ) -> Result<Self, Self::Error> {
                use $crate::ffi::{SEXPTYPE, SexpExt};
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                unsafe {
                    <$t as $crate::from_r::TryFromSexp>::try_from_sexp_unchecked(sexp).map(Some)
                }
            }
        }
    };
}

/// Implement `TryFromSexp for Vec<T>` from R list (VECSXP).
///
/// Each element is converted via T::try_from_sexp.
#[macro_export]
macro_rules! impl_vec_try_from_sexp_list {
    ($t:ty) => {
        impl $crate::from_r::TryFromSexp for Vec<$t> {
            type Error = $crate::from_r::SexpError;

            fn try_from_sexp(sexp: $crate::ffi::SEXP) -> Result<Self, Self::Error> {
                use $crate::ffi::{SEXPTYPE, SexpExt, VECTOR_ELT};
                use $crate::from_r::SexpTypeError;

                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut result = Vec::with_capacity(len);
                for i in 0..len {
                    let elem = unsafe { VECTOR_ELT(sexp, i as $crate::ffi::R_xlen_t) };
                    result.push(<$t as $crate::from_r::TryFromSexp>::try_from_sexp(elem)?);
                }
                Ok(result)
            }

            unsafe fn try_from_sexp_unchecked(
                sexp: $crate::ffi::SEXP,
            ) -> Result<Self, Self::Error> {
                use $crate::ffi::{SEXPTYPE, SexpExt, VECTOR_ELT_unchecked};
                use $crate::from_r::SexpTypeError;

                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = unsafe { sexp.len_unchecked() };
                let mut result = Vec::with_capacity(len);
                for i in 0..len {
                    let elem = unsafe { VECTOR_ELT_unchecked(sexp, i as $crate::ffi::R_xlen_t) };
                    result.push(unsafe {
                        <$t as $crate::from_r::TryFromSexp>::try_from_sexp_unchecked(elem)?
                    });
                }
                Ok(result)
            }
        }
    };
}

/// Implement `TryFromSexp for Vec<Option<T>>` from R list (VECSXP).
///
/// NULL elements become None, others are converted via T::try_from_sexp.
#[macro_export]
macro_rules! impl_vec_option_try_from_sexp_list {
    ($t:ty) => {
        impl $crate::from_r::TryFromSexp for Vec<Option<$t>> {
            type Error = $crate::from_r::SexpError;

            fn try_from_sexp(sexp: $crate::ffi::SEXP) -> Result<Self, Self::Error> {
                use $crate::ffi::{SEXPTYPE, SexpExt, VECTOR_ELT};
                use $crate::from_r::SexpTypeError;

                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut result = Vec::with_capacity(len);
                for i in 0..len {
                    let elem = unsafe { VECTOR_ELT(sexp, i as $crate::ffi::R_xlen_t) };
                    if elem == unsafe { $crate::ffi::R_NilValue } {
                        result.push(None);
                    } else {
                        result.push(Some(<$t as $crate::from_r::TryFromSexp>::try_from_sexp(
                            elem,
                        )?));
                    }
                }
                Ok(result)
            }

            unsafe fn try_from_sexp_unchecked(
                sexp: $crate::ffi::SEXP,
            ) -> Result<Self, Self::Error> {
                use $crate::ffi::{SEXPTYPE, SexpExt, VECTOR_ELT_unchecked};
                use $crate::from_r::SexpTypeError;

                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = unsafe { sexp.len_unchecked() };
                let mut result = Vec::with_capacity(len);
                for i in 0..len {
                    let elem = unsafe { VECTOR_ELT_unchecked(sexp, i as $crate::ffi::R_xlen_t) };
                    if elem == unsafe { $crate::ffi::R_NilValue } {
                        result.push(None);
                    } else {
                        result.push(Some(unsafe {
                            <$t as $crate::from_r::TryFromSexp>::try_from_sexp_unchecked(elem)?
                        }));
                    }
                }
                Ok(result)
            }
        }
    };
}
