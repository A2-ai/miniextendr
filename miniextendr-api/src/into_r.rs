#![allow(rustdoc::private_intra_doc_links)]
//! Conversions from Rust types to R SEXP.
//!
//! This module provides the [`IntoR`] trait for converting Rust values to R SEXPs.
//!
//! # Submodules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`large_integers`] | `i64`, `u64`, `isize`, `usize` → REALSXP, plus string/bool/Option scalars |
//! | [`collections`] | `HashMap`, `BTreeMap`, `HashSet`, `BTreeSet` → named/unnamed lists |
//! | [`result`] | `Result<T, E>` → list with `ok`/`err` fields |
//! | [`altrep`] | `Altrep<T>` marker type, `Lazy<T>` alias, `IntoRAltrep` trait |
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

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

use crate::altrep_traits::{NA_INTEGER, NA_LOGICAL, NA_REAL};
use crate::ffi::SexpExt;
use crate::gc_protect::OwnedProtect;

/// Trait for converting Rust types to R SEXP values.
///
/// # Required Method
///
/// Implementors must provide [`try_into_sexp`](IntoR::try_into_sexp) and
/// specify [`Error`](IntoR::Error). The other three methods have sensible
/// defaults.
///
/// # Examples
///
/// ```no_run
/// use miniextendr_api::into_r::IntoR;
///
/// let sexp = 42i32.into_sexp();
/// let sexp = "hello".to_string().into_sexp();
///
/// // Fallible path:
/// let result = "hello".try_into_sexp();
/// assert!(result.is_ok());
/// ```
pub trait IntoR {
    /// The error type for fallible conversions.
    ///
    /// Use [`std::convert::Infallible`] for types that can never fail.
    /// Use [`IntoRError`](crate::into_r_error::IntoRError) for types
    /// that may fail (e.g. strings exceeding R's i32 length limit).
    type Error: std::fmt::Display;

    /// Try to convert this value to an R SEXP.
    ///
    /// This is the **required** method. All other methods delegate to it.
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error>;

    /// Try to convert to SEXP without thread safety checks.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread.
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error>
    where
        Self: Sized,
    {
        self.try_into_sexp()
    }

    /// Convert this value to an R SEXP, panicking on error.
    ///
    /// In debug builds, asserts that we're on R's main thread.
    fn into_sexp(self) -> crate::ffi::SEXP
    where
        Self: Sized,
    {
        match self.try_into_sexp() {
            Ok(sexp) => sexp,
            Err(e) => panic!("IntoR conversion failed: {e}"),
        }
    }

    /// Convert to SEXP without thread safety checks, panicking on error.
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
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self)
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self)
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self
    }
}

impl IntoR for crate::worker::Sendable<crate::ffi::SEXP> {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.0)
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.0)
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.0
    }
}

impl From<crate::worker::Sendable<crate::ffi::SEXP>> for crate::ffi::SEXP {
    #[inline]
    fn from(s: crate::worker::Sendable<crate::ffi::SEXP>) -> Self {
        s.0
    }
}

impl IntoR for () {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(crate::ffi::SEXP::nil())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        crate::ffi::SEXP::nil()
    }
}

impl IntoR for std::convert::Infallible {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(crate::ffi::SEXP::nil())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        crate::ffi::SEXP::nil()
    }
}

/// Macro for scalar IntoR via SEXP::scalar_* methods.
macro_rules! impl_scalar_into_r {
    ($ty:ty, $checked:ident, $unchecked:ident) => {
        impl IntoR for $ty {
            type Error = std::convert::Infallible;
            #[inline]
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(crate::ffi::SEXP::$checked(self))
            }
            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                crate::ffi::SEXP::$checked(self)
            }
            #[inline]
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe { crate::ffi::SEXP::$unchecked(self) }
            }
        }
    };
}

impl_scalar_into_r!(i32, scalar_integer, scalar_integer_unchecked);
impl_scalar_into_r!(f64, scalar_real, scalar_real_unchecked);
impl_scalar_into_r!(u8, scalar_raw, scalar_raw_unchecked);
impl_scalar_into_r!(
    crate::ffi::Rcomplex,
    scalar_complex,
    scalar_complex_unchecked
);

impl IntoR for Option<crate::ffi::Rcomplex> {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => crate::ffi::SEXP::scalar_complex(crate::ffi::Rcomplex {
                r: NA_REAL,
                i: NA_REAL,
            }),
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe {
                crate::ffi::SEXP::scalar_complex_unchecked(crate::ffi::Rcomplex {
                    r: NA_REAL,
                    i: NA_REAL,
                })
            },
        }
    }
}

/// Macro for infallible widening IntoR via Coerce.
macro_rules! impl_into_r_via_coerce {
    ($from:ty => $to:ty) => {
        impl IntoR for $from {
            type Error = std::convert::Infallible;
            #[inline]
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(crate::coerce::Coerce::<$to>::coerce(self).into_sexp())
            }
            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
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

mod large_integers;
pub(crate) use large_integers::{str_to_charsxp, str_to_charsxp_unchecked};

// region: Vector conversions

// Concrete IntoR impls for Vec<T> where T: RNativeType.
//
// These are written as concrete impls rather than a blanket
// `impl<T: RNativeType> IntoR for Vec<T>` to avoid a coherence conflict
// with `impl<T: MatchArg> IntoR for Vec<T>` in match_arg.rs.
// Both blankets would target `Vec<_>` for the same foreign trait, and the
// Rust coherence checker (E0119) rejects them even when the bounds are
// disjoint in practice, because negative trait bounds are not stable.
macro_rules! impl_into_r_vec_native {
    ($t:ty) => {
        impl IntoR for Vec<$t> {
            type Error = std::convert::Infallible;
            #[inline]
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { vec_to_sexp(&self) })
            }
            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                unsafe { vec_to_sexp(&self) }
            }
            #[inline]
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe { vec_to_sexp_unchecked(&self) }
            }
        }
    };
}

