//! ALTREP helpers for `ExternalPtr` — data1/data2 slot access.
//!
//! Convenience functions for ALTREP implementations that store their data
//! in `ExternalPtr` slots. Also provides the `Sidecar` marker type for
//! `#[r_data]` fields.

use super::{ErasedExternalPtr, ExternalPtr, TypedExternal};
use crate::altrep_ext::AltrepSexpExt;
use crate::ffi::SEXP;

/// Extract the ALTREP data1 slot as a typed `ExternalPtr<T>`.
///
/// This is a convenience function for ALTREP implementations that store
/// their data in an `ExternalPtr` in the data1 slot.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread
///
/// # Example
///
/// ```ignore
/// impl Altrep for MyAltrepClass {
///     const HAS_LENGTH: bool = true;
///     fn length(x: SEXP) -> R_xlen_t {
///         match unsafe { altrep_data1_as::<MyData>(x) } {
///             Some(ext) => ext.data.len() as R_xlen_t,
///             None => 0,
///         }
///     }
/// }
/// ```
#[inline]
pub unsafe fn altrep_data1_as<T: TypedExternal>(x: SEXP) -> Option<ExternalPtr<T>> {
    unsafe { ExternalPtr::wrap_sexp(x.altrep_data1_raw()) }
}

/// Extract the ALTREP data1 slot (unchecked version).
///
/// Skips thread safety checks for performance-critical ALTREP callbacks.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread (guaranteed in ALTREP callbacks)
#[inline]
pub unsafe fn altrep_data1_as_unchecked<T: TypedExternal>(x: SEXP) -> Option<ExternalPtr<T>> {
    unsafe { ExternalPtr::wrap_sexp_unchecked(x.altrep_data1_raw_unchecked()) }
}

/// Extract the ALTREP data2 slot as a typed `ExternalPtr<T>`.
///
/// Similar to `altrep_data1_as`, but for the data2 slot.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread
#[inline]
pub unsafe fn altrep_data2_as<T: TypedExternal>(x: SEXP) -> Option<ExternalPtr<T>> {
    unsafe { ExternalPtr::wrap_sexp(x.altrep_data2()) }
}

/// Extract the ALTREP data2 slot (unchecked version).
///
/// Skips thread safety checks for performance-critical ALTREP callbacks.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread (guaranteed in ALTREP callbacks)
#[inline]
pub unsafe fn altrep_data2_as_unchecked<T: TypedExternal>(x: SEXP) -> Option<ExternalPtr<T>> {
    unsafe { ExternalPtr::wrap_sexp_unchecked(x.altrep_data2_raw_unchecked()) }
}

/// Get a mutable reference to data in ALTREP data1 slot via `ErasedExternalPtr`.
///
/// This is useful for ALTREP methods that need to mutate the underlying data.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread
/// - The caller must ensure no other references to the data exist
///
/// # Example
///
/// ```ignore
/// fn dataptr(x: SEXP, _writable: bool) -> *mut c_void {
///     match unsafe { altrep_data1_mut::<MyData>(x) } {
///         Some(data) => data.buffer.as_mut_ptr().cast(),
///         None => core::ptr::null_mut(),
///     }
/// }
/// ```
#[inline]
pub unsafe fn altrep_data1_mut<T: TypedExternal>(x: SEXP) -> Option<&'static mut T> {
    unsafe {
        let mut erased = ErasedExternalPtr::from_sexp(x.altrep_data1_raw());
        // Transmute the lifetime to 'static - this is safe because:
        // 1. The ExternalPtr is protected by R's GC as part of the ALTREP object
        // 2. The ALTREP object `x` is kept alive by R during the callback
        erased.downcast_mut::<T>().map(|r| std::mem::transmute(r))
    }
}

/// Get a mutable reference to data in ALTREP data1 slot (unchecked version).
///
/// Skips thread safety checks for performance-critical ALTREP callbacks.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread (guaranteed in ALTREP callbacks)
/// - The caller must ensure no other references to the data exist
#[inline]
pub unsafe fn altrep_data1_mut_unchecked<T: TypedExternal>(x: SEXP) -> Option<&'static mut T> {
    unsafe {
        let mut erased = ErasedExternalPtr::from_sexp(x.altrep_data1_raw_unchecked());
        erased.downcast_mut::<T>().map(|r| std::mem::transmute(r))
    }
}

// Tests for ExternalPtr require R runtime, so they are in rpkg/src/rust/lib.rs
// endregion

// region: Sidecar Marker Type for #[r_data] Fields

/// Marker type for enabling R sidecar accessors in an `ExternalPtr` struct.
///
/// When used with `#[derive(ExternalPtr)]` and `#[r_data]`, this field acts as
/// a selector that enables R-facing accessors for sibling `#[r_data]` fields.
///
/// # Supported Field Types
///
/// - **`SEXP`** - Raw SEXP access, no conversion
/// - **`i32`, `f64`, `bool`, `u8`** - Zero-overhead scalars (stored directly in R)
/// - **Any `IntoR` type** - Automatic conversion (e.g., `String`, `Vec<T>`)
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::ffi::SEXP;
///
/// #[derive(ExternalPtr)]
/// pub struct MyType {
///     pub x: i32,
///
///     /// Selector field - enables R wrapper generation
///     #[r_data]
///     r: RSidecar,
///
///     /// Raw SEXP slot - MyType_get_raw() / MyType_set_raw()
///     #[r_data]
///     pub raw: SEXP,
///
///     /// Zero-overhead scalar - MyType_get_count() / MyType_set_count()
///     #[r_data]
///     pub count: i32,
///
///     /// Conversion type - MyType_get_name() / MyType_set_name()
///     #[r_data]
///     pub name: String,
/// }
/// ```
///
/// # Design Notes
///
/// - `RSidecar` is a ZST (zero-sized type) - no runtime cost
/// - Only `pub` `#[r_data]` fields get R wrapper functions generated
/// - Multiple `RSidecar` fields in one struct is a compile error
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct RSidecar;
// endregion
