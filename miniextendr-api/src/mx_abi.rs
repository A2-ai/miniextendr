//! Rust implementation of the mx_abi C-callable functions.
//!
//! These functions are registered with `R_RegisterCCallable` during package init
//! and loaded by consumer packages via `R_GetCCallable`.
//!
//! This module replaces the C implementation that was previously in `mx_abi.c.in`.

use crate::abi::{mx_erased, mx_tag};
use crate::ffi::{
    R_ClearExternalPtr, R_ExternalPtrAddr, R_ExternalPtrTag, R_MakeExternalPtr, R_NilValue,
    R_PreserveObject, R_RegisterCCallable, R_RegisterCFinalizerEx, Rboolean, Rf_install,
    Rf_protect, Rf_unprotect, SEXP, SEXPTYPE, SexpExt,
};
use std::ffi::CStr;
use std::sync::OnceLock;

/// The tag symbol used to identify miniextendr external pointers.
/// SEXP is Send+Sync so OnceLock<SEXP> works directly.
static MX_TAG: OnceLock<SEXP> = OnceLock::new();

/// Get the miniextendr tag symbol, initializing it if needed.
///
/// # Safety
/// Must be called on R's main thread.
#[inline]
unsafe fn get_tag() -> SEXP {
    *MX_TAG.get_or_init(|| unsafe {
        let tag = Rf_install(c"miniextendr::mx_erased".as_ptr());
        R_PreserveObject(tag);
        tag
    })
}

/// Finalizer callback for R's garbage collector.
///
/// Called when an external pointer wrapping `mx_erased` is collected.
/// Invokes the object's drop function to clean up the Rust allocation.
unsafe extern "C-unwind" fn mx_externalptr_finalizer(ptr: SEXP) {
    unsafe {
        debug_assert_eq!(ptr.type_of(), crate::ffi::SEXPTYPE::EXTPTRSXP,);
        let erased = R_ExternalPtrAddr(ptr) as *mut mx_erased;
        if !erased.is_null() {
            let base = (*erased).base;
            if !base.is_null() {
                ((*base).drop)(erased);
            }
        }
        R_ClearExternalPtr(ptr);
    }
}

/// Wrap an erased object pointer in an R external pointer.
///
/// Registered as `"mx_wrap"` via `R_RegisterCCallable`.
///
/// # Safety
///
/// `ptr` must point to a valid `mx_erased` allocated by a miniextendr constructor.
/// Must be called on R's main thread.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn mx_wrap(ptr: *mut mx_erased) -> SEXP {
    unsafe {
        let tag = get_tag();
        let sexp = Rf_protect(R_MakeExternalPtr(ptr.cast(), tag, R_NilValue));
        R_RegisterCFinalizerEx(sexp, Some(mx_externalptr_finalizer), Rboolean::TRUE);
        Rf_unprotect(1);
        sexp
    }
}

/// Extract an erased object pointer from an R external pointer.
///
/// Returns null if the SEXP is not an external pointer or doesn't carry
/// the miniextendr tag.
///
/// Registered as `"mx_get"` via `R_RegisterCCallable`.
///
/// # Safety
///
/// `sexp` must be a valid SEXP. Must be called on R's main thread.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn mx_get(sexp: SEXP) -> *mut mx_erased {
    unsafe {
        if sexp.type_of() != SEXPTYPE::EXTPTRSXP {
            return std::ptr::null_mut();
        }
        if R_ExternalPtrTag(sexp) != get_tag() {
            return std::ptr::null_mut();
        }
        R_ExternalPtrAddr(sexp) as *mut mx_erased
    }
}

/// Query an object for an interface vtable by tag.
///
/// Returns the vtable pointer, or null if the type does not implement
/// the requested trait.
///
/// Registered as `"mx_query"` via `R_RegisterCCallable`.
///
/// # Safety
///
/// `sexp` must be a valid SEXP. Must be called on R's main thread.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn mx_query(sexp: SEXP, tag: mx_tag) -> *const std::ffi::c_void {
    unsafe {
        let erased = mx_get(sexp);
        if erased.is_null() {
            return std::ptr::null();
        }
        let base = (*erased).base;
        if base.is_null() {
            return std::ptr::null();
        }
        ((*base).query)(erased, tag)
    }
}

/// Register the mx_* C-callables with R.
///
/// Called during package init (`R_init_*`) to make `mx_wrap`, `mx_get`,
/// and `mx_query` available to consumer packages via `R_GetCCallable`.
///
/// # Safety
///
/// Must be called from R's main thread during package initialization.
/// `pkg_name` must be a valid null-terminated C string.
pub unsafe fn mx_abi_register(pkg_name: &CStr) {
    unsafe {
        // Initialize the tag symbol
        get_tag();

        // Register C-callables for cross-package access.
        // Cast function pointers to DL_FUNC (Option<unsafe extern "C-unwind" fn() -> *mut c_void>).
        R_RegisterCCallable(
            pkg_name.as_ptr(),
            c"mx_wrap".as_ptr(),
            Some(std::mem::transmute::<
                unsafe extern "C-unwind" fn(*mut mx_erased) -> SEXP,
                unsafe extern "C-unwind" fn() -> *mut std::os::raw::c_void,
            >(mx_wrap)),
        );
        R_RegisterCCallable(
            pkg_name.as_ptr(),
            c"mx_get".as_ptr(),
            Some(std::mem::transmute::<
                unsafe extern "C-unwind" fn(SEXP) -> *mut mx_erased,
                unsafe extern "C-unwind" fn() -> *mut std::os::raw::c_void,
            >(mx_get)),
        );
        R_RegisterCCallable(
            pkg_name.as_ptr(),
            c"mx_query".as_ptr(),
            Some(std::mem::transmute::<
                unsafe extern "C-unwind" fn(SEXP, mx_tag) -> *const std::ffi::c_void,
                unsafe extern "C-unwind" fn() -> *mut std::os::raw::c_void,
            >(mx_query)),
        );
    }
}