impl_into_r_vec_native!(i32);
impl_into_r_vec_native!(f64);
impl_into_r_vec_native!(u8);
impl_into_r_vec_native!(crate::ffi::RLogical);
impl_into_r_vec_native!(crate::ffi::Rcomplex);

impl<T> IntoR for &[T]
where
    T: crate::ffi::RNativeType,
{
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { vec_to_sexp(self) })
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { vec_to_sexp(self) }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { vec_to_sexp_unchecked(self) }
    }
}

impl<T> IntoR for Box<[T]>
where
    T: crate::ffi::RNativeType,
{
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { vec_to_sexp(&self) })
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { vec_to_sexp_unchecked(&self) })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { vec_to_sexp(&self) }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { vec_to_sexp_unchecked(&self) }
    }
}

// region: R vector allocation helpers
//
// These are the ONLY place in the codebase that should call Rf_allocVector
// for typed vectors and obtain a mutable data slice. All conversion code
// uses these helpers instead of raw FFI pointer arithmetic.

/// Allocate an R vector of type `T` with `n` elements and return `(SEXP, &mut [T])`.
///
/// The returned SEXP is **unprotected** — caller must protect via `Rf_protect`,
/// `OwnedProtect`, or `ProtectScope` before any further R allocation.
///
/// # Safety
///
/// Must be called from R's main thread.
#[inline]
pub(crate) unsafe fn alloc_r_vector<T: crate::ffi::RNativeType>(
    n: usize,
) -> (crate::ffi::SEXP, &'static mut [T]) {
    unsafe {
        let sexp = crate::ffi::Rf_allocVector(T::SEXP_TYPE, n as crate::ffi::R_xlen_t);
        let slice = crate::from_r::r_slice_mut(T::dataptr_mut(sexp), n);
        (sexp, slice)
    }
}

/// Allocate an R vector (unchecked FFI variant).
///
/// # Safety
///
/// Must be called from R's main thread.
#[inline]
pub(crate) unsafe fn alloc_r_vector_unchecked<T: crate::ffi::RNativeType>(
    n: usize,
) -> (crate::ffi::SEXP, &'static mut [T]) {
    unsafe {
        let sexp = crate::ffi::Rf_allocVector_unchecked(T::SEXP_TYPE, n as crate::ffi::R_xlen_t);
        let slice = crate::from_r::r_slice_mut(T::dataptr_mut(sexp), n);
        (sexp, slice)
    }
}

// endregion

/// Convert a slice to an R vector (checked) using `copy_from_slice`.
#[inline]
unsafe fn vec_to_sexp<T: crate::ffi::RNativeType>(slice: &[T]) -> crate::ffi::SEXP {
    unsafe {
        let (sexp, dst) = alloc_r_vector::<T>(slice.len());
        dst.copy_from_slice(slice);
        sexp
    }
}

/// Convert a slice to an R vector (unchecked) using `copy_from_slice`.
#[inline]
unsafe fn vec_to_sexp_unchecked<T: crate::ffi::RNativeType>(slice: &[T]) -> crate::ffi::SEXP {
    unsafe {
        let (sexp, dst) = alloc_r_vector_unchecked::<T>(slice.len());
        dst.copy_from_slice(slice);
        sexp
    }
}
// endregion

// region: Vec coercion for non-native types (i8, i16, u16 → i32; f32 → f64)

