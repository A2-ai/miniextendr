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
//! | CPLXSXP | `Rcomplex` | `COMPLEX()` / `DATAPTR_RO` |
//! | STRSXP | `&str`, `String` | `)` + `R_CHAR()` / `Rf_translateCharUTF8()` |
//!
//! # Submodules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`logical`] | `Rboolean`.string_elt(`bool`, `Option<bool>` |
//! | [`coerced_scalars`] | Multi-source numeric scalars (`i8`..`usize`) + large integers (`i64`, `u64`) |
//! | [`references`] | Borrowed views: `&T`, `&mut T`, `&[T]`, `Vec<&T>` |
//! | [`strings`] | `&str`, `String`, `char` from STRSXP |
//! | [`na_vectors`] | `Vec<Option<T>>`, `Box<[Option<T>]>` with NA awareness |
//! | [`collections`] | `HashMap`, `BTreeMap`, `HashSet`, `BTreeSet` |
//! | [`cow_and_paths`] | `Cow<[T]>`, `PathBuf`, `OsString`, string sets |
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

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::altrep_traits::NA_REAL;
use crate::coerce::TryCoerce;
use crate::ffi::{RLogical, SEXP, SEXPTYPE, SexpExt};

/// Check if an f64 value is R's NA_real_ (a specific NaN bit pattern).
///
/// This is different from `f64::is_nan()` which returns true for ALL NaN values.
/// R's `NA_real_` is a specific NaN with a particular bit pattern, while regular
/// NaN values (e.g., from `0.0/0.0`) should be preserved as valid values.
#[inline]
pub(crate) fn is_na_real(value: f64) -> bool {
    value.to_bits() == NA_REAL.to_bits()
}

// region: CHARSXP to &str conversion helpers

/// Convert CHARSXP to `&str` using LENGTH (O(1)) instead of strlen (O(n)).
///
/// # Encoding Assumption
///
/// This function assumes the CHARSXP contains valid UTF-8 or ASCII bytes.
/// Modern R (4.2+) with UTF-8 locale support typically ensures this, but R can
/// store strings in other encodings (latin1, native, bytes).
///
/// # Safety
///
/// - `charsxp` must be a valid CHARSXP (not NA_STRING, not null).
/// - The returned `&str` is only valid as long as R doesn't GC the CHARSXP.
///
/// # Panics
///
/// Panics if the CHARSXP bytes are not valid UTF-8. The R session's UTF-8 locale
/// is validated once at package init (see entrypoint), so this should never fire.
#[inline]
pub(crate) unsafe fn charsxp_to_str(charsxp: SEXP) -> &'static str {
    unsafe {
        let ptr = crate::ffi::R_CHAR(charsxp);
        let len: usize = crate::ffi::LENGTH(charsxp)
            .try_into()
            .expect("CHARSXP LENGTH must be non-negative");
        let bytes = r_slice(ptr.cast::<u8>(), len);
        std::str::from_utf8(bytes).expect("R CHARSXP is not valid UTF-8")
    }
}

/// Unchecked version of [`charsxp_to_str`] (skips R thread checks, still validates UTF-8).
#[inline]
pub(crate) unsafe fn charsxp_to_str_unchecked(charsxp: SEXP) -> &'static str {
    unsafe {
        let ptr = crate::ffi::R_CHAR_unchecked(charsxp);
        let len: usize = crate::ffi::LENGTH(charsxp)
            .try_into()
            .expect("CHARSXP LENGTH must be non-negative");
        let bytes = r_slice(ptr.cast::<u8>(), len);
        std::str::from_utf8(bytes).expect("R CHARSXP is not valid UTF-8")
    }
}

/// Create a slice from an R data pointer, handling the zero-length case.
///
/// R returns a sentinel pointer (`0x1`) instead of null for empty vectors
/// (e.g., `LOGICAL(integer(0))` → `0x1`). Rust 1.93+ validates pointer
/// alignment in `slice::from_raw_parts` even for `len == 0`, so passing
/// R's sentinel directly causes a precondition-check abort.
///
/// This helper returns an empty slice for `len == 0` without touching the pointer.
///
/// # Safety
///
/// If `len > 0`, `ptr` must satisfy the requirements of [`std::slice::from_raw_parts`].
#[inline(always)]
pub(crate) unsafe fn r_slice<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

/// Mutable version of [`r_slice`] for `from_raw_parts_mut`.
///
/// # Safety
///
/// If `len > 0`, `ptr` must satisfy the requirements of [`std::slice::from_raw_parts_mut`].
#[inline(always)]
pub(crate) unsafe fn r_slice_mut<'a, T>(ptr: *mut T, len: usize) -> &'a mut [T] {
    if len == 0 {
        &mut []
    } else {
        unsafe { std::slice::from_raw_parts_mut(ptr, len) }
    }
}

