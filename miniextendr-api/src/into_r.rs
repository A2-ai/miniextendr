//! Conversions from Rust types to R SEXP.
//!
//! This module provides the [`IntoR`] trait for converting Rust values to R SEXPs.
//!
//! # Thread Safety
//!
//! The trait provides two methods:
//! - [`IntoR::into_sexp`] - checked version with debug thread assertions
//! - [`IntoR::into_sexp_unchecked`] - unchecked version for performance-critical paths
//!
//! Use `into_sexp_unchecked` when you're certain you're on the main thread:
//! - Inside ALTREP callbacks
//! - Inside `#[miniextendr(unsafe(main_thread))]` functions
//! - Inside `extern "C-unwind"` functions called directly by R

use crate::altrep_traits::{NA_INTEGER, NA_LOGICAL, NA_REAL};

/// Trait for converting Rust types to R SEXP values.
///
/// # Examples
///
/// ```no_run
/// use miniextendr_api::into_r::IntoR;
///
/// let sexp = 42i32.into_sexp();
/// let sexp = "hello".to_string().into_sexp();
/// ```
pub trait IntoR {
    /// Convert this value to an R SEXP.
    ///
    /// In debug builds, asserts that we're on R's main thread.
    fn into_sexp(self) -> crate::ffi::SEXP;

    /// Convert to SEXP without thread safety checks.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread. In debug builds, this still
    /// calls the checked version by default, but implementations may
    /// skip thread assertions for performance.
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP
    where
        Self: Sized,
    {
        // Default: just call the checked version
        self.into_sexp()
    }
}

impl IntoR for crate::ffi::SEXP {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self
    }
}

impl IntoR for () {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::R_NilValue }
    }
}

impl IntoR for std::convert::Infallible {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::R_NilValue }
    }
}

/// Macro for scalar IntoR via Rf_Scalar* functions.
macro_rules! impl_scalar_into_r {
    ($ty:ty, $checked:ident, $unchecked:ident) => {
        impl IntoR for $ty {
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                unsafe { crate::ffi::$checked(self) }
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe { crate::ffi::$unchecked(self) }
            }
        }
    };
}

impl_scalar_into_r!(i32, Rf_ScalarInteger, Rf_ScalarInteger_unchecked);
impl_scalar_into_r!(f64, Rf_ScalarReal, Rf_ScalarReal_unchecked);
impl_scalar_into_r!(u8, Rf_ScalarRaw, Rf_ScalarRaw_unchecked);

/// Macro for infallible widening IntoR via Coerce.
macro_rules! impl_into_r_via_coerce {
    ($from:ty => $to:ty) => {
        impl IntoR for $from {
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                crate::coerce::Coerce::<$to>::coerce(self).into_sexp()
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe { crate::coerce::Coerce::<$to>::coerce(self).into_sexp_unchecked() }
            }
        }
    };
}

// Infallible widening to i32 (R's INTSXP)
impl_into_r_via_coerce!(i8 => i32);
impl_into_r_via_coerce!(i16 => i32);
impl_into_r_via_coerce!(u16 => i32);

// Infallible widening to f64 (R's REALSXP)
impl_into_r_via_coerce!(f32 => f64);
impl_into_r_via_coerce!(u32 => f64); // all u32 exactly representable in f64

// =============================================================================
// Large integer types → REALSXP (f64)
// =============================================================================
//
// R doesn't have native 64-bit integers. These types convert to f64 (REALSXP)
// which may lose precision for values outside the "safe integer" range.
//
// **Precision Loss Warning:**
// - f64 can exactly represent integers in range [-2^53, 2^53] (±9,007,199,254,740,992)
// - Values outside this range may be rounded to the nearest representable f64
// - This is silent - no error or warning is raised
//
// **Alternatives for exact 64-bit integers:**
// - Use the `bit64` R package (stores as REALSXP but interprets as int64)
// - Store as character strings and parse in R
// - Split into high/low 32-bit parts
//
// For most use cases (counters, IDs, timestamps), values fit within 2^53.

/// Convert `i64` to R integer (INTSXP) or numeric (REALSXP).
///
/// Uses smart conversion: values in `(i32::MIN, i32::MAX]` are returned as
/// R integers for exact representation. Values outside that range (including
/// `i32::MIN` which is `NA_integer_` in R) fall back to R doubles.
///
/// ```ignore
/// let small: i64 = 42;
/// small.into_sexp(); // R integer 42L
///
/// let big: i64 = 3_000_000_000;
/// big.into_sexp(); // R double 3e9
///
/// let na_trap: i64 = i32::MIN as i64;
/// na_trap.into_sexp(); // R double (not NA_integer_!)
/// ```
impl IntoR for i64 {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        // i32::MIN is NA_integer_ in R, so exclude it from the integer range
        if self > i32::MIN as i64 && self <= i32::MAX as i64 {
            (self as i32).into_sexp()
        } else {
            (self as f64).into_sexp()
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        if self > i32::MIN as i64 && self <= i32::MAX as i64 {
            unsafe { (self as i32).into_sexp_unchecked() }
        } else {
            unsafe { (self as f64).into_sexp_unchecked() }
        }
    }
}

/// Convert `u64` to R integer (INTSXP) or numeric (REALSXP).
///
/// Values in `[0, i32::MAX]` are returned as R integers. Larger values
/// fall back to R doubles (which may lose precision above 2^53).
impl IntoR for u64 {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        if self <= i32::MAX as u64 {
            (self as i32).into_sexp()
        } else {
            (self as f64).into_sexp()
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        if self <= i32::MAX as u64 {
            unsafe { (self as i32).into_sexp_unchecked() }
        } else {
            unsafe { (self as f64).into_sexp_unchecked() }
        }
    }
}

/// Convert `isize` to R integer (INTSXP) or numeric (REALSXP).
///
/// On 64-bit platforms, uses the same smart conversion as [`i64`](impl IntoR for i64).
/// On 32-bit platforms, `isize` fits in i32 so conversion is always exact.
impl IntoR for isize {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        (self as i64).into_sexp()
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { (self as i64).into_sexp_unchecked() }
    }
}

/// Convert `usize` to R integer (INTSXP) or numeric (REALSXP).
///
/// Values in `[0, i32::MAX]` are returned as R integers. Larger values
/// fall back to R doubles.
impl IntoR for usize {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        (self as u64).into_sexp()
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { (self as u64).into_sexp_unchecked() }
    }
}

/// Macro for logical IntoR via Rf_ScalarLogical with conversion to i32.
macro_rules! impl_logical_into_r {
    ($ty:ty, $to_i32:expr) => {
        impl IntoR for $ty {
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                unsafe { crate::ffi::Rf_ScalarLogical($to_i32(self)) }
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe { crate::ffi::Rf_ScalarLogical_unchecked($to_i32(self)) }
            }
        }
    };
}

impl_logical_into_r!(bool, |v: bool| v as i32);
impl_logical_into_r!(crate::ffi::Rboolean, |v: crate::ffi::Rboolean| v as i32);
impl_logical_into_r!(crate::ffi::RLogical, crate::ffi::RLogical::to_i32);

impl IntoR for Option<i32> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::Rf_ScalarInteger(NA_INTEGER) },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::Rf_ScalarInteger_unchecked(NA_INTEGER) },
        }
    }
}

impl IntoR for Option<f64> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::Rf_ScalarReal(NA_REAL) },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::Rf_ScalarReal_unchecked(NA_REAL) },
        }
    }
}

impl IntoR for Option<crate::ffi::Rboolean> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { crate::ffi::Rf_ScalarLogical(v as i32) },
            None => unsafe { crate::ffi::Rf_ScalarLogical(NA_LOGICAL) },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { crate::ffi::Rf_ScalarLogical_unchecked(v as i32) },
            None => unsafe { crate::ffi::Rf_ScalarLogical_unchecked(NA_LOGICAL) },
        }
    }
}

impl IntoR for Option<crate::ffi::RLogical> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { crate::ffi::Rf_ScalarLogical(v.to_i32()) },
            None => unsafe { crate::ffi::Rf_ScalarLogical(NA_LOGICAL) },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { crate::ffi::Rf_ScalarLogical_unchecked(v.to_i32()) },
            None => unsafe { crate::ffi::Rf_ScalarLogical_unchecked(NA_LOGICAL) },
        }
    }
}

