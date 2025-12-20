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

/// Trait for converting Rust types to R SEXP values.
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

impl IntoR for i32 {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarInteger(self) }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarInteger_unchecked(self) }
    }
}

impl IntoR for f64 {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarReal(self) }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarReal_unchecked(self) }
    }
}

impl IntoR for u8 {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarRaw(self) }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarRaw_unchecked(self) }
    }
}

impl IntoR for bool {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarLogical(self as i32) }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarLogical_unchecked(self as i32) }
    }
}

impl IntoR for Option<bool> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::Rf_ScalarLogical(i32::MIN) },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::Rf_ScalarLogical_unchecked(i32::MIN) },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarLogical_unchecked(self as i32) }
    }
}

impl IntoR for crate::ffi::RLogical {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarLogical(self.to_i32()) }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarLogical_unchecked(self.to_i32()) }
    }
}

impl IntoR for Option<bool> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::Rf_ScalarLogical(i32::MIN) },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::Rf_ScalarLogical_unchecked(i32::MIN) },
        }
    }
}

impl<T: crate::externalptr::TypedExternal> IntoR for crate::externalptr::ExternalPtr<T> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_sexp()
    }
}

/// Helper to convert a string slice to R CHARSXP.
/// Uses UTF-8 encoding. Empty strings return R_BlankString equivalent.
#[inline]
fn str_to_charsxp(s: &str) -> crate::ffi::SEXP {
    unsafe {
        if s.is_empty() {
            // For empty string, still use mkCharLenCE with length 0
            crate::ffi::Rf_mkCharLenCE(s.as_ptr().cast(), 0, crate::ffi::CE_UTF8)
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
            crate::ffi::Rf_mkCharLenCE_unchecked(s.as_ptr().cast(), 0, crate::ffi::CE_UTF8)
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

/// Convert a slice to an R vector (checked).
#[inline]
unsafe fn vec_to_sexp<T: crate::ffi::RNativeType>(slice: &[T]) -> crate::ffi::SEXP {
    unsafe {
        let n = slice.len();
        let vec = crate::ffi::Rf_allocVector(T::SEXP_TYPE, n as crate::ffi::R_xlen_t);
        let ptr = crate::ffi::DATAPTR_RO(vec) as *mut T;
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
        let ptr = crate::ffi::DATAPTR_RO_unchecked(vec) as *mut T;
        std::ptr::copy_nonoverlapping(slice.as_ptr(), ptr, n);
        vec
    }
}

// =============================================================================
// Collection conversions (HashMap, BTreeMap, HashSet, BTreeSet)
// =============================================================================

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

/// Convert HashMap<String, V> to R named list (VECSXP).
impl<V: IntoR> IntoR for HashMap<String, V> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        map_to_named_list(self.into_iter())
    }
}

/// Convert BTreeMap<String, V> to R named list (VECSXP).
impl<V: IntoR> IntoR for BTreeMap<String, V> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        map_to_named_list(self.into_iter())
    }
}

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

/// Convert `HashSet<T>` to R vector.
impl<T> IntoR for HashSet<T>
where
    T: crate::ffi::RNativeType + Eq + Hash,
{
    fn into_sexp(self) -> crate::ffi::SEXP {
        let vec: Vec<T> = self.into_iter().collect();
        vec.into_sexp()
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
}

/// Convert `HashSet<String>` to R character vector.
impl IntoR for HashSet<String> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        let vec: Vec<String> = self.into_iter().collect();
        vec.into_sexp()
    }
}

/// Convert `BTreeSet<String>` to R character vector.
impl IntoR for BTreeSet<String> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        let vec: Vec<String> = self.into_iter().collect();
        vec.into_sexp()
    }
}