#[derive(Debug, Clone, Copy)]
/// Error describing an unexpected R `SEXPTYPE`.
pub struct SexpTypeError {
    /// Expected R type.
    pub expected: SEXPTYPE,
    /// Actual R type encountered.
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
/// Error describing an unexpected R object length.
pub struct SexpLengthError {
    /// Required length.
    pub expected: usize,
    /// Actual length encountered.
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
/// Error for NA values in conversions that require non-missing values.
pub struct SexpNaError {
    /// R type where an NA was found.
    pub sexp_type: SEXPTYPE,
}

impl std::fmt::Display for SexpNaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unexpected NA value in {:?}", self.sexp_type)
    }
}

impl std::error::Error for SexpNaError {}

#[derive(Debug, Clone)]
/// Unified conversion error when decoding an R `SEXP`.
pub enum SexpError {
    /// `SEXPTYPE` did not match the expected one.
    Type(SexpTypeError),
    /// Length did not match the expected one.
    Length(SexpLengthError),
    /// Missing value encountered where disallowed.
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
///
/// # Examples
///
/// ```no_run
/// use miniextendr_api::ffi::SEXP;
/// use miniextendr_api::from_r::TryFromSexp;
///
/// fn example(sexp: SEXP) {
///     let value: i32 = TryFromSexp::try_from_sexp(sexp).unwrap();
///     let text: String = TryFromSexp::try_from_sexp(sexp).unwrap();
/// }
/// ```
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

/// Pass-through conversion for raw SEXP values with ALTREP auto-materialization.
///
/// This allows `SEXP` to be used directly in `#[miniextendr]` function signatures.
/// When R passes an ALTREP vector (e.g., `1:10`, `seq_len(N)`),
/// [`ensure_materialized`](crate::altrep_sexp::ensure_materialized) is called
/// automatically to force materialization on the R main thread. After this,
/// the SEXP's data pointer is stable and safe to access from any thread.
///
/// # ALTREP handling
///
/// | Input | Result |
/// |---|---|
/// | Regular SEXP | Passed through unchanged |
/// | ALTREP SEXP | Materialized via `ensure_materialized`, then passed through |
///
/// To receive ALTREP without materializing, use
/// [`AltrepSexp`](crate::altrep_sexp::AltrepSexp) as the parameter type instead.
/// To receive the raw SEXP without any conversion (including no materialization),
/// use `extern "C-unwind"`.
///
/// See `docs/ALTREP_SEXP.md` for the full guide.
///
/// # Safety
///
/// SEXP handles are only valid on R's main thread. Use with
/// `#[miniextendr(unsafe(main_thread))]` functions.
impl TryFromSexp for SEXP {
    type Error = SexpError;

    /// Converts a SEXP, auto-materializing ALTREP vectors.
    ///
    /// If the input is ALTREP, [`ensure_materialized`](crate::altrep_sexp::ensure_materialized)
    /// is called to force materialization on the R main thread. After
    /// materialization the data pointer is stable and the SEXP can be safely
    /// sent to other threads.
    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        Ok(unsafe { crate::altrep_sexp::ensure_materialized(sexp) })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Ok(unsafe { crate::altrep_sexp::ensure_materialized(sexp) })
    }
}

impl TryFromSexp for Option<SEXP> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            Ok(Some(unsafe {
                crate::altrep_sexp::ensure_materialized(sexp)
            }))
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}
// endregion

mod logical;

mod coerced_scalars;
pub(crate) use coerced_scalars::coerce_value;

mod references;

// region: Blanket implementations for slices with arbitrary lifetimes

/// Blanket impl for `&[T]` where T: RNativeType
///
/// This replaces the macro-generated `&'static [T]` impls with a more composable
/// blanket impl that works for any lifetime. This enables containers like TinyVec
/// to use blanket impls without needing helper functions.
impl<T> TryFromSexp for &[T]
where
    T: crate::ffi::RNativeType + Copy,
{
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

/// Blanket impl for `&mut [T]` where T: RNativeType
///
/// # Safety note (aliasing)
///
/// This impl can produce aliased `&mut` slices if the same R vector is passed
/// to multiple mutable slice parameters. The caller is responsible for ensuring
/// no two `&mut` borrows alias the same SEXP.
impl<T> TryFromSexp for &mut [T]
where
    T: crate::ffi::RNativeType + Copy,
{
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
        let len = sexp.len();
        let ptr = unsafe { T::dataptr_mut(sexp) };
        Ok(unsafe { r_slice_mut(ptr, len) })
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
        let len = unsafe { sexp.len_unchecked() };
        let ptr = unsafe { T::dataptr_mut(sexp) };
        Ok(unsafe { r_slice_mut(ptr, len) })
    }
}