impl IntoR for Option<bool> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::Rf_ScalarLogical(NA_LOGICAL) },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::Rf_ScalarLogical_unchecked(NA_LOGICAL) },
        }
    }
}

/// Macro for NA-aware `Option<T> → R` smart scalar conversion.
/// Checks if value fits i32 → INTSXP with NA_INTEGER for None,
/// otherwise REALSXP with NA_REAL for None.
macro_rules! impl_option_smart_i64_into_r {
    ($t:ty, $fits_i32:expr) => {
        impl IntoR for Option<$t> {
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                match self {
                    Some(x) if $fits_i32(x) => (x as i32).into_sexp(),
                    Some(x) => (x as f64).into_sexp(),
                    None => unsafe { crate::ffi::Rf_ScalarInteger(NA_INTEGER) },
                }
            }
        }
    };
}

impl_option_smart_i64_into_r!(i64, |x: i64| x > i32::MIN as i64 && x <= i32::MAX as i64);
impl_option_smart_i64_into_r!(u64, |x: u64| x <= i32::MAX as u64);
impl_option_smart_i64_into_r!(isize, |x: isize| x > i32::MIN as isize
    && x <= i32::MAX as isize);
impl_option_smart_i64_into_r!(usize, |x: usize| x <= i32::MAX as usize);

/// Macro for `Option<T>` where `T` coerces to a type with existing Option impl.
macro_rules! impl_option_coerce_into_r {
    ($from:ty => $to:ty) => {
        impl IntoR for Option<$from> {
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                self.map(|x| x as $to).into_sexp()
            }
        }
    };
}

impl_option_coerce_into_r!(i8 => i32);
impl_option_coerce_into_r!(i16 => i32);
impl_option_coerce_into_r!(u16 => i32);
impl_option_coerce_into_r!(u32 => i64); // delegates to smart i64 path
impl_option_coerce_into_r!(f32 => f64);

impl<T: crate::externalptr::TypedExternal> IntoR for crate::externalptr::ExternalPtr<T> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_sexp()
    }
}

/// Blanket impl: Types marked with `IntoExternalPtr` get automatic `IntoR`.
///
/// This wraps the value in `ExternalPtr<T>` automatically, so you can return
/// `MyType` directly from `#[miniextendr]` functions instead of `ExternalPtr<MyType>`.
impl<T: crate::externalptr::IntoExternalPtr> IntoR for T {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        crate::externalptr::ExternalPtr::new(self).into_sexp()
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { crate::externalptr::ExternalPtr::new_unchecked(self).into_sexp() }
    }
}

/// Helper to convert a string slice to R CHARSXP.
/// Uses UTF-8 encoding. Empty strings return R_BlankString (static, no allocation).
#[inline]
fn str_to_charsxp(s: &str) -> crate::ffi::SEXP {
    unsafe {
        if s.is_empty() {
            crate::ffi::R_BlankString
        } else {
            crate::ffi::Rf_mkCharLenCE(s.as_ptr().cast(), s.len() as i32, crate::ffi::CE_UTF8)
        }
    }
}

/// Unchecked version of [`str_to_charsxp`].
#[inline]
unsafe fn str_to_charsxp_unchecked(s: &str) -> crate::ffi::SEXP {
    unsafe {
        if s.is_empty() {
            crate::ffi::R_BlankString
        } else {
            crate::ffi::Rf_mkCharLenCE_unchecked(
                s.as_ptr().cast(),
                s.len() as i32,
                crate::ffi::CE_UTF8,
            )
        }
    }
}

impl IntoR for String {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_str().into_sexp()
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { self.as_str().into_sexp_unchecked() }
    }
}

impl IntoR for char {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        // Convert char to a single-character string
        let mut buf = [0u8; 4];
        let s = self.encode_utf8(&mut buf);
        s.into_sexp()
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        let mut buf = [0u8; 4];
        let s = self.encode_utf8(&mut buf);
        unsafe { s.into_sexp_unchecked() }
    }
}

impl IntoR for &str {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let charsxp = str_to_charsxp(self);
            crate::ffi::Rf_ScalarString(charsxp)
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe {
            let charsxp = str_to_charsxp_unchecked(self);
            crate::ffi::Rf_ScalarString_unchecked(charsxp)
        }
    }
}

impl IntoR for Option<&str> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let charsxp = match self {
                Some(s) => str_to_charsxp(s),
                None => crate::ffi::R_NaString,
            };
            crate::ffi::Rf_ScalarString(charsxp)
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe {
            let charsxp = match self {
                Some(s) => str_to_charsxp_unchecked(s),
                None => crate::ffi::R_NaString,
            };
            crate::ffi::Rf_ScalarString_unchecked(charsxp)
        }
    }
}

impl IntoR for Option<String> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_deref().into_sexp()
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { self.as_deref().into_sexp_unchecked() }
    }
}

/// Convert `Option<&T>` to R SEXP by copying the value.
///
/// - `Some(&v)` → copies `v` and converts to R
/// - `None` → returns `NULL` (R_NilValue)
///
/// Note: This returns NULL for None, not NA, since there's no reference to return.
/// Use `Option<T>` directly if you want NA semantics for scalar types.
impl<T> IntoR for Option<&T>
where
    T: Copy + IntoR,
{
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(&v) => v.into_sexp(),
            None => unsafe { crate::ffi::R_NilValue },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(&v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::R_NilValue },
        }
    }
}

// =============================================================================
// ProtectOnce RAII guard
// =============================================================================

/// RAII guard for a single `Rf_protect` / `Rf_unprotect(1)` pair.
///
/// This eliminates the common pattern of manual `Rf_protect` + `Rf_unprotect`
/// calls that are error-prone when early returns or panics are possible.
struct ProtectOnce(crate::ffi::SEXP);

impl ProtectOnce {
    /// Protect a SEXP and return the guard.
    #[inline]
    fn new(sexp: crate::ffi::SEXP) -> Self {
        unsafe { crate::ffi::Rf_protect(sexp) };
        Self(sexp)
    }
}

impl Drop for ProtectOnce {
    #[inline]
    fn drop(&mut self) {
        unsafe { crate::ffi::Rf_unprotect(1) };
    }
}

impl std::ops::Deref for ProtectOnce {
    type Target = crate::ffi::SEXP;
    #[inline]
    fn deref(&self) -> &crate::ffi::SEXP {
        &self.0
    }
}

// =============================================================================
// Vector conversions
// =============================================================================

impl<T> IntoR for Vec<T>
where
    T: crate::ffi::RNativeType,
{
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { vec_to_sexp(&self) }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { vec_to_sexp_unchecked(&self) }
    }
}

impl<T> IntoR for &[T]
where
    T: crate::ffi::RNativeType,
{
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { vec_to_sexp(self) }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { vec_to_sexp_unchecked(self) }
    }
}

impl<T> IntoR for crate::from_r::CopySliceMut<T>
where
    T: crate::ffi::RNativeType,
{
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.into_vec().into_sexp()
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { self.into_vec().into_sexp_unchecked() }
    }
}

/// Convert a slice to an R vector (checked).
#[inline]
unsafe fn vec_to_sexp<T: crate::ffi::RNativeType>(slice: &[T]) -> crate::ffi::SEXP {
    unsafe {
        let n = slice.len();
        let vec = crate::ffi::Rf_allocVector(T::SEXP_TYPE, n as crate::ffi::R_xlen_t);
        let ptr = crate::ffi::DATAPTR_RO(vec).cast_mut().cast::<T>();
        std::ptr::copy_nonoverlapping(slice.as_ptr(), ptr, n);
        vec
    }
}

/// Convert a slice to an R vector (unchecked).
#[inline]
unsafe fn vec_to_sexp_unchecked<T: crate::ffi::RNativeType>(slice: &[T]) -> crate::ffi::SEXP {
    unsafe {
        let n = slice.len();
        let vec = crate::ffi::Rf_allocVector_unchecked(T::SEXP_TYPE, n as crate::ffi::R_xlen_t);
        let ptr = crate::ffi::DATAPTR_RO_unchecked(vec).cast_mut().cast::<T>();
        std::ptr::copy_nonoverlapping(slice.as_ptr(), ptr, n);
        vec
    }
}

// =============================================================================
// Vec coercion for non-native types (i8, i16, u16 → i32; f32 → f64)
// =============================================================================