/// Macro for `Vec<T>` where `T` coerces to a native R type.
///
/// Allocates the R vector directly and coerces in-place — no intermediate Vec.
macro_rules! impl_vec_coerce_into_r {
    ($from:ty => $to:ty) => {
        impl IntoR for Vec<$from> {
            type Error = std::convert::Infallible;
            #[inline]
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                unsafe {
                    let (sexp, dst) = alloc_r_vector::<$to>(self.len());
                    for (slot, val) in dst.iter_mut().zip(self.into_iter()) {
                        *slot = <$to>::from(val);
                    }
                    sexp
                }
            }
            #[inline]
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe {
                    let (sexp, dst) = alloc_r_vector_unchecked::<$to>(self.len());
                    for (slot, val) in dst.iter_mut().zip(self.into_iter()) {
                        *slot = <$to>::from(val);
                    }
                    sexp
                }
            }
        }

        impl IntoR for &[$from] {
            type Error = std::convert::Infallible;
            #[inline]
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                unsafe {
                    let (sexp, dst) = alloc_r_vector::<$to>(self.len());
                    for (slot, &val) in dst.iter_mut().zip(self.iter()) {
                        *slot = <$to>::from(val);
                    }
                    sexp
                }
            }
            #[inline]
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe {
                    let (sexp, dst) = alloc_r_vector_unchecked::<$to>(self.len());
                    for (slot, &val) in dst.iter_mut().zip(self.iter()) {
                        *slot = <$to>::from(val);
                    }
                    sexp
                }
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
//
// Allocates the R vector directly and coerces in-place — no intermediate Vec.
macro_rules! impl_vec_smart_i64_into_r {
    ($t:ty, $fits_i32:expr) => {
        impl IntoR for Vec<$t> {
            type Error = std::convert::Infallible;
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
            fn into_sexp(self) -> crate::ffi::SEXP {
                unsafe {
                    if self.iter().all(|&x| $fits_i32(x)) {
                        let (sexp, dst) = alloc_r_vector::<i32>(self.len());
                        for (slot, val) in dst.iter_mut().zip(self.into_iter()) {
                            // fits_i32 guard verified range
                            *slot = val as i32;
                        }
                        sexp
                    } else {
                        let (sexp, dst) = alloc_r_vector::<f64>(self.len());
                        for (slot, val) in dst.iter_mut().zip(self.into_iter()) {
                            // R has no 64-bit integer; f64 loses precision > 2^53
                            *slot = val as f64;
                        }
                        sexp
                    }
                }
            }
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe {
                    if self.iter().all(|&x| $fits_i32(x)) {
                        let (sexp, dst) = alloc_r_vector_unchecked::<i32>(self.len());
                        for (slot, val) in dst.iter_mut().zip(self.into_iter()) {
                            // fits_i32 guard verified range
                            *slot = val as i32;
                        }
                        sexp
                    } else {
                        let (sexp, dst) = alloc_r_vector_unchecked::<f64>(self.len());
                        for (slot, val) in dst.iter_mut().zip(self.into_iter()) {
                            // R has no 64-bit integer; f64 loses precision > 2^53
                            *slot = val as f64;
                        }
                        sexp
                    }
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
// endregion

mod altrep;
mod collections;
mod result;

pub use altrep::*;
pub use result::*;

// region: Fixed-size array conversions

/// Blanket impl for `[T; N]` where T: RNativeType.
///
/// Enables direct conversion of fixed-size arrays to R vectors.
/// Useful for SHA hashes, fixed-size byte patterns, etc.
impl<T: crate::ffi::RNativeType, const N: usize> IntoR for [T; N] {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.as_slice().into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_slice().into_sexp()
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { self.as_slice().into_sexp_unchecked() }
    }
}
// endregion

// region: VecDeque conversions

use std::collections::VecDeque;

/// Convert `VecDeque<T>` to R vector where T: RNativeType.
impl<T> IntoR for VecDeque<T>
where
    T: crate::ffi::RNativeType,
{
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        let vec: Vec<T> = self.into_iter().collect();
        vec.into_sexp()
    }
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        let vec: Vec<T> = self.into_iter().collect();
        unsafe { vec.into_sexp_unchecked() }
    }
}
// endregion

// region: BinaryHeap conversions

use std::collections::BinaryHeap;

/// Convert `BinaryHeap<T>` to R vector where T: RNativeType + Ord.
///
/// The heap is drained into a vector (destroying the heap property).
/// Elements are returned in arbitrary order, not sorted.
impl<T> IntoR for BinaryHeap<T>
where
    T: crate::ffi::RNativeType + Ord,
{
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_vec().into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.into_vec().into_sexp()
    }
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { self.into_vec().into_sexp_unchecked() }
    }
}
// endregion

// region: Cow conversions

use std::borrow::Cow;

/// Try SEXP pointer recovery for a borrowed Cow slice.
#[inline]
#[allow(clippy::ptr_arg)] // Need &Cow to inspect Borrowed vs Owned variant
fn try_recover_cow_slice<T: crate::ffi::RNativeType>(
    cow: &Cow<'_, [T]>,
) -> Option<crate::ffi::SEXP> {
    if let Cow::Borrowed(slice) = cow {
        unsafe {
            crate::r_memory::try_recover_r_sexp(
                slice.as_ptr() as *const u8,
                T::SEXP_TYPE,
                slice.len(),
            )
        }
    } else {
        None
    }
}

/// Convert `Cow<'_, [T]>` to R vector where T: RNativeType.
///
/// For `Cow::Borrowed` slices that came from R (e.g., via `TryFromSexp`),
/// SEXP pointer recovery is attempted — if the borrowed data points into
/// an R vector, the original SEXP is returned without copying. Otherwise
/// falls through to the standard copy path.
impl<T> IntoR for Cow<'_, [T]>
where
    T: crate::ffi::RNativeType + Clone,
{
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        if let Some(sexp) = try_recover_cow_slice(&self) {
            return sexp;
        }
        self.as_ref().into_sexp()
    }
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        if let Some(sexp) = try_recover_cow_slice(&self) {
            return sexp;
        }
        unsafe { self.as_ref().into_sexp_unchecked() }
    }
}

/// Convert `Cow<'_, str>` to R character scalar.
impl IntoR for Cow<'_, str> {
    type Error = crate::into_r_error::IntoRError;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.as_ref().try_into_sexp()
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_ref().into_sexp()
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { self.as_ref().into_sexp_unchecked() }
    }
}
// endregion

// region: Box conversions (skipped - conflicts with IntoExternalPtr blanket impl)
//
// We can't add `impl<T: IntoR> IntoR for Box<T>` because it conflicts with
// the blanket impl `impl<T: IntoExternalPtr> IntoR for T`. If downstream
// crates implement `IntoExternalPtr for Box<SomeType>`, we'd have overlapping
// impls. Users can manually unbox with `*boxed_value` before conversion.
// endregion