/// Blanket impl for `Option<&[T]>` where T: RNativeType
impl<T> TryFromSexp for Option<&[T]>
where
    T: crate::ffi::RNativeType + Copy,
{
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp).map_err(SexpError::from)?;
        Ok(Some(slice))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let slice: &[T] =
            unsafe { TryFromSexp::try_from_sexp_unchecked(sexp).map_err(SexpError::from)? };
        Ok(Some(slice))
    }
}

/// Blanket impl for `Option<&mut [T]>` where T: RNativeType
impl<T> TryFromSexp for Option<&mut [T]>
where
    T: crate::ffi::RNativeType + Copy,
{
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let slice: &mut [T] = TryFromSexp::try_from_sexp(sexp).map_err(SexpError::from)?;
        Ok(Some(slice))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let slice: &mut [T] =
            unsafe { TryFromSexp::try_from_sexp_unchecked(sexp).map_err(SexpError::from)? };
        Ok(Some(slice))
    }
}
// endregion

mod strings;

// region: Result conversions (NULL -> Err(()))

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
// endregion

mod na_vectors;

mod collections;

// region: Fixed-size array conversions

/// Blanket impl: Convert R vector to `[T; N]` where T: RNativeType.
///
/// Returns an error if the R vector length doesn't match N.
/// Useful for SHA hashes ([u8; 32]), fixed-size patterns, etc.
impl<T, const N: usize> TryFromSexp for [T; N]
where
    T: crate::ffi::RNativeType + Copy,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        if slice.len() != N {
            return Err(SexpLengthError {
                expected: N,
                actual: slice.len(),
            }
            .into());
        }

        // T: Copy, length verified above. Use MaybeUninit + copy_from_slice.
        let mut arr = std::mem::MaybeUninit::<[T; N]>::uninit();
        unsafe {
            // SAFETY: MaybeUninit<[T; N]> and [T; N] have the same layout.
            // We write all N elements via copy_from_slice, so assume_init is safe.
            let dst: &mut [T] = std::slice::from_raw_parts_mut(arr.as_mut_ptr().cast::<T>(), N);
            dst.copy_from_slice(&slice[..N]);
            Ok(arr.assume_init())
        }
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}
// endregion

// region: VecDeque conversions

use std::collections::VecDeque;

/// Blanket impl: Convert R vector to `VecDeque<T>` where T: RNativeType.
impl<T> TryFromSexp for VecDeque<T>
where
    T: crate::ffi::RNativeType + Copy,
{
    type Error = SexpTypeError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        Ok(VecDeque::from(slice.to_vec()))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(VecDeque::from(slice.to_vec()))
    }
}
// endregion

// region: BinaryHeap conversions

use std::collections::BinaryHeap;

/// Blanket impl: Convert R vector to `BinaryHeap<T>` where T: RNativeType + Ord.
///
/// Creates a binary heap from the R vector elements.
impl<T> TryFromSexp for BinaryHeap<T>
where
    T: crate::ffi::RNativeType + Copy + Ord,
{
    type Error = SexpTypeError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        Ok(BinaryHeap::from(slice.to_vec()))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(BinaryHeap::from(slice.to_vec()))
    }
}
// endregion

mod cow_and_paths;

// region: Option<Collection> conversions
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

macro_rules! impl_option_map_try_from_sexp {
    ($(#[$meta:meta])* $map_ty:ident) => {
        $(#[$meta])*
        impl<V: TryFromSexp> TryFromSexp for Option<$map_ty<String, V>>
        where
            V::Error: Into<SexpError>,
        {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    Ok(None)
                } else {
                    $map_ty::<String, V>::try_from_sexp(sexp).map(Some)
                }
            }
        }
    };
}

impl_option_map_try_from_sexp!(
    /// Convert R value to `Option<HashMap<String, V>>`: NULL -> None, otherwise Some(map).
    HashMap
);
impl_option_map_try_from_sexp!(
    /// Convert R value to `Option<BTreeMap<String, V>>`: NULL -> None, otherwise Some(map).
    BTreeMap
);

