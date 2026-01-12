//! # C-Callable Function Loading
//!
//! This module loads the trait ABI C-callables from R's registered callable
//! table. These functions are registered by the base miniextendr R package
//! and provide the stable ABI surface for cross-package dispatch.
//!
//! ## C-Callables
//!
//! | Function | Purpose |
//! |----------|---------|
//! | `mx_wrap` | Wrap `*mut mx_erased` in R's EXTPTRSXP |
//! | `mx_get` | Extract `*mut mx_erased` from EXTPTRSXP |
//! | `mx_query` | Query for interface vtable by tag |
//!
//! ## Loading Mechanism
//!
//! C-callables are loaded via `R_GetCCallable("miniextendr", "mx_*")`.
//! This is R's standard mechanism for cross-package native function sharing.
//!
//! ## Initialization
//!
//! Must call [`init_ccallables`] from `R_init_<pkg>` before using any
//! trait ABI functions. This is required even for the base miniextendr
//! package itself.
//!
//! ```ignore
//! #[unsafe(no_mangle)]
//! pub extern "C" fn R_init_mypackage(info: *mut DllInfo) {
//!     miniextendr_worker_init();
//!     init_ccallables();  // Load C-callables
//!     // ...
//! }
//! ```
//!
//! ## Thread Safety
//!
//! - [`init_ccallables`] must be called from main thread
//! - All wrapper functions must be called from main thread
//! - Function pointers are stored in static `OnceLock` for thread-safe init

use crate::abi::{mx_erased, mx_tag};
use crate::ffi::SEXP;
use std::os::raw::c_void;
use std::sync::OnceLock;

// =============================================================================
// Function pointer types
// =============================================================================

/// Type for `mx_wrap`: wraps erased pointer in R external pointer.
type MxWrapFn = unsafe extern "C" fn(*mut mx_erased) -> SEXP;

/// Type for `mx_get`: extracts erased pointer from R external pointer.
type MxGetFn = unsafe extern "C" fn(SEXP) -> *mut mx_erased;

/// Type for `mx_query`: queries for interface vtable by tag.
type MxQueryFn = unsafe extern "C" fn(SEXP, mx_tag) -> *const c_void;

// =============================================================================
// Global function pointers (loaded once)
// =============================================================================

/// Loaded `mx_wrap` function pointer.
static P_MX_WRAP: OnceLock<MxWrapFn> = OnceLock::new();

/// Loaded `mx_get` function pointer.
static P_MX_GET: OnceLock<MxGetFn> = OnceLock::new();

/// Loaded `mx_query` function pointer.
static P_MX_QUERY: OnceLock<MxQueryFn> = OnceLock::new();

// =============================================================================
// Initialization
// =============================================================================

/// Initialize C-callable function pointers.
///
/// Loads `mx_wrap`, `mx_get`, and `mx_query` from R's callable table
/// via `R_GetCCallable("miniextendr", ...)`.
///
/// # Panics
///
/// Panics if:
///
/// - Called from a non-main thread
/// - Any C-callable is not found (miniextendr package not loaded)
/// - Called multiple times (function pointers are already set)
///
/// # Safety
///
/// This function is safe to call but has requirements:
///
/// - Must be called from R's main thread
/// - Must be called during `R_init_<pkg>` or after R is initialized
/// - The miniextendr R package must be loaded first
///
/// # Example
///
/// ```ignore
/// #[unsafe(no_mangle)]
/// pub extern "C" fn R_init_mypackage(info: *mut DllInfo) {
///     miniextendr_worker_init();
///     init_ccallables();
///     // Now mx_wrap, mx_get, mx_query are available
/// }
/// ```
pub fn init_ccallables() {
    // Check we're on main thread
    if !crate::worker::is_r_main_thread() {
        panic!("init_ccallables must be called from R's main thread");
    }

    // Load mx_wrap
    let wrap_ptr =
        unsafe { crate::ffi::R_GetCCallable(c"miniextendr".as_ptr(), c"mx_wrap".as_ptr()) };
    if wrap_ptr.is_none() {
        panic!("init_ccallables: mx_wrap not found - is miniextendr package loaded?");
    }
    let wrap_fn: MxWrapFn = unsafe { std::mem::transmute(wrap_ptr) };
    P_MX_WRAP
        .set(wrap_fn)
        .expect("init_ccallables called multiple times");

    // Load mx_get
    let get_ptr =
        unsafe { crate::ffi::R_GetCCallable(c"miniextendr".as_ptr(), c"mx_get".as_ptr()) };
    if get_ptr.is_none() {
        panic!("init_ccallables: mx_get not found - is miniextendr package loaded?");
    }
    let get_fn: MxGetFn = unsafe { std::mem::transmute(get_ptr) };
    P_MX_GET
        .set(get_fn)
        .expect("init_ccallables called multiple times");

    // Load mx_query
    let query_ptr =
        unsafe { crate::ffi::R_GetCCallable(c"miniextendr".as_ptr(), c"mx_query".as_ptr()) };
    if query_ptr.is_none() {
        panic!("init_ccallables: mx_query not found - is miniextendr package loaded?");
    }
    let query_fn: MxQueryFn = unsafe { std::mem::transmute(query_ptr) };
    P_MX_QUERY
        .set(query_fn)
        .expect("init_ccallables called multiple times");
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
/// - Must call [`init_ccallables`] first
///
/// # Panics
///
/// Panics if [`init_ccallables`] has not been called.
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
    let f = P_MX_WRAP
        .get()
        .expect("init_ccallables() must be called before mx_wrap()");
    // SAFETY: Caller guarantees ptr is valid and we're on main thread
    unsafe { f(ptr) }
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
/// - Must call [`init_ccallables`] first
/// - The returned pointer is only valid while R protects the SEXP
///
/// # Panics
///
/// Panics if [`init_ccallables`] has not been called.
#[inline]
pub unsafe fn mx_get(sexp: SEXP) -> *mut mx_erased {
    let f = P_MX_GET
        .get()
        .expect("init_ccallables() must be called before mx_get()");
    // SAFETY: Caller guarantees sexp is valid and we're on main thread
    unsafe { f(sexp) }
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
/// - Must call [`init_ccallables`] first
/// - The returned pointer must be cast to the correct vtable type
///
/// # Panics
///
/// Panics if [`init_ccallables`] has not been called.
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
    let f = P_MX_QUERY
        .get()
        .expect("init_ccallables() must be called before mx_query()");
    // SAFETY: Caller guarantees sexp is valid and we're on main thread
    unsafe { f(sexp, tag) }
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
/// - Must call [`init_ccallables`] first
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