// region: PathBuf / OsString conversions

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
            type Error = crate::into_r_error::IntoRError;
            #[inline]
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                self.to_string_lossy().into_owned().try_into_sexp()
            }
            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
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
            type Error = crate::into_r_error::IntoRError;
            #[inline]
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                self.to_string_lossy().into_owned().try_into_sexp()
            }
            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
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
            type Error = crate::into_r_error::IntoRError;
            #[inline]
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                self.map(|v| v.to_string_lossy().into_owned()).try_into_sexp()
            }
            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
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
            type Error = crate::into_r_error::IntoRError;
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
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
            type Error = crate::into_r_error::IntoRError;
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
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
// endregion

// region: Set coercion for non-native types (i8, i16, u16 → i32)

/// Macro for `HashSet<T>`/`BTreeSet<T>` where `T` coerces to i32 (R's native integer type).
macro_rules! impl_set_coerce_into_r {
    ($from:ty) => {
        impl IntoR for HashSet<$from> {
            type Error = std::convert::Infallible;
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                self.try_into_sexp()
            }
            fn into_sexp(self) -> crate::ffi::SEXP {
                let vec: Vec<i32> = self.into_iter().map(|x| i32::from(x)).collect();
                vec.into_sexp()
            }
        }

        impl IntoR for BTreeSet<$from> {
            type Error = std::convert::Infallible;
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                self.try_into_sexp()
            }
            fn into_sexp(self) -> crate::ffi::SEXP {
                let vec: Vec<i32> = self.into_iter().map(|x| i32::from(x)).collect();
                vec.into_sexp()
            }
        }
    };
}

// Sub-i32 integer types in sets coerce to i32 (R's INTSXP)
impl_set_coerce_into_r!(i8);
impl_set_coerce_into_r!(i16);
impl_set_coerce_into_r!(u16);
// endregion

// region: Option<Collection> conversions
//
// These return NULL (R_NilValue) for None, and the converted collection for Some.
// This differs from Option<scalar> which returns NA for None.

/// Convert `Option<Vec<T>>` to R: Some(vec) → vector, None → NULL.
impl<T: crate::ffi::RNativeType> IntoR for Option<Vec<T>> {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => crate::ffi::SEXP::nil(),
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => crate::ffi::SEXP::nil(),
        }
    }
}

/// Convert `Option<Vec<String>>` to R: Some(vec) → character vector, None → NULL.
impl IntoR for Option<Vec<String>> {
    type Error = crate::into_r_error::IntoRError;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => crate::ffi::SEXP::nil(),
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => crate::ffi::SEXP::nil(),
        }
    }
}

/// Convert `Option<HashMap<String, V>>` to R: Some(map) -> named list, None -> NULL.
impl<V: IntoR> IntoR for Option<HashMap<String, V>> {
    type Error = crate::into_r_error::IntoRError;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => crate::ffi::SEXP::nil(),
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => crate::ffi::SEXP::nil(),
        }
    }
}

/// Convert `Option<BTreeMap<String, V>>` to R: Some(map) -> named list, None -> NULL.
impl<V: IntoR> IntoR for Option<BTreeMap<String, V>> {
    type Error = crate::into_r_error::IntoRError;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => crate::ffi::SEXP::nil(),
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => crate::ffi::SEXP::nil(),
        }
    }
}

/// Convert `Option<HashSet<T>>` to R: Some(set) -> vector, None -> NULL.
impl<T: crate::ffi::RNativeType + Eq + Hash> IntoR for Option<HashSet<T>> {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => crate::ffi::SEXP::nil(),
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => crate::ffi::SEXP::nil(),
        }
    }
}

/// Convert `Option<BTreeSet<T>>` to R: Some(set) -> vector, None -> NULL.
impl<T: crate::ffi::RNativeType + Ord> IntoR for Option<BTreeSet<T>> {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => crate::ffi::SEXP::nil(),
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => crate::ffi::SEXP::nil(),
        }
    }
}

macro_rules! impl_option_collection_into_r {
    ($(#[$meta:meta])* $ty:ty) => {
        $(#[$meta])*
        impl IntoR for Option<$ty> {
            type Error = crate::into_r_error::IntoRError;
            #[inline]
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                match self {
                    Some(v) => v.into_sexp(),
                    None => crate::ffi::SEXP::nil(),
                }
            }
            #[inline]
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                match self {
                    Some(v) => unsafe { v.into_sexp_unchecked() },
                    None => crate::ffi::SEXP::nil(),
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
pub(crate) fn str_iter_to_strsxp<'a>(
    iter: impl ExactSizeIterator<Item = &'a str>,
) -> crate::ffi::SEXP {
    unsafe {
        let n: crate::ffi::R_xlen_t = iter
            .len()
            .try_into()
            .expect("string vec length exceeds isize::MAX");
        let sexp = OwnedProtect::new(crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::STRSXP, n));
        for (i, s) in iter.enumerate() {
            let idx: crate::ffi::R_xlen_t = i.try_into().expect("index exceeds isize::MAX");
            let charsxp = str_to_charsxp(s);
            sexp.set_string_elt(idx, charsxp);
        }
        *sexp
    }
}

/// Helper: allocate STRSXP and fill from a string iterator (unchecked).
pub(crate) unsafe fn str_iter_to_strsxp_unchecked<'a>(
    iter: impl ExactSizeIterator<Item = &'a str>,
) -> crate::ffi::SEXP {
    unsafe {
        let n: crate::ffi::R_xlen_t = iter
            .len()
            .try_into()
            .expect("string vec length exceeds isize::MAX");
        let sexp = OwnedProtect::new(crate::ffi::Rf_allocVector_unchecked(
            crate::ffi::SEXPTYPE::STRSXP,
            n,
        ));
        for (i, s) in iter.enumerate() {
            let idx: crate::ffi::R_xlen_t = i.try_into().expect("index exceeds isize::MAX");
            let charsxp = str_to_charsxp_unchecked(s);
            sexp.set_string_elt_unchecked(idx, charsxp);
        }
        *sexp
    }
}

/// Convert `Vec<String>` to R character vector (STRSXP).
impl IntoR for Vec<String> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        str_iter_to_strsxp(self.iter().map(|s| s.as_str()))
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { str_iter_to_strsxp_unchecked(self.iter().map(|s| s.as_str())) }
    }
}