/// Macro for `Vec<T>` where `T` coerces to a native R type.
macro_rules! impl_vec_coerce_into_r {
    ($from:ty => $to:ty) => {
        impl IntoR for Vec<$from> {
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                let coerced: Vec<$to> = self.into_iter().map(|x| x as $to).collect();
                coerced.into_sexp()
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                let coerced: Vec<$to> = self.into_iter().map(|x| x as $to).collect();
                unsafe { coerced.into_sexp_unchecked() }
            }
        }

        impl IntoR for &[$from] {
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                let coerced: Vec<$to> = self.iter().map(|&x| x as $to).collect();
                coerced.into_sexp()
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                let coerced: Vec<$to> = self.iter().map(|&x| x as $to).collect();
                unsafe { coerced.into_sexp_unchecked() }
            }
        }
    };
}

// Sub-i32 integer types coerce to i32 (R's INTSXP)
impl_vec_coerce_into_r!(i8 => i32);
impl_vec_coerce_into_r!(i16 => i32);
impl_vec_coerce_into_r!(u16 => i32);

// f32 coerces to f64 (R's REALSXP)
impl_vec_coerce_into_r!(f32 => f64);

// i64/u64/isize/usize: smart conversion (INTSXP when all fit, else REALSXP)
macro_rules! impl_vec_smart_i64_into_r {
    ($t:ty, $fits_i32:expr) => {
        impl IntoR for Vec<$t> {
            fn into_sexp(self) -> crate::ffi::SEXP {
                if self.iter().all(|&x| $fits_i32(x)) {
                    let coerced: Vec<i32> = self.into_iter().map(|x| x as i32).collect();
                    coerced.into_sexp()
                } else {
                    let coerced: Vec<f64> = self.into_iter().map(|x| x as f64).collect();
                    coerced.into_sexp()
                }
            }

            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                if self.iter().all(|&x| $fits_i32(x)) {
                    let coerced: Vec<i32> = self.into_iter().map(|x| x as i32).collect();
                    unsafe { coerced.into_sexp_unchecked() }
                } else {
                    let coerced: Vec<f64> = self.into_iter().map(|x| x as f64).collect();
                    unsafe { coerced.into_sexp_unchecked() }
                }
            }
        }
    };
}

// i32::MIN is NA_integer_ in R, so exclude it
impl_vec_smart_i64_into_r!(i64, |x: i64| x > i32::MIN as i64 && x <= i32::MAX as i64);
impl_vec_smart_i64_into_r!(u64, |x: u64| x <= i32::MAX as u64);
impl_vec_smart_i64_into_r!(isize, |x: isize| x > i32::MIN as isize
    && x <= i32::MAX as isize);
impl_vec_smart_i64_into_r!(usize, |x: usize| x <= i32::MAX as usize);

// =============================================================================
// Collection conversions (HashMap, BTreeMap, HashSet, BTreeSet)
// =============================================================================

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

macro_rules! impl_map_into_r {
    ($(#[$meta:meta])* $map_ty:ident) => {
        $(#[$meta])*
        impl<V: IntoR> IntoR for $map_ty<String, V> {
            fn into_sexp(self) -> crate::ffi::SEXP {
                map_to_named_list(self.into_iter())
            }

            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe { map_to_named_list_unchecked(self.into_iter()) }
            }
        }
    };
}

impl_map_into_r!(
    /// Convert HashMap<String, V> to R named list (VECSXP).
    HashMap
);
impl_map_into_r!(
    /// Convert BTreeMap<String, V> to R named list (VECSXP).
    BTreeMap
);

/// Helper to convert an iterator of (String, V) pairs to a named R list.
fn map_to_named_list<V: IntoR>(
    iter: impl ExactSizeIterator<Item = (String, V)>,
) -> crate::ffi::SEXP {
    unsafe {
        let n = iter.len();
        let list =
            crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::VECSXP, n as crate::ffi::R_xlen_t);
        crate::ffi::Rf_protect(list);

        // Allocate names vector
        let names =
            crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::STRSXP, n as crate::ffi::R_xlen_t);
        crate::ffi::Rf_protect(names);

        for (i, (key, value)) in iter.enumerate() {
            // Set list element
            crate::ffi::SET_VECTOR_ELT(list, i as crate::ffi::R_xlen_t, value.into_sexp());

            // Set name
            let charsxp = crate::ffi::Rf_mkCharLenCE(
                key.as_ptr().cast(),
                key.len() as i32,
                crate::ffi::CE_UTF8,
            );
            crate::ffi::SET_STRING_ELT(names, i as crate::ffi::R_xlen_t, charsxp);
        }

        // Attach names attribute
        crate::ffi::Rf_setAttrib(list, crate::ffi::R_NamesSymbol, names);

        crate::ffi::Rf_unprotect(2);
        list
    }
}

/// Unchecked version of [`map_to_named_list`].
unsafe fn map_to_named_list_unchecked<V: IntoR>(
    iter: impl ExactSizeIterator<Item = (String, V)>,
) -> crate::ffi::SEXP {
    unsafe {
        let n = iter.len();
        let list = crate::ffi::Rf_allocVector_unchecked(
            crate::ffi::SEXPTYPE::VECSXP,
            n as crate::ffi::R_xlen_t,
        );
        crate::ffi::Rf_protect(list);

        let names = crate::ffi::Rf_allocVector_unchecked(
            crate::ffi::SEXPTYPE::STRSXP,
            n as crate::ffi::R_xlen_t,
        );
        crate::ffi::Rf_protect(names);

        for (i, (key, value)) in iter.enumerate() {
            crate::ffi::SET_VECTOR_ELT_unchecked(
                list,
                i as crate::ffi::R_xlen_t,
                value.into_sexp_unchecked(),
            );

            let charsxp = str_to_charsxp_unchecked(&key);
            crate::ffi::SET_STRING_ELT_unchecked(names, i as crate::ffi::R_xlen_t, charsxp);
        }

        crate::ffi::Rf_setAttrib_unchecked(list, crate::ffi::R_NamesSymbol, names);

        crate::ffi::Rf_unprotect(2);
        list
    }
}

/// Convert `HashSet<T>` to R vector.
impl<T> IntoR for HashSet<T>
where
    T: crate::ffi::RNativeType + Eq + Hash,
{
    fn into_sexp(self) -> crate::ffi::SEXP {
        let vec: Vec<T> = self.into_iter().collect();
        vec.into_sexp()
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        let vec: Vec<T> = self.into_iter().collect();
        unsafe { vec.into_sexp_unchecked() }
    }
}

/// Convert `BTreeSet<T>` to R vector.
impl<T> IntoR for BTreeSet<T>
where
    T: crate::ffi::RNativeType + Ord,
{
    fn into_sexp(self) -> crate::ffi::SEXP {
        let vec: Vec<T> = self.into_iter().collect();
        vec.into_sexp()
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        let vec: Vec<T> = self.into_iter().collect();
        unsafe { vec.into_sexp_unchecked() }
    }
}

macro_rules! impl_set_string_into_r {
    ($(#[$meta:meta])* $set_ty:ident) => {
        $(#[$meta])*
        impl IntoR for $set_ty<String> {
            fn into_sexp(self) -> crate::ffi::SEXP {
                let vec: Vec<String> = self.into_iter().collect();
                vec.into_sexp()
            }

            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                let vec: Vec<String> = self.into_iter().collect();
                unsafe { vec.into_sexp_unchecked() }
            }
        }
    };
}

impl_set_string_into_r!(
    /// Convert `HashSet<String>` to R character vector.
    HashSet
);
impl_set_string_into_r!(
    /// Convert `BTreeSet<String>` to R character vector.
    BTreeSet
);

// =============================================================================
// Fixed-size array conversions
// =============================================================================

/// Blanket impl for `[T; N]` where T: RNativeType.
///
/// Enables direct conversion of fixed-size arrays to R vectors.
/// Useful for SHA hashes, fixed-size byte patterns, etc.
impl<T: crate::ffi::RNativeType, const N: usize> IntoR for [T; N] {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_slice().into_sexp()
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { self.as_slice().into_sexp_unchecked() }
    }
}

// =============================================================================
// VecDeque conversions
// =============================================================================

use std::collections::VecDeque;