/// Convert `Vec<String>` to R character vector (STRSXP).
impl IntoR for Vec<String> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let vec =
                crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::STRSXP, n as crate::ffi::R_xlen_t);
            crate::ffi::Rf_protect(vec);

            for (i, s) in self.into_iter().enumerate() {
                let charsxp = crate::ffi::Rf_mkCharLenCE(
                    s.as_ptr().cast(),
                    s.len() as i32,
                    crate::ffi::CE_UTF8,
                );
                crate::ffi::SET_STRING_ELT(vec, i as crate::ffi::R_xlen_t, charsxp);
            }

            crate::ffi::Rf_unprotect(1);
            vec
        }
    }
}

/// Convert &[&str] to R character vector (STRSXP).
impl IntoR for &[&str] {
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let vec =
                crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::STRSXP, n as crate::ffi::R_xlen_t);
            crate::ffi::Rf_protect(vec);

            for (i, s) in self.iter().enumerate() {
                let charsxp = crate::ffi::Rf_mkCharLenCE(
                    s.as_ptr().cast(),
                    s.len() as i32,
                    crate::ffi::CE_UTF8,
                );
                crate::ffi::SET_STRING_ELT(vec, i as crate::ffi::R_xlen_t, charsxp);
            }

            crate::ffi::Rf_unprotect(1);
            vec
        }
    }
}

/// Convert `Vec<&str>` to R character vector (STRSXP).
impl IntoR for Vec<&str> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_slice().into_sexp()
    }
}

// =============================================================================
// NA-aware vector conversions
// =============================================================================

/// Convert `Vec<Option<f64>>` to R real vector with NA support.
///
/// `None` values become `NA_real_` (NaN) in R.
impl IntoR for Vec<Option<f64>> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let vec =
                crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::REALSXP, n as crate::ffi::R_xlen_t);
            crate::ffi::Rf_protect(vec);

            let ptr = crate::ffi::REAL(vec);
            for (i, val) in self.into_iter().enumerate() {
                // NA_real_ is represented as NaN in R
                *ptr.add(i) = val.unwrap_or(f64::NAN);
            }

            crate::ffi::Rf_unprotect(1);
            vec
        }
    }
}

/// Convert `Vec<Option<i32>>` to R integer vector with NA support.
///
/// `None` values become `NA_integer_` (i32::MIN) in R.
impl IntoR for Vec<Option<i32>> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let vec =
                crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::INTSXP, n as crate::ffi::R_xlen_t);
            crate::ffi::Rf_protect(vec);

            let ptr = crate::ffi::INTEGER(vec);
            for (i, val) in self.into_iter().enumerate() {
                // NA_integer_ is i32::MIN in R
                *ptr.add(i) = val.unwrap_or(i32::MIN);
            }

            crate::ffi::Rf_unprotect(1);
            vec
        }
    }
}

/// Convert `Vec<Option<String>>` to R character vector with NA support.
///
/// `None` values become `NA_character_` in R.
impl IntoR for Vec<Option<String>> {
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let n = self.len();
            let vec =
                crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::STRSXP, n as crate::ffi::R_xlen_t);
            crate::ffi::Rf_protect(vec);

            for (i, opt_s) in self.into_iter().enumerate() {
                let charsxp = match opt_s {
                    Some(s) => crate::ffi::Rf_mkCharLenCE(
                        s.as_ptr().cast(),
                        s.len() as i32,
                        crate::ffi::CE_UTF8,
                    ),
                    None => crate::ffi::R_NaString,
                };
                crate::ffi::SET_STRING_ELT(vec, i as crate::ffi::R_xlen_t, charsxp);
            }

            crate::ffi::Rf_unprotect(1);
            vec
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
// Rayon RVec conversion
// =============================================================================

#[cfg(feature = "rayon")]
impl<T> IntoR for crate::rayon_bridge::RVec<T>
where
    T: crate::ffi::RNativeType + Send,
{
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        // RVec was collected on Rayon threads, convert to R on main thread
        let vec = self.into_inner();
        crate::worker::with_r_thread(move || {
            crate::externalptr::SendableSexp::new(vec.into_sexp())
        })
        .into_inner()
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        // Still need to go through with_r_thread since we might be on a Rayon thread
        self.into_sexp()
    }
}