/// Convert `&[String]` to R character vector (STRSXP).
impl IntoR for &[String] {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        str_iter_to_strsxp(self.iter().map(|s| s.as_str()))
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { str_iter_to_strsxp_unchecked(self.iter().map(|s| s.as_str())) }
    }
}

/// Convert `Box<[String]>` to R character vector (STRSXP).
impl IntoR for Box<[String]> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        str_iter_to_strsxp(self.iter().map(|s| s.as_str()))
    }
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { str_iter_to_strsxp_unchecked(self.iter().map(|s| s.as_str())) }
    }
}

/// Convert &[&str] to R character vector (STRSXP).
impl IntoR for &[&str] {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        str_iter_to_strsxp(self.iter().copied())
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { str_iter_to_strsxp_unchecked(self.iter().copied()) }
    }
}

/// Convert `Vec<Cow<'_, str>>` to R character vector (STRSXP).
impl IntoR for Vec<std::borrow::Cow<'_, str>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        str_iter_to_strsxp(self.iter().map(|s| s.as_ref()))
    }
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { str_iter_to_strsxp_unchecked(self.iter().map(|s| s.as_ref())) }
    }
}

/// Convert `Box<[Cow<'_, str>]>` to R character vector (STRSXP).
impl IntoR for Box<[std::borrow::Cow<'_, str>]> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        str_iter_to_strsxp(self.iter().map(|s| s.as_ref()))
    }
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { str_iter_to_strsxp_unchecked(self.iter().map(|s| s.as_ref())) }
    }
}

/// Convert `Vec<Option<Cow<'_, str>>>` to R character vector with NA support.
///
/// `None` values become `NA_character_` in R.
impl IntoR for Vec<Option<std::borrow::Cow<'_, str>>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let n: crate::ffi::R_xlen_t = self
                .len()
                .try_into()
                .expect("vec length exceeds isize::MAX");
            let sexp =
                OwnedProtect::new(crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::STRSXP, n));
            for (i, opt_s) in self.iter().enumerate() {
                let idx: crate::ffi::R_xlen_t = i.try_into().expect("index exceeds isize::MAX");
                let charsxp = match opt_s {
                    Some(s) => str_to_charsxp(s.as_ref()),
                    None => crate::ffi::SEXP::na_string(),
                };
                sexp.set_string_elt(idx, charsxp);
            }
            *sexp
        }
    }
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe {
            let n: crate::ffi::R_xlen_t = self
                .len()
                .try_into()
                .expect("vec length exceeds isize::MAX");
            let sexp = OwnedProtect::new(crate::ffi::Rf_allocVector_unchecked(
                crate::ffi::SEXPTYPE::STRSXP,
                n,
            ));
            for (i, opt_s) in self.iter().enumerate() {
                let idx: crate::ffi::R_xlen_t = i.try_into().expect("index exceeds isize::MAX");
                let charsxp = match opt_s {
                    Some(s) => str_to_charsxp_unchecked(s.as_ref()),
                    None => crate::ffi::SEXP::na_string(),
                };
                sexp.set_string_elt_unchecked(idx, charsxp);
            }
            *sexp
        }
    }
}

/// Convert `Vec<&str>` to R character vector (STRSXP).
impl IntoR for Vec<&str> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_slice().into_sexp()
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { self.as_slice().into_sexp_unchecked() }
    }
}
// endregion

// region: Nested vector conversions (list of vectors)

/// Convert `Vec<Vec<T>>` to R list of vectors (VECSXP of typed vectors).
impl<T> IntoR for Vec<Vec<T>>
where
    T: crate::ffi::RNativeType,
{
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let list =
                crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::VECSXP, n as crate::ffi::R_xlen_t);
            crate::ffi::Rf_protect(list);

            for (i, inner) in self.into_iter().enumerate() {
                let inner_sexp = inner.into_sexp();
                list.set_vector_elt(i as crate::ffi::R_xlen_t, inner_sexp);
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
                list.set_vector_elt_unchecked(i as crate::ffi::R_xlen_t, inner_sexp);
            }

            crate::ffi::Rf_unprotect(1);
            list
        }
    }
}