/// Convert `VecDeque<T>` to R vector where T: RNativeType.
impl<T> IntoR for VecDeque<T>
where
    T: crate::ffi::RNativeType,
{
    fn into_sexp(self) -> crate::ffi::SEXP {
        // Convert to Vec for efficient bulk copy
        let vec: Vec<T> = self.into_iter().collect();
        vec.into_sexp()
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        let vec: Vec<T> = self.into_iter().collect();
        unsafe { vec.into_sexp_unchecked() }
    }
}

// =============================================================================
// BinaryHeap conversions
// =============================================================================

use std::collections::BinaryHeap;

/// Convert `BinaryHeap<T>` to R vector where T: RNativeType + Ord.
///
/// The heap is drained into a vector (destroying the heap property).
/// Elements are returned in arbitrary order, not sorted.
impl<T> IntoR for BinaryHeap<T>
where
    T: crate::ffi::RNativeType + Ord,
{
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.into_vec().into_sexp()
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { self.into_vec().into_sexp_unchecked() }
    }
}

// =============================================================================
// Cow conversions
// =============================================================================

use std::borrow::Cow;

/// Convert `Cow<'_, [T]>` to R vector where T: RNativeType.
///
/// Clones borrowed data if needed.
impl<T> IntoR for Cow<'_, [T]>
where
    T: crate::ffi::RNativeType + Clone,
{
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_ref().into_sexp()
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { self.as_ref().into_sexp_unchecked() }
    }
}

/// Convert `Cow<'_, str>` to R character scalar.
impl IntoR for Cow<'_, str> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_ref().into_sexp()
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { self.as_ref().into_sexp_unchecked() }
    }
}

// =============================================================================
// Box conversions (skipped - conflicts with IntoExternalPtr blanket impl)
// =============================================================================
//
// We can't add `impl<T: IntoR> IntoR for Box<T>` because it conflicts with
// the blanket impl `impl<T: IntoExternalPtr> IntoR for T`. If downstream
// crates implement `IntoExternalPtr for Box<SomeType>`, we'd have overlapping
// impls. Users can manually unbox with `*boxed_value` before conversion.

// =============================================================================
// PathBuf / OsString conversions
// =============================================================================

use std::ffi::OsString;
use std::path::PathBuf;

/// Generate IntoR impls for types with `to_string_lossy()` (owned scalar, ref scalar,
/// Option, Vec, Vec<Option>). Used for PathBuf/&Path and OsString/&OsStr.
macro_rules! impl_lossy_string_into_r {
    (
        $(#[$owned_meta:meta])*
        owned: $owned_ty:ty;
        $(#[$ref_meta:meta])*
        ref: $ref_ty:ty;
        $(#[$option_meta:meta])*
        option: $opt_ty:ty;
        $(#[$vec_meta:meta])*
        vec: $vec_ty:ty;
        $(#[$vec_option_meta:meta])*
        vec_option: $vec_opt_ty:ty;
    ) => {
        $(#[$owned_meta])*
        impl IntoR for $owned_ty {
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                self.to_string_lossy().into_owned().into_sexp()
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe { self.to_string_lossy().into_owned().into_sexp_unchecked() }
            }
        }

        $(#[$ref_meta])*
        impl IntoR for $ref_ty {
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                self.to_string_lossy().into_owned().into_sexp()
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe { self.to_string_lossy().into_owned().into_sexp_unchecked() }
            }
        }

        $(#[$option_meta])*
        impl IntoR for Option<$owned_ty> {
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                self.map(|v| v.to_string_lossy().into_owned()).into_sexp()
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe {
                    self.map(|v| v.to_string_lossy().into_owned())
                        .into_sexp_unchecked()
                }
            }
        }

        $(#[$vec_meta])*
        impl IntoR for Vec<$owned_ty> {
            fn into_sexp(self) -> crate::ffi::SEXP {
                let strings: Vec<String> = self
                    .into_iter()
                    .map(|v| v.to_string_lossy().into_owned())
                    .collect();
                strings.into_sexp()
            }

            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                let strings: Vec<String> = self
                    .into_iter()
                    .map(|v| v.to_string_lossy().into_owned())
                    .collect();
                unsafe { strings.into_sexp_unchecked() }
            }
        }

        $(#[$vec_option_meta])*
        impl IntoR for Vec<Option<$owned_ty>> {
            fn into_sexp(self) -> crate::ffi::SEXP {
                let strings: Vec<Option<String>> = self
                    .into_iter()
                    .map(|opt| opt.map(|v| v.to_string_lossy().into_owned()))
                    .collect();
                strings.into_sexp()
            }

            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                let strings: Vec<Option<String>> = self
                    .into_iter()
                    .map(|opt| opt.map(|v| v.to_string_lossy().into_owned()))
                    .collect();
                unsafe { strings.into_sexp_unchecked() }
            }
        }
    };
}

impl_lossy_string_into_r!(
    /// Convert `PathBuf` to R character scalar.
    ///
    /// On Unix, paths that are not valid UTF-8 will produce lossy output
    /// (invalid sequences replaced with U+FFFD).
    owned: PathBuf;
    /// Convert `&Path` to R character scalar.
    ref: &std::path::Path;
    /// Convert `Option<PathBuf>` to R: Some(path) -> character, None -> NA_character_.
    option: PathBuf;
    /// Convert `Vec<PathBuf>` to R character vector.
    vec: PathBuf;
    /// Convert `Vec<Option<PathBuf>>` to R character vector with NA support.
    vec_option: PathBuf;
);

impl_lossy_string_into_r!(
    /// Convert `OsString` to R character scalar.
    ///
    /// On Unix, strings that are not valid UTF-8 will produce lossy output
    /// (invalid sequences replaced with U+FFFD).
    owned: OsString;
    /// Convert `&OsStr` to R character scalar.
    ref: &std::ffi::OsStr;
    /// Convert `Option<OsString>` to R: Some(s) -> character, None -> NA_character_.
    option: OsString;
    /// Convert `Vec<OsString>` to R character vector.
    vec: OsString;
    /// Convert `Vec<Option<OsString>>` to R character vector with NA support.
    vec_option: OsString;
);

// =============================================================================
// Set coercion for non-native types (i8, i16, u16 → i32)
// =============================================================================

/// Macro for `HashSet<T>`/`BTreeSet<T>` where `T` coerces to i32 (R's native integer type).
macro_rules! impl_set_coerce_into_r {
    ($from:ty) => {
        impl IntoR for HashSet<$from> {
            fn into_sexp(self) -> crate::ffi::SEXP {
                let vec: Vec<i32> = self.into_iter().map(|x| x as i32).collect();
                vec.into_sexp()
            }
        }

        impl IntoR for BTreeSet<$from> {
            fn into_sexp(self) -> crate::ffi::SEXP {
                let vec: Vec<i32> = self.into_iter().map(|x| x as i32).collect();
                vec.into_sexp()
            }
        }
    };
}

// Sub-i32 integer types in sets coerce to i32 (R's INTSXP)
impl_set_coerce_into_r!(i8);
impl_set_coerce_into_r!(i16);
impl_set_coerce_into_r!(u16);

// =============================================================================
// Option<Collection> conversions
// =============================================================================
//
// These return NULL (R_NilValue) for None, and the converted collection for Some.
// This differs from Option<scalar> which returns NA for None.

/// Convert `Option<Vec<T>>` to R: Some(vec) → vector, None → NULL.
impl<T: crate::ffi::RNativeType> IntoR for Option<Vec<T>> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::R_NilValue },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::R_NilValue },
        }
    }
}

/// Convert `Option<Vec<String>>` to R: Some(vec) → character vector, None → NULL.
impl IntoR for Option<Vec<String>> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::R_NilValue },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::R_NilValue },
        }
    }
}

/// Convert `Option<HashMap<String, V>>` to R: Some(map) -> named list, None -> NULL.
impl<V: IntoR> IntoR for Option<HashMap<String, V>> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::R_NilValue },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::R_NilValue },
        }
    }
}

/// Convert `Option<BTreeMap<String, V>>` to R: Some(map) -> named list, None -> NULL.
impl<V: IntoR> IntoR for Option<BTreeMap<String, V>> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::R_NilValue },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::R_NilValue },
        }
    }
}

/// Convert `Option<HashSet<T>>` to R: Some(set) -> vector, None -> NULL.
impl<T: crate::ffi::RNativeType + Eq + Hash> IntoR for Option<HashSet<T>> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::R_NilValue },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::R_NilValue },
        }
    }
}

