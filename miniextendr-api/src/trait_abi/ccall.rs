//! # Direct FFI to mx_abi.c Functions
//!
//! This module provides Rust wrappers around the C functions defined in
//! `mx_abi.c`, which is compiled into each package's shared object.
//!
//! ## C Functions
//!
//! | Function | Purpose |
//! |----------|---------|
//! | `mx_wrap` | Wrap `*mut mx_erased` in R's EXTPTRSXP |
//! | `mx_get` | Extract `*mut mx_erased` from EXTPTRSXP |
//! | `mx_query` | Query for interface vtable by tag |
//!
//! ## Linkage
//!
//! Each package compiles `mx_abi.c` into its own `.so`, so these symbols are
//! resolved at link time — no `R_GetCCallable` indirection needed.
//!
//! ## Thread Safety
//!
//! All wrapper functions must be called from R's main thread.

use crate::abi::{mx_erased, mx_tag};
use crate::ffi::SEXP;
use std::os::raw::c_void;

// =============================================================================
// Direct extern declarations to mx_abi.c functions (same .so)
// =============================================================================

mod ffi {
    use super::*;

    unsafe extern "C" {
        pub(super) fn mx_wrap(ptr: *mut mx_erased) -> SEXP;
        pub(super) fn mx_get(sexp: SEXP) -> *mut mx_erased;
        pub(super) fn mx_query(sexp: SEXP, tag: mx_tag) -> *const c_void;
    }
}

// =============================================================================
// Wrapper functions
// =============================================================================

/// Wrap an erased object pointer in an R external pointer.
///
/// Creates an R `EXTPTRSXP` that wraps the given erased object. The external
/// pointer's finalizer will call the object's `drop` function when garbage
/// collected.
///
/// # Arguments
///
/// * `ptr` - Pointer to erased object (must be heap-allocated)
///
/// # Returns
///
/// R external pointer (`EXTPTRSXP`) containing the erased object.
///
/// # Safety
///
/// - `ptr` must be a valid pointer to `mx_erased`
/// - `ptr` must be heap-allocated (will be freed by finalizer)
/// - Must be called on R's main thread
/// - `mx_abi_register()` must have been called (from entrypoint.c)
///
/// # Example
///
/// ```ignore
/// // In constructor
/// let obj = Box::into_raw(Box::new(MyErasedWrapper::new(data)));
/// let sexp = unsafe { mx_wrap(obj as *mut mx_erased) };
/// ```
#[inline]
pub unsafe fn mx_wrap(ptr: *mut mx_erased) -> SEXP {
    // SAFETY: Caller guarantees ptr is valid and we're on main thread.
    // mx_wrap is linked from mx_abi.c in the same .so.
    unsafe { ffi::mx_wrap(ptr) }
}

/// Extract an erased object pointer from an R external pointer.
///
/// Retrieves the `*mut mx_erased` stored in an R `EXTPTRSXP`.
///
/// # Arguments
///
/// * `sexp` - R external pointer created by [`mx_wrap`]
///
/// # Returns
///
/// Pointer to the erased object, or null if:
/// - `sexp` is not an external pointer
/// - The external pointer has been invalidated
///
/// # Safety
///
/// - `sexp` must be a valid SEXP
/// - Must be called on R's main thread
/// - The returned pointer is only valid while R protects the SEXP
#[inline]
pub unsafe fn mx_get(sexp: SEXP) -> *mut mx_erased {
    // SAFETY: Caller guarantees sexp is valid and we're on main thread.
    unsafe { ffi::mx_get(sexp) }
}

/// Query an object for an interface vtable by tag.
///
/// Looks up whether the object implements the trait identified by `tag`,
/// and returns a pointer to the vtable if so.
///
/// # Arguments
///
/// * `sexp` - R external pointer wrapping an erased object
/// * `tag` - Tag identifying the requested trait interface
///
/// # Returns
///
/// - Non-null pointer to the trait's vtable if implemented
/// - Null pointer if:
///   - `sexp` is not a valid erased object
///   - The object does not implement the requested trait
///
/// # Safety
///
/// - `sexp` must be a valid SEXP
/// - Must be called on R's main thread
/// - The returned pointer must be cast to the correct vtable type
///
/// # Example
///
/// ```ignore
/// let vtable = unsafe { mx_query(obj, TAG_FOO) };
/// if !vtable.is_null() {
///     let foo_vtable = vtable as *const FooVTable;
///     // Call method through vtable...
/// }
/// ```
#[inline]
pub unsafe fn mx_query(sexp: SEXP, tag: mx_tag) -> *const c_void {
    // SAFETY: Caller guarantees sexp is valid and we're on main thread.
    unsafe { ffi::mx_query(sexp, tag) }
}

/// Query an object for an interface and return a typed view.
///
/// Convenience wrapper around [`mx_query`] that returns an `Option<&V>`
/// where `V` is the view type for the trait.
///
/// # Type Parameters
///
/// * `V` - The view type (e.g., `FooView`) containing data pointer and vtable
///
/// # Arguments
///
/// * `sexp` - R external pointer wrapping an erased object
/// * `tag` - Tag identifying the requested trait interface
///
/// # Returns
///
/// - `Some(&V)` if the object implements the trait
/// - `None` if the object does not implement the trait
///
/// # Safety
///
/// - `sexp` must be a valid SEXP
/// - `V` must be the correct view type for `tag`
/// - Must be called on R's main thread
///
/// # Example
///
/// ```ignore
/// if let Some(view) = unsafe { mx_query_as::<FooView>(obj, TAG_FOO) } {
///     let result = view.some_method(args);
/// } else {
///     r_stop("object does not implement Foo");
/// }
/// ```
#[inline]
pub unsafe fn mx_query_as<V>(sexp: SEXP, tag: mx_tag) -> Option<&'static V> {
    let vtable = unsafe { mx_query(sexp, tag) };
    if vtable.is_null() {
        None
    } else {
        Some(unsafe { &*(vtable as *const V) })
    }
}