macro_rules! impl_option_set_try_from_sexp {
    ($(#[$meta:meta])* $set_ty:ident) => {
        $(#[$meta])*
        impl<T> TryFromSexp for Option<$set_ty<T>>
        where
            $set_ty<T>: TryFromSexp,
            <$set_ty<T> as TryFromSexp>::Error: Into<SexpError>,
        {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    Ok(None)
                } else {
                    $set_ty::<T>::try_from_sexp(sexp)
                        .map(Some)
                        .map_err(Into::into)
                }
            }
        }
    };
}

impl_option_set_try_from_sexp!(
    /// Convert R value to `Option<HashSet<T>>`: NULL -> None, otherwise Some(set).
    HashSet
);
impl_option_set_try_from_sexp!(
    /// Convert R value to `Option<BTreeSet<T>>`: NULL -> None, otherwise Some(set).
    BTreeSet
);
// endregion

// region: Nested vector conversions (list of vectors)

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
            let elem = sexp.vector_elt(i as crate::ffi::R_xlen_t);
            let inner: Vec<T> = Vec::<T>::try_from_sexp(elem).map_err(Into::into)?;
            result.push(inner);
        }

        Ok(result)
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
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
            let elem = sexp.vector_elt(i as crate::ffi::R_xlen_t);
            let inner: Vec<T> =
                unsafe { Vec::<T>::try_from_sexp_unchecked(elem).map_err(Into::into)? };
            result.push(inner);
        }

        Ok(result)
    }
}
// endregion

// region: Coerced wrapper - bridge between TryFromSexp and TryCoerce

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
// endregion

// region: Direct Vec coercion conversions
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

impl TryFromSexp for Box<[bool]> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let vec: Vec<bool> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(vec.into_boxed_slice())
    }
}
// endregion

// region: Direct HashSet / BTreeSet coercion conversions

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

macro_rules! impl_set_try_from_sexp_numeric {
    ($set_ty:ident, $target:ty) => {
        impl TryFromSexp for $set_ty<$target> {
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

impl_set_try_from_sexp_numeric!(HashSet, i8);
impl_set_try_from_sexp_numeric!(HashSet, i16);
impl_set_try_from_sexp_numeric!(HashSet, i64);
impl_set_try_from_sexp_numeric!(HashSet, isize);
impl_set_try_from_sexp_numeric!(HashSet, u16);
impl_set_try_from_sexp_numeric!(HashSet, u32);
impl_set_try_from_sexp_numeric!(HashSet, u64);
impl_set_try_from_sexp_numeric!(HashSet, usize);

impl_set_try_from_sexp_numeric!(BTreeSet, i8);
impl_set_try_from_sexp_numeric!(BTreeSet, i16);
impl_set_try_from_sexp_numeric!(BTreeSet, i64);
impl_set_try_from_sexp_numeric!(BTreeSet, isize);
impl_set_try_from_sexp_numeric!(BTreeSet, u16);
impl_set_try_from_sexp_numeric!(BTreeSet, u32);
impl_set_try_from_sexp_numeric!(BTreeSet, u64);
impl_set_try_from_sexp_numeric!(BTreeSet, usize);

macro_rules! impl_set_try_from_sexp_bool {
    ($set_ty:ident) => {
        impl TryFromSexp for $set_ty<bool> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let vec: Vec<bool> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(vec.into_iter().collect())
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                Self::try_from_sexp(sexp)
            }
        }
    };
}

impl_set_try_from_sexp_bool!(HashSet);
impl_set_try_from_sexp_bool!(BTreeSet);
// endregion

// region: ExternalPtr conversions

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
// endregion

// region: Helper macros for feature-gated modules

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
                use $crate::ffi::{SEXPTYPE, SexpExt};
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
                    let elem = sexp.vector_elt(i as $crate::ffi::R_xlen_t);
                    result.push(<$t as $crate::from_r::TryFromSexp>::try_from_sexp(elem)?);
                }
                Ok(result)
            }

            unsafe fn try_from_sexp_unchecked(
                sexp: $crate::ffi::SEXP,
            ) -> Result<Self, Self::Error> {
                use $crate::ffi::{SEXPTYPE, SexpExt};
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
                    let elem = unsafe { sexp.vector_elt_unchecked(i as $crate::ffi::R_xlen_t) };
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
                use $crate::ffi::{SEXPTYPE, SexpExt};
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
                    let elem = sexp.vector_elt(i as $crate::ffi::R_xlen_t);
                    if elem == $crate::ffi::SEXP::nil() {
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
                use $crate::ffi::{SEXPTYPE, SexpExt};
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
                    let elem = unsafe { sexp.vector_elt_unchecked(i as $crate::ffi::R_xlen_t) };
                    if elem == $crate::ffi::SEXP::nil() {
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
// endregion