/// Convert `Option<BTreeSet<T>>` to R: Some(set) -> vector, None -> NULL.
impl<T: crate::ffi::RNativeType + Ord> IntoR for Option<BTreeSet<T>> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::R_NilValue },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::R_NilValue },
        }
    }
}

macro_rules! impl_option_collection_into_r {
    ($(#[$meta:meta])* $ty:ty) => {
        $(#[$meta])*
        impl IntoR for Option<$ty> {
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                match self {
                    Some(v) => v.into_sexp(),
                    None => unsafe { crate::ffi::R_NilValue },
                }
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                match self {
                    Some(v) => unsafe { v.into_sexp_unchecked() },
                    None => unsafe { crate::ffi::R_NilValue },
                }
            }
        }
    };
}

impl_option_collection_into_r!(
    /// Convert `Option<HashSet<String>>` to R: Some(set) -> character vector, None -> NULL.
    HashSet<String>
);
impl_option_collection_into_r!(
    /// Convert `Option<BTreeSet<String>>` to R: Some(set) -> character vector, None -> NULL.
    BTreeSet<String>
);

/// Helper: allocate STRSXP and fill from a string iterator (checked).
fn str_iter_to_strsxp<'a>(iter: impl ExactSizeIterator<Item = &'a str>) -> crate::ffi::SEXP {
    unsafe {
        let n = iter.len();
        let sexp = ProtectOnce::new(crate::ffi::Rf_allocVector(
            crate::ffi::SEXPTYPE::STRSXP,
            n as crate::ffi::R_xlen_t,
        ));
        for (i, s) in iter.enumerate() {
            let charsxp =
                crate::ffi::Rf_mkCharLenCE(s.as_ptr().cast(), s.len() as i32, crate::ffi::CE_UTF8);
            crate::ffi::SET_STRING_ELT(*sexp, i as crate::ffi::R_xlen_t, charsxp);
        }
        *sexp
    }
}

/// Helper: allocate STRSXP and fill from a string iterator (unchecked).
unsafe fn str_iter_to_strsxp_unchecked<'a>(
    iter: impl ExactSizeIterator<Item = &'a str>,
) -> crate::ffi::SEXP {
    unsafe {
        let n = iter.len();
        let sexp = ProtectOnce::new(crate::ffi::Rf_allocVector_unchecked(
            crate::ffi::SEXPTYPE::STRSXP,
            n as crate::ffi::R_xlen_t,
        ));
        for (i, s) in iter.enumerate() {
            let charsxp = str_to_charsxp_unchecked(s);
            crate::ffi::SET_STRING_ELT_unchecked(*sexp, i as crate::ffi::R_xlen_t, charsxp);
        }
        *sexp
    }
}

/// Convert `Vec<String>` to R character vector (STRSXP).
impl IntoR for Vec<String> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        str_iter_to_strsxp(self.iter().map(|s| s.as_str()))
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { str_iter_to_strsxp_unchecked(self.iter().map(|s| s.as_str())) }
    }
}

/// Convert `&[String]` to R character vector (STRSXP).
impl IntoR for &[String] {
    fn into_sexp(self) -> crate::ffi::SEXP {
        str_iter_to_strsxp(self.iter().map(|s| s.as_str()))
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { str_iter_to_strsxp_unchecked(self.iter().map(|s| s.as_str())) }
    }
}

/// Convert &[&str] to R character vector (STRSXP).
impl IntoR for &[&str] {
    fn into_sexp(self) -> crate::ffi::SEXP {
        str_iter_to_strsxp(self.iter().copied())
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { str_iter_to_strsxp_unchecked(self.iter().copied()) }
    }
}

/// Convert `Vec<&str>` to R character vector (STRSXP).
impl IntoR for Vec<&str> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_slice().into_sexp()
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { self.as_slice().into_sexp_unchecked() }
    }
}

// =============================================================================
// Nested vector conversions (list of vectors)
// =============================================================================

/// Convert `Vec<Vec<T>>` to R list of vectors (VECSXP of typed vectors).
impl<T> IntoR for Vec<Vec<T>>
where
    T: crate::ffi::RNativeType,
{
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let list =
                crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::VECSXP, n as crate::ffi::R_xlen_t);
            crate::ffi::Rf_protect(list);

            for (i, inner) in self.into_iter().enumerate() {
                let inner_sexp = inner.into_sexp();
                crate::ffi::SET_VECTOR_ELT(list, i as crate::ffi::R_xlen_t, inner_sexp);
            }

            crate::ffi::Rf_unprotect(1);
            list
        }
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let list = crate::ffi::Rf_allocVector_unchecked(
                crate::ffi::SEXPTYPE::VECSXP,
                n as crate::ffi::R_xlen_t,
            );
            crate::ffi::Rf_protect(list);

            for (i, inner) in self.into_iter().enumerate() {
                let inner_sexp = inner.into_sexp_unchecked();
                crate::ffi::SET_VECTOR_ELT_unchecked(list, i as crate::ffi::R_xlen_t, inner_sexp);
            }

            crate::ffi::Rf_unprotect(1);
            list
        }
    }
}

/// Convert `Vec<Vec<String>>` to R list of character vectors.
impl IntoR for Vec<Vec<String>> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let list =
                crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::VECSXP, n as crate::ffi::R_xlen_t);
            crate::ffi::Rf_protect(list);

            for (i, inner) in self.into_iter().enumerate() {
                let inner_sexp = inner.into_sexp();
                crate::ffi::SET_VECTOR_ELT(list, i as crate::ffi::R_xlen_t, inner_sexp);
            }

            crate::ffi::Rf_unprotect(1);
            list
        }
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let list = crate::ffi::Rf_allocVector_unchecked(
                crate::ffi::SEXPTYPE::VECSXP,
                n as crate::ffi::R_xlen_t,
            );
            crate::ffi::Rf_protect(list);

            for (i, inner) in self.into_iter().enumerate() {
                let inner_sexp = inner.into_sexp_unchecked();
                crate::ffi::SET_VECTOR_ELT_unchecked(list, i as crate::ffi::R_xlen_t, inner_sexp);
            }

            crate::ffi::Rf_unprotect(1);
            list
        }
    }
}

// =============================================================================
// NA-aware vector conversions
// =============================================================================

/// Macro for NA-aware `Vec<Option<T>> → R` vector conversions.
macro_rules! impl_vec_option_into_r {
    ($t:ty, $sexptype:ident, $dataptr:ident, $dataptr_unchecked:ident, $na_value:expr) => {
        impl IntoR for Vec<Option<$t>> {
            fn into_sexp(self) -> crate::ffi::SEXP {
                unsafe {
                    let n = self.len();
                    let vec = crate::ffi::Rf_allocVector(
                        crate::ffi::SEXPTYPE::$sexptype,
                        n as crate::ffi::R_xlen_t,
                    );
                    crate::ffi::Rf_protect(vec);

                    if n > 0 {
                        let ptr = crate::ffi::$dataptr(vec);
                        let out = std::slice::from_raw_parts_mut(ptr, n);
                        for (slot, val) in out.iter_mut().zip(self.into_iter()) {
                            *slot = val.unwrap_or($na_value);
                        }
                    }

                    crate::ffi::Rf_unprotect(1);
                    vec
                }
            }

            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe {
                    let n = self.len();
                    let vec = crate::ffi::Rf_allocVector_unchecked(
                        crate::ffi::SEXPTYPE::$sexptype,
                        n as crate::ffi::R_xlen_t,
                    );
                    crate::ffi::Rf_protect(vec);

                    if n > 0 {
                        let ptr = crate::ffi::$dataptr_unchecked(vec);
                        let out = std::slice::from_raw_parts_mut(ptr, n);
                        for (slot, val) in out.iter_mut().zip(self.into_iter()) {
                            *slot = val.unwrap_or($na_value);
                        }
                    }

                    crate::ffi::Rf_unprotect(1);
                    vec
                }
            }
        }
    };
}

impl_vec_option_into_r!(f64, REALSXP, REAL, REAL_unchecked, NA_REAL); // NA_real_
impl_vec_option_into_r!(i32, INTSXP, INTEGER, INTEGER_unchecked, NA_INTEGER); // NA_integer_