/// Convert `Vec<Vec<String>>` to R list of character vectors.
impl IntoR for Vec<Vec<String>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let list =
                crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::VECSXP, n as crate::ffi::R_xlen_t);
            crate::ffi::Rf_protect(list);

            for (i, inner) in self.into_iter().enumerate() {
                let inner_sexp = inner.into_sexp();
                list.set_vector_elt(i as crate::ffi::R_xlen_t, inner_sexp);
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
                list.set_vector_elt_unchecked(i as crate::ffi::R_xlen_t, inner_sexp);
            }

            crate::ffi::Rf_unprotect(1);
            list
        }
    }
}
// endregion

// region: NA-aware vector conversions

/// Macro for NA-aware `Vec<Option<T>> → R` vector conversions.
///
/// Uses `alloc_r_vector` to get a mutable slice, then fills it.
macro_rules! impl_vec_option_into_r {
    ($t:ty, $na_value:expr) => {
        impl IntoR for Vec<Option<$t>> {
            type Error = std::convert::Infallible;
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
            fn into_sexp(self) -> crate::ffi::SEXP {
                unsafe {
                    let (sexp, dst) = alloc_r_vector::<$t>(self.len());
                    for (slot, val) in dst.iter_mut().zip(self.into_iter()) {
                        *slot = val.unwrap_or($na_value);
                    }
                    sexp
                }
            }

            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe {
                    let (sexp, dst) = alloc_r_vector_unchecked::<$t>(self.len());
                    for (slot, val) in dst.iter_mut().zip(self.into_iter()) {
                        *slot = val.unwrap_or($na_value);
                    }
                    sexp
                }
            }
        }
    };
}

impl_vec_option_into_r!(f64, NA_REAL); // NA_real_
impl_vec_option_into_r!(i32, NA_INTEGER); // NA_integer_

