//! Extension trait for SEXP providing ALTREP-specific accessors.
//!
//! Using methods on SEXP (via `&self`) instead of free functions avoids
//! `clippy::not_unsafe_ptr_arg_deref` in ALTREP trait implementations,
//! mirroring the pattern established by [`SexpExt`](crate::ffi::SexpExt).

use crate::externalptr::{ExternalPtr, TypedExternal};
use crate::ffi::SEXP;

/// ALTREP-specific extension methods for SEXP.
///
/// These methods wrap the free functions in `externalptr::altrep_helpers`
/// and `ffi::altrep`, converting `func(x)` calls to `x.method()` calls.
/// This avoids the `clippy::not_unsafe_ptr_arg_deref` lint in ALTREP trait
/// method implementations that receive SEXP as a parameter.
pub trait AltrepSexpExt {
    /// Extract the ALTREP data1 slot as a typed `ExternalPtr<T>`.
    ///
    /// # Safety
    ///
    /// - `self` must be a valid ALTREP SEXP
    /// - Must be called from the R main thread
    unsafe fn altrep_data1<T: TypedExternal>(&self) -> Option<ExternalPtr<T>>;

    /// Get a mutable reference to data in the ALTREP data1 slot.
    ///
    /// # Safety
    ///
    /// - `self` must be a valid ALTREP SEXP
    /// - Must be called from the R main thread
    /// - The caller must ensure no other references to the data exist
    unsafe fn altrep_data1_mut_ref<T: TypedExternal>(&self) -> Option<&'static mut T>;

    /// Get the raw SEXP in the ALTREP data1 slot.
    ///
    /// # Safety
    ///
    /// - `self` must be a valid ALTREP SEXP
    /// - Must be called from the R main thread
    unsafe fn altrep_data1_raw(&self) -> SEXP;

    /// Get the raw SEXP in the ALTREP data1 slot (unchecked — no thread routing).
    ///
    /// # Safety
    ///
    /// - `self` must be a valid ALTREP SEXP
    /// - Must be called from the R main thread
    unsafe fn altrep_data1_raw_unchecked(&self) -> SEXP;

    /// Set the ALTREP data1 slot.
    ///
    /// # Safety
    ///
    /// - `self` must be a valid ALTREP SEXP
    /// - Must be called from the R main thread
    unsafe fn set_altrep_data1(&self, data1: SEXP);

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
    unsafe fn altrep_data1<T: TypedExternal>(&self) -> Option<ExternalPtr<T>> {
        unsafe { crate::altrep_data1_as::<T>(*self) }
    }

    #[inline]
    unsafe fn altrep_data1_mut_ref<T: TypedExternal>(&self) -> Option<&'static mut T> {
        unsafe { crate::altrep_data1_mut::<T>(*self) }
    }

    #[inline]
    unsafe fn altrep_data1_raw(&self) -> SEXP {
        unsafe { SEXP::altrep_data1_raw(*self) }
    }

    #[inline]
    unsafe fn altrep_data1_raw_unchecked(&self) -> SEXP {
        unsafe { SEXP::altrep_data1_raw_unchecked(*self) }
    }

    #[inline]
    unsafe fn set_altrep_data1(&self, data1: SEXP) {
        unsafe { SEXP::set_altrep_data1(*self, data1) }
    }

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