/// Macro for NA-aware `Vec<Option<T>> → R` smart vector conversion.
/// Checks if all non-None values fit i32 → INTSXP, otherwise REALSXP.
macro_rules! impl_vec_option_smart_i64_into_r {
    ($t:ty, $fits_i32:expr) => {
        impl IntoR for Vec<Option<$t>> {
            fn into_sexp(self) -> crate::ffi::SEXP {
                if self.iter().all(|opt| match opt {
                    Some(x) => $fits_i32(*x),
                    None => true,
                }) {
                    // All values fit i32 — emit INTSXP with NA_INTEGER for None
                    let coerced: Vec<Option<i32>> =
                        self.into_iter().map(|opt| opt.map(|x| x as i32)).collect();
                    coerced.into_sexp()
                } else {
                    // Some values overflow — emit REALSXP with NA_REAL for None
                    let coerced: Vec<Option<f64>> =
                        self.into_iter().map(|opt| opt.map(|x| x as f64)).collect();
                    coerced.into_sexp()
                }
            }
        }
    };
}

// i32::MIN is NA_integer_ in R, so exclude it
impl_vec_option_smart_i64_into_r!(i64, |x: i64| x > i32::MIN as i64 && x <= i32::MAX as i64);
impl_vec_option_smart_i64_into_r!(u64, |x: u64| x <= i32::MAX as u64);
impl_vec_option_smart_i64_into_r!(isize, |x: isize| x > i32::MIN as isize
    && x <= i32::MAX as isize);
impl_vec_option_smart_i64_into_r!(usize, |x: usize| x <= i32::MAX as usize);

/// Macro for `Vec<Option<T>>` where `T` coerces to a type with existing Option impl.
macro_rules! impl_vec_option_coerce_into_r {
    ($from:ty => $to:ty) => {
        impl IntoR for Vec<Option<$from>> {
            fn into_sexp(self) -> crate::ffi::SEXP {
                let coerced: Vec<Option<$to>> =
                    self.into_iter().map(|opt| opt.map(|x| x as $to)).collect();
                coerced.into_sexp()
            }
        }
    };
}

impl_vec_option_coerce_into_r!(i8 => i32);
impl_vec_option_coerce_into_r!(i16 => i32);
impl_vec_option_coerce_into_r!(u16 => i32);
impl_vec_option_coerce_into_r!(u32 => i64); // delegates to smart i64 path
impl_vec_option_coerce_into_r!(f32 => f64);

/// Helper: allocate LGLSXP and fill from an i32 iterator (checked).
fn logical_iter_to_lglsxp(n: usize, iter: impl Iterator<Item = i32>) -> crate::ffi::SEXP {
    unsafe {
        let sexp = ProtectOnce::new(crate::ffi::Rf_allocVector(
            crate::ffi::SEXPTYPE::LGLSXP,
            n as crate::ffi::R_xlen_t,
        ));
        if n > 0 {
            let ptr = crate::ffi::LOGICAL(*sexp);
            let out = std::slice::from_raw_parts_mut(ptr, n);
            for (slot, val) in out.iter_mut().zip(iter) {
                *slot = val;
            }
        }
        *sexp
    }
}

/// Helper: allocate LGLSXP and fill from an i32 iterator (unchecked).
unsafe fn logical_iter_to_lglsxp_unchecked(
    n: usize,
    iter: impl Iterator<Item = i32>,
) -> crate::ffi::SEXP {
    unsafe {
        let sexp = ProtectOnce::new(crate::ffi::Rf_allocVector_unchecked(
            crate::ffi::SEXPTYPE::LGLSXP,
            n as crate::ffi::R_xlen_t,
        ));
        if n > 0 {
            let ptr = crate::ffi::LOGICAL_unchecked(*sexp);
            let out = std::slice::from_raw_parts_mut(ptr, n);
            for (slot, val) in out.iter_mut().zip(iter) {
                *slot = val;
            }
        }
        *sexp
    }
}

/// Convert `Vec<bool>` to R logical vector.
impl IntoR for Vec<bool> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        let n = self.len();
        logical_iter_to_lglsxp(n, self.into_iter().map(|v| v as i32))
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        let n = self.len();
        unsafe { logical_iter_to_lglsxp_unchecked(n, self.into_iter().map(|v| v as i32)) }
    }
}

/// Convert `&[bool]` to R logical vector.
impl IntoR for &[bool] {
    fn into_sexp(self) -> crate::ffi::SEXP {
        let n = self.len();
        logical_iter_to_lglsxp(n, self.iter().map(|&v| v as i32))
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        let n = self.len();
        unsafe { logical_iter_to_lglsxp_unchecked(n, self.iter().map(|&v| v as i32)) }
    }
}

macro_rules! impl_vec_option_logical_into_r {
    ($(#[$meta:meta])* $t:ty, $convert:expr) => {
        $(#[$meta])*
        impl IntoR for Vec<Option<$t>> {
            fn into_sexp(self) -> crate::ffi::SEXP {
                let n = self.len();
                logical_iter_to_lglsxp(n, self.into_iter().map($convert))
            }

            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                let n = self.len();
                unsafe { logical_iter_to_lglsxp_unchecked(n, self.into_iter().map($convert)) }
            }
        }
    };
}

impl_vec_option_logical_into_r!(
    /// Convert `Vec<Option<bool>>` to R logical vector with NA support.
    bool,
    |v: Option<bool>| match v {
        Some(true) => 1,
        Some(false) => 0,
        None => NA_LOGICAL,
    }
);
impl_vec_option_logical_into_r!(
    /// Convert `Vec<Option<Rboolean>>` to R logical vector with NA support.
    crate::ffi::Rboolean,
    |v: Option<crate::ffi::Rboolean>| match v {
        Some(b) => b as i32,
        None => NA_LOGICAL,
    }
);
impl_vec_option_logical_into_r!(
    /// Convert `Vec<Option<RLogical>>` to R logical vector with NA support.
    crate::ffi::RLogical,
    |v: Option<crate::ffi::RLogical>| match v {
        Some(b) => b.to_i32(),
        None => NA_LOGICAL,
    }
);

/// Convert `Vec<Option<String>>` to R character vector with NA support.
///
/// `None` values become `NA_character_` in R.
impl IntoR for Vec<Option<String>> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let sexp = ProtectOnce::new(crate::ffi::Rf_allocVector(
                crate::ffi::SEXPTYPE::STRSXP,
                n as crate::ffi::R_xlen_t,
            ));

            for (i, opt_s) in self.iter().enumerate() {
                let charsxp = match opt_s {
                    Some(s) => crate::ffi::Rf_mkCharLenCE(
                        s.as_ptr().cast(),
                        s.len() as i32,
                        crate::ffi::CE_UTF8,
                    ),
                    None => crate::ffi::R_NaString,
                };
                crate::ffi::SET_STRING_ELT(*sexp, i as crate::ffi::R_xlen_t, charsxp);
            }

            *sexp
        }
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let sexp = ProtectOnce::new(crate::ffi::Rf_allocVector_unchecked(
                crate::ffi::SEXPTYPE::STRSXP,
                n as crate::ffi::R_xlen_t,
            ));

            for (i, opt_s) in self.iter().enumerate() {
                let charsxp = match opt_s {
                    Some(s) => str_to_charsxp_unchecked(s),
                    None => crate::ffi::R_NaString,
                };
                crate::ffi::SET_STRING_ELT_unchecked(*sexp, i as crate::ffi::R_xlen_t, charsxp);
            }

            *sexp
        }
    }
}

// =============================================================================
// Tuple to list conversions
// =============================================================================

/// Macro to implement IntoR for tuples of various sizes.
/// Converts Rust tuples to unnamed R lists (VECSXP).
macro_rules! impl_tuple_into_r {
    // Base case: 2-tuple
    (($($T:ident),+), ($($idx:tt),+), $n:expr) => {
        impl<$($T: IntoR),+> IntoR for ($($T,)+) {
            fn into_sexp(self) -> crate::ffi::SEXP {
                unsafe {
                    let list = crate::ffi::Rf_allocVector(
                        crate::ffi::SEXPTYPE::VECSXP,
                        $n as crate::ffi::R_xlen_t
                    );
                    crate::ffi::Rf_protect(list);

                    $(
                        crate::ffi::SET_VECTOR_ELT(
                            list,
                            $idx as crate::ffi::R_xlen_t,
                            self.$idx.into_sexp()
                        );
                    )+

                    crate::ffi::Rf_unprotect(1);
                    list
                }
            }

            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe {
                    let list = crate::ffi::Rf_allocVector_unchecked(
                        crate::ffi::SEXPTYPE::VECSXP,
                        $n as crate::ffi::R_xlen_t
                    );
                    crate::ffi::Rf_protect(list);

                    $(
                        crate::ffi::SET_VECTOR_ELT_unchecked(
                            list,
                            $idx as crate::ffi::R_xlen_t,
                            self.$idx.into_sexp_unchecked()
                        );
                    )+

                    crate::ffi::Rf_unprotect(1);
                    list
                }
            }
        }
    };
}