/// Macro for NA-aware `Vec<Option<T>> → R` smart vector conversion.
/// Checks if all non-None values fit i32 → INTSXP, otherwise REALSXP.
///
/// Allocates the R vector directly and coerces in-place — no intermediate Vec.
macro_rules! impl_vec_option_smart_i64_into_r {
    ($t:ty, $fits_i32:expr) => {
        impl IntoR for Vec<Option<$t>> {
            type Error = std::convert::Infallible;
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                self.try_into_sexp()
            }
            fn into_sexp(self) -> crate::ffi::SEXP {
                unsafe {
                    if self.iter().all(|opt| match opt {
                        Some(x) => $fits_i32(*x),
                        None => true,
                    }) {
                        let (sexp, dst) = alloc_r_vector::<i32>(self.len());
                        for (slot, val) in dst.iter_mut().zip(self.into_iter()) {
                            *slot = match val {
                                Some(x) => x as i32,
                                None => NA_INTEGER,
                            };
                        }
                        sexp
                    } else {
                        let (sexp, dst) = alloc_r_vector::<f64>(self.len());
                        for (slot, val) in dst.iter_mut().zip(self.into_iter()) {
                            *slot = match val {
                                Some(x) => x as f64,
                                None => NA_REAL,
                            };
                        }
                        sexp
                    }
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
///
/// Delegates to the target type's `Vec<Option<$to>>` impl (which itself uses alloc_r_vector).
macro_rules! impl_vec_option_coerce_into_r {
    ($from:ty => $to:ty) => {
        impl IntoR for Vec<Option<$from>> {
            type Error = std::convert::Infallible;
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                self.try_into_sexp()
            }
            fn into_sexp(self) -> crate::ffi::SEXP {
                // Delegate to the target Option type's impl (coerce inline)
                let coerced: Vec<Option<$to>> = self
                    .into_iter()
                    .map(|opt| opt.map(|x| <$to>::from(x)))
                    .collect();
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
///
/// Uses `alloc_r_vector` — logical vectors are `RLogical` (repr(transparent) i32).
fn logical_iter_to_lglsxp(n: usize, iter: impl Iterator<Item = i32>) -> crate::ffi::SEXP {
    unsafe {
        let (sexp, dst) = alloc_r_vector::<crate::ffi::RLogical>(n);
        // RLogical is repr(transparent) over i32, safe to write i32 values.
        let dst_i32: &mut [i32] = std::slice::from_raw_parts_mut(dst.as_mut_ptr().cast::<i32>(), n);
        for (slot, val) in dst_i32.iter_mut().zip(iter) {
            *slot = val;
        }
        sexp
    }
}

/// Helper: allocate LGLSXP and fill from an i32 iterator (unchecked).
unsafe fn logical_iter_to_lglsxp_unchecked(
    n: usize,
    iter: impl Iterator<Item = i32>,
) -> crate::ffi::SEXP {
    unsafe {
        let (sexp, dst) = alloc_r_vector_unchecked::<crate::ffi::RLogical>(n);
        let dst_i32: &mut [i32] = std::slice::from_raw_parts_mut(dst.as_mut_ptr().cast::<i32>(), n);
        for (slot, val) in dst_i32.iter_mut().zip(iter) {
            *slot = val;
        }
        sexp
    }
}

/// Convert `Vec<bool>` to R logical vector.
impl IntoR for Vec<bool> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        let n = self.len();
        logical_iter_to_lglsxp(n, self.into_iter().map(i32::from))
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        let n = self.len();
        unsafe { logical_iter_to_lglsxp_unchecked(n, self.into_iter().map(i32::from)) }
    }
}

/// Convert `Box<[bool]>` to R logical vector.
impl IntoR for Box<[bool]> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        let n = self.len();
        logical_iter_to_lglsxp(n, self.iter().map(|&v| i32::from(v)))
    }
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        let n = self.len();
        unsafe { logical_iter_to_lglsxp_unchecked(n, self.iter().map(|&v| i32::from(v))) }
    }
}

/// Convert `&[bool]` to R logical vector.
impl IntoR for &[bool] {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        let n = self.len();
        logical_iter_to_lglsxp(n, self.iter().map(|&v| i32::from(v)))
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        let n = self.len();
        unsafe { logical_iter_to_lglsxp_unchecked(n, self.iter().map(|&v| i32::from(v))) }
    }
}

macro_rules! impl_vec_option_logical_into_r {
    ($(#[$meta:meta])* $t:ty, $convert:expr) => {
        $(#[$meta])*
        impl IntoR for Vec<Option<$t>> {
            type Error = std::convert::Infallible;
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
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
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let n: crate::ffi::R_xlen_t = self
                .len()
                .try_into()
                .expect("vec length exceeds isize::MAX");
            let sexp =
                OwnedProtect::new(crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::STRSXP, n));

            for (i, opt_s) in self.iter().enumerate() {
                let idx: crate::ffi::R_xlen_t = i.try_into().expect("index exceeds isize::MAX");
                let charsxp = match opt_s {
                    Some(s) => str_to_charsxp(s),
                    None => crate::ffi::SEXP::na_string(),
                };
                sexp.set_string_elt(idx, charsxp);
            }

            *sexp
        }
    }

    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe {
            let n: crate::ffi::R_xlen_t = self
                .len()
                .try_into()
                .expect("vec length exceeds isize::MAX");
            let sexp = OwnedProtect::new(crate::ffi::Rf_allocVector_unchecked(
                crate::ffi::SEXPTYPE::STRSXP,
                n,
            ));

            for (i, opt_s) in self.iter().enumerate() {
                let idx: crate::ffi::R_xlen_t = i.try_into().expect("index exceeds isize::MAX");
                let charsxp = match opt_s {
                    Some(s) => str_to_charsxp_unchecked(s),
                    None => crate::ffi::SEXP::na_string(),
                };
                sexp.set_string_elt_unchecked(idx, charsxp);
            }

            *sexp
        }
    }
}
// endregion

// region: Tuple to list conversions

/// Macro to implement IntoR for tuples of various sizes.
/// Converts Rust tuples to unnamed R lists (VECSXP).
macro_rules! impl_tuple_into_r {
    // Base case: 2-tuple
    (($($T:ident),+), ($($idx:tt),+), $n:expr) => {
        impl<$($T: IntoR),+> IntoR for ($($T,)+) {
            type Error = std::convert::Infallible;
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
            fn into_sexp(self) -> crate::ffi::SEXP {
                unsafe {
                    let list = crate::ffi::Rf_allocVector(
                        crate::ffi::SEXPTYPE::VECSXP,
                        $n as crate::ffi::R_xlen_t
                    );
                    crate::ffi::Rf_protect(list);

                    $(

                            list.set_vector_elt($idx as crate::ffi::R_xlen_t, self.$idx.into_sexp()
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

                            list.set_vector_elt_unchecked($idx as crate::ffi::R_xlen_t, self.$idx.into_sexp_unchecked()
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
// endregion

// region: ALTREP zero-copy extension trait

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

    /// Convert to R SEXP using ALTREP, skipping debug thread assertions.
    ///
    /// # Safety
    ///
    /// Caller must ensure they are on R's main thread.
    unsafe fn into_sexp_altrep_unchecked(self) -> crate::ffi::SEXP
    where
        Self: Sized,
    {
        self.into_sexp_altrep()
    }

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

    unsafe fn into_sexp_altrep_unchecked(self) -> crate::ffi::SEXP {
        unsafe { Altrep(self).into_sexp_unchecked() }
    }
}
// endregion

// region: Additional collection type conversions for DataFrameRow support

/// Convert `Vec<Box<[T]>>` to R list of vectors (for RNativeType elements).
/// Each boxed slice becomes an R vector.
impl<T> IntoR for Vec<Box<[T]>>
where
    T: crate::ffi::RNativeType,
{
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let list =
                crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::VECSXP, n as crate::ffi::R_xlen_t);
            crate::ffi::Rf_protect(list);

            for (i, boxed_slice) in self.into_iter().enumerate() {
                let vec: Vec<T> = boxed_slice.into_vec();
                let inner_sexp = vec.into_sexp();
                list.set_vector_elt(i as crate::ffi::R_xlen_t, inner_sexp);
            }

            crate::ffi::Rf_unprotect(1);
            list
        }
    }
}

/// Convert `Vec<Box<[String]>>` to R list of character vectors.
impl IntoR for Vec<Box<[String]>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let list =
                crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::VECSXP, n as crate::ffi::R_xlen_t);
            crate::ffi::Rf_protect(list);

            for (i, boxed_slice) in self.into_iter().enumerate() {
                let vec: Vec<String> = boxed_slice.into_vec();
                let inner_sexp = vec.into_sexp();
                list.set_vector_elt(i as crate::ffi::R_xlen_t, inner_sexp);
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
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
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
                list.set_vector_elt(i as crate::ffi::R_xlen_t, inner_sexp);
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
        let list = OwnedProtect::new(crate::ffi::Rf_allocVector(
            crate::ffi::SEXPTYPE::VECSXP,
            n as crate::ffi::R_xlen_t,
        ));
        for (i, item) in items.into_iter().enumerate() {
            list.get()
                .set_vector_elt(i as crate::ffi::R_xlen_t, item.into_sexp());
        }
        *list
    }
}

// region: Vec<Option<Collection>> conversions ──────────────────────────────────

/// Helper: convert `Vec<Option<C: IntoR>>` to a VECSXP, with `None` mapping to
/// `R_NilValue` (NULL) and `Some(v)` mapping to whatever `v.into_sexp()` produces.
fn vec_option_of_into_r_to_list<T: IntoR>(items: Vec<Option<T>>) -> crate::ffi::SEXP {
    unsafe {
        let n = items.len();
        let list = OwnedProtect::new(crate::ffi::Rf_allocVector(
            crate::ffi::SEXPTYPE::VECSXP,
            n as crate::ffi::R_xlen_t,
        ));
        for (i, item) in items.into_iter().enumerate() {
            let elt = match item {
                Some(v) => v.into_sexp(),
                None => crate::ffi::SEXP::nil(),
            };
            list.get().set_vector_elt(i as crate::ffi::R_xlen_t, elt);
        }
        *list
    }
}

/// Convert `Vec<Option<Vec<T>>>` to R list where `None` → NULL, `Some(v)` → typed vector.
impl<T: crate::ffi::RNativeType> IntoR for Vec<Option<Vec<T>>>
where
    Vec<T>: IntoR,
{
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        vec_option_of_into_r_to_list(self)
    }
}

/// Convert `Vec<Option<Vec<String>>>` to R list where `None` → NULL, `Some(v)` → character vector.
impl IntoR for Vec<Option<Vec<String>>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        vec_option_of_into_r_to_list(self)
    }
}

