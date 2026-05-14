//! Extension trait for SEXP providing ALTREP-specific accessors.
//!
//! Using methods on SEXP (via `&self`) instead of free functions avoids
//! `clippy::not_unsafe_ptr_arg_deref` in ALTREP trait implementations,
//! mirroring the pattern established by [`SexpExt`](crate::ffi::SexpExt).

use crate::ffi::SEXP;

/// ALTREP-specific extension methods for SEXP.
///
/// These methods wrap the free functions in `ffi::altrep`, converting
/// `func(x)` calls to `x.method()` calls. This avoids the
/// `clippy::not_unsafe_ptr_arg_deref` lint in ALTREP trait method
/// implementations that receive SEXP as a parameter.
///
/// Only data2 (cache) accessors are exposed here; data1 (storage) is
/// accessed via the `AltrepExtract` trait or the standalone free functions
/// `altrep_data1_as` / `altrep_data1_as_unchecked` / `altrep_data1_mut` /
/// `altrep_data1_mut_unchecked`.
pub trait AltrepSexpExt {
    /// Get the raw SEXP in the ALTREP data2 slot.
    ///
    /// # Safety
    ///
    /// - `self` must be a valid ALTREP SEXP
    /// - Must be called from the R main thread
    unsafe fn altrep_data2_raw(&self) -> SEXP;

    /// Get the ALTREP data2 slot (unchecked — no thread routing).
    ///
    /// # Safety
    ///
    /// - `self` must be a valid ALTREP SEXP
    /// - Must be called from the R main thread
    unsafe fn altrep_data2_raw_unchecked(&self) -> SEXP;

    /// Set the ALTREP data2 slot.
    ///
    /// # Safety
    ///
    /// - `self` must be a valid ALTREP SEXP
    /// - Must be called from the R main thread
    unsafe fn set_altrep_data2(&self, data2: SEXP);

    /// Set the ALTREP data2 slot (unchecked — no thread routing).
    ///
    /// # Safety
    ///
    /// - `self` must be a valid ALTREP SEXP
    /// - Must be called from the R main thread
    unsafe fn set_altrep_data2_unchecked(&self, data2: SEXP);
}

impl AltrepSexpExt for SEXP {
    #[inline]
    unsafe fn altrep_data2_raw(&self) -> SEXP {
        unsafe { SEXP::altrep_data2_raw(*self) }
    }

    #[inline]
    unsafe fn altrep_data2_raw_unchecked(&self) -> SEXP {
        unsafe { SEXP::altrep_data2_raw_unchecked(*self) }
    }

    #[inline]
    unsafe fn set_altrep_data2(&self, data2: SEXP) {
        unsafe { SEXP::set_altrep_data2(*self, data2) }
    }

    #[inline]
    unsafe fn set_altrep_data2_unchecked(&self, data2: SEXP) {
        unsafe { SEXP::set_altrep_data2_unchecked(*self, data2) }
    }
}