// Implement for tuples of sizes 2-8
impl_tuple_into_r!((A, B), (0, 1), 2);
impl_tuple_into_r!((A, B, C), (0, 1, 2), 3);
impl_tuple_into_r!((A, B, C, D), (0, 1, 2, 3), 4);
impl_tuple_into_r!((A, B, C, D, E), (0, 1, 2, 3, 4), 5);
impl_tuple_into_r!((A, B, C, D, E, F), (0, 1, 2, 3, 4, 5), 6);
impl_tuple_into_r!((A, B, C, D, E, F, G), (0, 1, 2, 3, 4, 5, 6), 7);
impl_tuple_into_r!((A, B, C, D, E, F, G, H), (0, 1, 2, 3, 4, 5, 6, 7), 8);

// =============================================================================
// Result conversions
// =============================================================================

/// Convert `Result<T, E>` to R (value-style, for `#[miniextendr(unwrap_in_r)]`).
///
/// # Behavior
///
/// - `Ok(value)` → returns the converted value directly
/// - `Err(msg)` → returns `list(error = "<msg>")` (value-style error)
///
/// # When This Is Used
///
/// This impl is **only used** when `#[miniextendr(unwrap_in_r)]` is specified.
/// Without that attribute, `#[miniextendr]` functions returning `Result<T, E>`
/// will unwrap in Rust and raise an R error on `Err` (error boundary semantics).
///
/// # Error Handling Summary
///
/// | Mode | On `Err(e)` | Bound Required |
/// |------|-------------|----------------|
/// | Default | R error via panic | `E: Debug` |
/// | `unwrap_in_r` | `list(error = ...)` | `E: Display` |
///
/// **Default** (without `unwrap_in_r`): `Result<T, E>` acts as an error boundary:
/// - `Ok(v)` → `v` converted to R
/// - `Err(e)` → R error with Debug-formatted message (requires `E: Debug`)
///
/// **With `unwrap_in_r`**: `Result<T, E>` is passed through to R:
/// - `Ok(v)` → `v` converted to R
/// - `Err(e)` → `list(error = e.to_string())` (requires `E: Display`)
///
/// # Example
///
/// ```ignore
/// // Default: error boundary - Err becomes R stop()
/// #[miniextendr]
/// fn divide(x: f64, y: f64) -> Result<f64, String> {
///     if y == 0.0 { Err("division by zero".into()) }
///     else { Ok(x / y) }
/// }
/// // In R: tryCatch(divide(1, 0), error = ...) catches the error
///
/// // Value-style: Err becomes list(error = ...)
/// #[miniextendr(unwrap_in_r)]
/// fn divide_safe(x: f64, y: f64) -> Result<f64, String> {
///     if y == 0.0 { Err("division by zero".into()) }
///     else { Ok(x / y) }
/// }
/// // In R: result <- divide_safe(1, 0)
/// //       if (!is.null(result$error)) { handle error }
/// ```
impl<T, E> IntoR for Result<T, E>
where
    T: IntoR,
    E: std::fmt::Display,
{
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Ok(value) => value.into_sexp(),
            Err(msg) => {
                // Create list(error = msg) for R-side error handling
                let mut map = HashMap::with_capacity(1);
                map.insert("error".to_string(), msg.to_string());
                map.into_sexp()
            }
        }
    }
}

/// Marker type for `Result<T, ()>` that converts `Err(())` to NULL.
///
/// This type is used internally by the `#[miniextendr]` macro when handling
/// `Result<T, ()>` return types. When the error type is `()`, there's no
/// error message to report, so we return NULL instead of raising an error.
///
/// # Usage
///
/// You typically don't use this directly. When you write:
///
/// ```ignore
/// #[miniextendr]
/// fn maybe_value(x: i32) -> Result<i32, ()> {
///     if x > 0 { Ok(x) } else { Err(()) }
/// }
/// ```
///
/// The macro generates code that converts `Err(())` to `Err(NullOnErr)` and
/// returns `NULL` in R.
///
/// # Note
///
/// `NullOnErr` intentionally does NOT implement `Display` to avoid conflicting
/// with the generic `IntoR for Result<T, E: Display>` impl. It has its own
/// specialized `IntoR` impl that returns NULL on error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NullOnErr;

/// Convert `Result<T, NullOnErr>` to R, returning NULL on error.
///
/// This is a special case for `Result<T, ()>` types where the error
/// carries no information. Instead of raising an R error, we return NULL.
impl<T: IntoR> IntoR for Result<T, NullOnErr> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Ok(value) => value.into_sexp(),
            Err(NullOnErr) => unsafe { crate::ffi::R_NilValue },
        }
    }
}

// =============================================================================
// ALTREP zero-copy extension trait
// =============================================================================

/// Extension trait for ALTREP conversions.
///
/// This trait provides ergonomic methods for converting Rust types to R ALTREP
/// vectors without copying data. The data stays in Rust memory (wrapped in an
/// ExternalPtr) and R accesses it via ALTREP callbacks.
///
/// # Performance Characteristics
///
/// | Operation | Regular (IntoR) | ALTREP (IntoRAltrep) |
/// |-----------|-----------------|------------------------|
/// | Creation | O(n) copy | O(1) wrap |
/// | Memory | Duplicated in R | Single copy in Rust |
/// | Element access | Direct pointer | Callback (~10ns overhead) |
/// | DATAPTR ops | O(1) | O(1) if Vec/Box, N/A if lazy |
///
/// # When to Use ALTREP
///
/// **Good candidates**:
/// - ✅ Large vectors (>1000 elements)
/// - ✅ Lazy/computed data (avoid eager materialization)
/// - ✅ External data sources (files, databases, APIs)
/// - ✅ Data that might not be fully accessed by R
///
/// **Not recommended**:
/// - ❌ Small vectors (<100 elements) - copy overhead is negligible
/// - ❌ Data R will immediately modify (triggers copy anyway)
/// - ❌ Temporary results (extra indirection not worth it)
///
/// # Example
///
/// ```rust,ignore
/// use miniextendr_api::{miniextendr, IntoRAltrep, IntoR, ffi::SEXP};
///
/// #[miniextendr]
/// fn large_dataset() -> SEXP {
///     let data: Vec<f64> = (0..1_000_000).map(|i| i as f64).collect();
///
///     // Zero-copy: wraps pointer instead of copying 1M elements
///     data.into_sexp_altrep()
/// }
///
/// #[miniextendr]
/// fn small_result() -> SEXP {
///     let data = vec![1, 2, 3, 4, 5];
///
///     // Regular copy is fine for small data
///     data.into_sexp()
/// }
/// ```
pub trait IntoRAltrep {
    /// Convert to R SEXP using ALTREP zero-copy representation.
    ///
    /// This is equivalent to `Altrep(self).into_sexp()` but more discoverable
    /// and explicit about the zero-copy intent.
    fn into_sexp_altrep(self) -> crate::ffi::SEXP;

    /// Create an `Altrep<Self>` wrapper.
    ///
    /// This returns the wrapper explicitly, allowing you to store it or
    /// further process it before conversion.
    fn into_altrep(self) -> Altrep<Self>
    where
        Self: Sized,
    {
        Altrep(self)
    }
}

impl<T> IntoRAltrep for T
where
    T: crate::altrep::RegisterAltrep + crate::externalptr::TypedExternal,
{
    fn into_sexp_altrep(self) -> crate::ffi::SEXP {
        Altrep(self).into_sexp()
    }
}

// =============================================================================
// ALTREP marker type
// =============================================================================