/// Convert `Vec<Option<HashSet<T>>>` to R list where `None` → NULL, `Some(s)` → unordered vector.
impl<T: crate::ffi::RNativeType + Eq + Hash> IntoR for Vec<Option<HashSet<T>>>
where
    HashSet<T>: IntoR,
{
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        vec_option_of_into_r_to_list(self)
    }
}

/// Convert `Vec<Option<HashSet<String>>>` to R list where `None` → NULL, `Some(s)` → character vector.
impl IntoR for Vec<Option<HashSet<String>>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        vec_option_of_into_r_to_list(self)
    }
}

/// Convert `Vec<Option<BTreeSet<T>>>` to R list where `None` → NULL, `Some(s)` → sorted vector.
impl<T: crate::ffi::RNativeType + Ord> IntoR for Vec<Option<BTreeSet<T>>>
where
    BTreeSet<T>: IntoR,
{
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        vec_option_of_into_r_to_list(self)
    }
}

/// Convert `Vec<Option<BTreeSet<String>>>` to R list where `None` → NULL, `Some(s)` → sorted character vector.
impl IntoR for Vec<Option<BTreeSet<String>>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        vec_option_of_into_r_to_list(self)
    }
}

/// Convert `Vec<Option<HashMap<String, V>>>` to R list where `None` → NULL, `Some(m)` → named list.
impl<V: IntoR> IntoR for Vec<Option<HashMap<String, V>>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        vec_option_of_into_r_to_list(self)
    }
}

/// Convert `Vec<Option<BTreeMap<String, V>>>` to R list where `None` → NULL, `Some(m)` → named list.
impl<V: IntoR> IntoR for Vec<Option<BTreeMap<String, V>>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        vec_option_of_into_r_to_list(self)
    }
}

// endregion

/// Convert `Vec<HashSet<T>>` to R list of vectors (for RNativeType elements).
/// Each HashSet becomes an R vector (unordered).
impl<T: crate::ffi::RNativeType> IntoR for Vec<std::collections::HashSet<T>>
where
    Vec<T>: IntoR,
{
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        let converted: Vec<Vec<T>> = self.into_iter().map(|s| s.into_iter().collect()).collect();
        vec_of_into_r_to_list(converted)
    }
}

/// Convert `Vec<BTreeSet<T>>` to R list of vectors (for RNativeType elements).
/// Each BTreeSet becomes an R vector (sorted).
impl<T: crate::ffi::RNativeType> IntoR for Vec<std::collections::BTreeSet<T>>
where
    Vec<T>: IntoR,
{
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        let converted: Vec<Vec<T>> = self.into_iter().map(|s| s.into_iter().collect()).collect();
        vec_of_into_r_to_list(converted)
    }
}

/// Convert `Vec<HashSet<String>>` to R list of character vectors.
impl IntoR for Vec<std::collections::HashSet<String>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        let converted: Vec<Vec<String>> =
            self.into_iter().map(|s| s.into_iter().collect()).collect();
        vec_of_into_r_to_list(converted)
    }
}

/// Convert `Vec<BTreeSet<String>>` to R list of character vectors.
impl IntoR for Vec<std::collections::BTreeSet<String>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
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
            type Error = std::convert::Infallible;
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                self.try_into_sexp()
            }
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
            list.set_vector_elt(i as crate::ffi::R_xlen_t, map.into_sexp());
        }

        crate::ffi::Rf_unprotect(1);
        list
    }
}
// endregion