/// Marker type to opt-in to ALTREP representation for types that have both
/// eager-copy and ALTREP implementations.
///
/// # Motivation
///
/// Types like `Vec<i32>` have two possible conversions to R:
/// 1. **Eager copy** (default): copies all data to R immediately
/// 2. **ALTREP**: keeps data in Rust, provides it on-demand to R
///
/// The default `IntoR` for `Vec<i32>` does eager copy. To get ALTREP behavior,
/// wrap your value in `Altrep<T>`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::{miniextendr, Altrep};
///
/// // Returns an ALTREP-backed integer vector (data stays in Rust)
/// #[miniextendr]
/// fn altrep_vec() -> Altrep<Vec<i32>> {
///     Altrep((0..1_000_000).collect())
/// }
///
/// // Returns a regular R vector (data copied to R)
/// #[miniextendr]
/// fn regular_vec() -> Vec<i32> {
///     (0..1_000_000).collect()
/// }
/// ```
///
/// # Supported Types
///
/// `Altrep<T>` works with any type that implements both:
/// - [`RegisterAltrep`](crate::altrep::RegisterAltrep) - for ALTREP class registration
/// - [`TypedExternal`](crate::externalptr::TypedExternal) - for wrapping in ExternalPtr
///
/// Built-in supported types:
/// - `Vec<i32>`, `Vec<f64>`, `Vec<bool>`, `Vec<u8>`, `Vec<String>`
/// - `Box<[i32]>`, `Box<[f64]>`, `Box<[bool]>`, `Box<[u8]>`, `Box<[String]>`
/// - `Range<i32>`, `Range<i64>`, `Range<f64>`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Altrep<T>(pub T);

impl<T> Altrep<T> {
    /// Create a new ALTREP marker wrapper.
    #[inline]
    pub fn new(value: T) -> Self {
        Altrep(value)
    }

    /// Unwrap and return the inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Altrep<T> {
    #[inline]
    fn from(value: T) -> Self {
        Altrep(value)
    }
}

impl<T> std::ops::Deref for Altrep<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Altrep<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Convert `Altrep<T>` to R using ALTREP representation.
///
/// This creates an ALTREP object where the data stays in Rust and is
/// provided to R on-demand through ALTREP callbacks.
impl<T> IntoR for Altrep<T>
where
    T: crate::altrep::RegisterAltrep + crate::externalptr::TypedExternal,
{
    fn into_sexp(self) -> crate::ffi::SEXP {
        let cls = <T as crate::altrep::RegisterAltrep>::get_or_init_class();
        let ext_ptr = crate::externalptr::ExternalPtr::new(self.0);
        let data1 = ext_ptr.as_sexp();
        // Protect data1 across R_new_altrep — it may allocate and trigger GC.
        unsafe {
            crate::ffi::Rf_protect_unchecked(data1);
            let out = crate::ffi::altrep::R_new_altrep(cls, data1, crate::ffi::SEXP::null());
            crate::ffi::Rf_unprotect_unchecked(1);
            out
        }
    }
}

// =============================================================================
// Additional collection type conversions for DataFrameRow support
// =============================================================================

/// Convert `Vec<Box<[T]>>` to R list of vectors (for RNativeType elements).
/// Each boxed slice becomes an R vector.
impl<T> IntoR for Vec<Box<[T]>>
where
    T: crate::ffi::RNativeType,
{
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let list =
                crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::VECSXP, n as crate::ffi::R_xlen_t);
            crate::ffi::Rf_protect(list);

            for (i, boxed_slice) in self.into_iter().enumerate() {
                let vec: Vec<T> = boxed_slice.into_vec();
                let inner_sexp = vec.into_sexp();
                crate::ffi::SET_VECTOR_ELT(list, i as crate::ffi::R_xlen_t, inner_sexp);
            }

            crate::ffi::Rf_unprotect(1);
            list
        }
    }
}

/// Convert `Vec<Box<[String]>>` to R list of character vectors.
impl IntoR for Vec<Box<[String]>> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let list =
                crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::VECSXP, n as crate::ffi::R_xlen_t);
            crate::ffi::Rf_protect(list);

            for (i, boxed_slice) in self.into_iter().enumerate() {
                let vec: Vec<String> = boxed_slice.into_vec();
                let inner_sexp = vec.into_sexp();
                crate::ffi::SET_VECTOR_ELT(list, i as crate::ffi::R_xlen_t, inner_sexp);
            }

            crate::ffi::Rf_unprotect(1);
            list
        }
    }
}

/// Convert `Vec<[T; N]>` to R list of vectors.
/// Each array becomes an R vector.
impl<T, const N: usize> IntoR for Vec<[T; N]>
where
    T: crate::ffi::RNativeType,
{
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let len = self.len();
            let list = crate::ffi::Rf_allocVector(
                crate::ffi::SEXPTYPE::VECSXP,
                len as crate::ffi::R_xlen_t,
            );
            crate::ffi::Rf_protect(list);

            for (i, array) in self.into_iter().enumerate() {
                let vec: Vec<T> = array.into();
                let inner_sexp = vec.into_sexp();
                crate::ffi::SET_VECTOR_ELT(list, i as crate::ffi::R_xlen_t, inner_sexp);
            }

            crate::ffi::Rf_unprotect(1);
            list
        }
    }
}

/// Helper: convert a Vec of IntoR items to an R list (VECSXP).
fn vec_of_into_r_to_list<T: IntoR>(items: Vec<T>) -> crate::ffi::SEXP {
    unsafe {
        let n = items.len();
        let list = ProtectOnce::new(crate::ffi::Rf_allocVector(
            crate::ffi::SEXPTYPE::VECSXP,
            n as crate::ffi::R_xlen_t,
        ));
        for (i, item) in items.into_iter().enumerate() {
            crate::ffi::SET_VECTOR_ELT(*list, i as crate::ffi::R_xlen_t, item.into_sexp());
        }
        *list
    }
}

/// Convert `Vec<HashSet<T>>` to R list of vectors (for RNativeType elements).
/// Each HashSet becomes an R vector (unordered).
impl<T: crate::ffi::RNativeType> IntoR for Vec<std::collections::HashSet<T>> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        let converted: Vec<Vec<T>> = self.into_iter().map(|s| s.into_iter().collect()).collect();
        vec_of_into_r_to_list(converted)
    }
}

/// Convert `Vec<BTreeSet<T>>` to R list of vectors (for RNativeType elements).
/// Each BTreeSet becomes an R vector (sorted).
impl<T: crate::ffi::RNativeType> IntoR for Vec<std::collections::BTreeSet<T>> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        let converted: Vec<Vec<T>> = self.into_iter().map(|s| s.into_iter().collect()).collect();
        vec_of_into_r_to_list(converted)
    }
}

/// Convert `Vec<HashSet<String>>` to R list of character vectors.
impl IntoR for Vec<std::collections::HashSet<String>> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        let converted: Vec<Vec<String>> =
            self.into_iter().map(|s| s.into_iter().collect()).collect();
        vec_of_into_r_to_list(converted)
    }
}

/// Convert `Vec<BTreeSet<String>>` to R list of character vectors.
impl IntoR for Vec<std::collections::BTreeSet<String>> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        let converted: Vec<Vec<String>> =
            self.into_iter().map(|s| s.into_iter().collect()).collect();
        vec_of_into_r_to_list(converted)
    }
}

macro_rules! impl_vec_map_into_r {
    ($(#[$meta:meta])* $map_ty:ident) => {
        $(#[$meta])*
        impl<V: IntoR> IntoR for Vec<$map_ty<String, V>> {
            fn into_sexp(self) -> crate::ffi::SEXP {
                vec_of_maps_to_list(self)
            }
        }
    };
}

impl_vec_map_into_r!(
    /// Convert `Vec<HashMap<String, V>>` to R list of named lists.
    HashMap
);
impl_vec_map_into_r!(
    /// Convert `Vec<BTreeMap<String, V>>` to R list of named lists.
    BTreeMap
);

/// Helper to convert a Vec of map-like types to an R list of named lists.
fn vec_of_maps_to_list<T: IntoR>(vec: Vec<T>) -> crate::ffi::SEXP {
    unsafe {
        let n = vec.len();
        let list =
            crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::VECSXP, n as crate::ffi::R_xlen_t);
        crate::ffi::Rf_protect(list);

        for (i, map) in vec.into_iter().enumerate() {
            crate::ffi::SET_VECTOR_ELT(list, i as crate::ffi::R_xlen_t, map.into_sexp());
        }

        crate::ffi::Rf_unprotect(1);
        list
    }
}
