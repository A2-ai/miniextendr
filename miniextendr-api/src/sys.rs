//! Raw FFI bindings to R headers.
//!
//! This module mirrors R's C API closely and is intentionally thin. **You
//! almost never call these directly from user code** — prefer the
//! higher-level wrappers: `SEXP` (in [`crate::sexp`]), [`SexpExt`] (in
//! [`crate::sexp_ext`]), the type vocabulary in [`crate::sexp_types`], plus
//! [`IntoR`], [`TryFromSexp`], [`with_r_thread`], [`with_r_unwind_protect`],
//! and the `#[miniextendr]` proc-macro. The items here exist so those
//! wrappers can be written; treat them as the framework's escape hatch.
//!
//! [`SexpExt`]: crate::sexp_ext::SexpExt
//! [`IntoR`]: crate::IntoR
//! [`TryFromSexp`]: crate::TryFromSexp
//! [`with_r_thread`]: crate::worker::with_r_thread
//! [`with_r_unwind_protect`]: crate::unwind_protect::with_r_unwind_protect
//!
//! # Checked vs `*_unchecked` variants
//!
//! Most non-variadic R API entry points come in two forms thanks to the
//! [`#[r_ffi_checked]`](miniextendr_macros::r_ffi_checked) proc-macro applied
//! to the `unsafe extern "C-unwind"` blocks below:
//!
//! - **Checked** (default — e.g. `Rf_allocVector`, `Rf_protect`, `INTEGER`):
//!   debug-asserts you're on R's main thread, routing through
//!   [`crate::worker::with_r_thread`] when called from a worker thread. **Use
//!   these by default.**
//! - **`*_unchecked`** (e.g. `Rf_allocVector_unchecked`): bypass the assertion
//!   and the worker round-trip. Calling one off the R main thread is
//!   undefined behaviour. They exist for three known-safe contexts:
//!     1. Inside ALTREP callbacks — R is already calling us on the main thread.
//!     2. Inside a [`crate::unwind_protect::with_r_unwind_protect`] body —
//!        the guard has already established main-thread context.
//!     3. Inside a [`crate::worker::with_r_thread`] body — the check would be
//!        redundant.
//!
//! The build-time lint **MXL301** enforces this: any `*_unchecked` call
//! outside those contexts is a compile-time error.
//!
//! # Don't raise R errors directly
//!
//! `Rf_error`, `Rf_errorcall`, and their `_unchecked` siblings longjmp,
//! which **skips Rust destructors** and leaks resources. The lint **MXL300**
//! forbids them in user code. Use `panic!()` instead; the framework converts
//! the panic into a structured R condition with `rust_*` class layering via
//! the tagged-SEXP transport (see [`crate::error_value`]).
//!
//! # Cross references
//!
//! - [`crate::ffi_guard`] — guard taxonomy and worker-thread invariants.
//! - [`crate::thread`] / [`crate::worker`] — worker / main-thread split.
//! - [`crate::altrep_traits`] / [`crate::altrep_bridge`] — guard modes
//!   inside ALTREP callbacks.
//! - [`crate::error_value`] / [`mod@crate::condition`] — panic → R condition
//!   transport.

/// Raw ALTREP C API method type aliases.
pub mod altrep;

// `extern "C-unwind"` signatures below reference the type vocabulary by
// bare name. Bring it into scope as a private `use` (NOT `pub use`) — the
// vocab's canonical home is the crate root (`crate::SEXP`, etc.) and
// `crate::sexp_types` for the niche helpers. Adding these to the public
// API of `sys::` would re-create the bridge.
use crate::sexp::SEXP;
use crate::sexp_types::{R_CFinalizer_t, R_xlen_t, Rboolean, Rbyte, Rcomplex, SEXPTYPE, cetype_t};

// region: Connections types (gated behind `connections` feature)
// WARNING: R's connection API is explicitly marked as UNSTABLE.

/// Opaque R connection implementation (from R_ext/Connections.h).
///
/// This is an opaque type representing R's internal connection structure.
/// The actual structure is explicitly unstable and may change between R versions.
#[cfg(feature = "connections")]
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct Rconnection_impl(::std::os::raw::c_void);

/// Pointer to an R connection handle.
///
/// This is the typed equivalent of R's `Rconnection` type, which is a pointer
/// to the opaque `Rconn` struct. Using this instead of `*mut c_void` provides
/// type safety for connection APIs.
#[cfg(feature = "connections")]
#[allow(non_camel_case_types)]
pub type Rconnection = *mut Rconnection_impl;

/// R connections API version from R's `R_ext/Connections.h` at compile time.
///
/// This is a compile-time constant baked into the Rust FFI bindings when they
/// were generated against a particular R version's headers. It does **not**
/// dynamically probe the running R session.
///
/// From R_ext/Connections.h: "you *must* check the version and proceed only
/// if it matches what you expect. We explicitly reserve the right to change
/// the connection implementation without a compatibility layer."
///
/// Before using any connection APIs, check that this equals the expected version (1).
#[cfg(feature = "connections")]
#[allow(non_upper_case_globals)]
pub const R_CONNECTIONS_VERSION: ::std::os::raw::c_int = 1;

// endregion

use miniextendr_macros::r_ffi_checked;

// Unchecked variadic functions (internal use only, no thread check)
#[allow(clashing_extern_declarations)]
#[allow(varargs_without_pattern)]
unsafe extern "C-unwind" {
    /// Unchecked variadic `Rf_error`; call checked wrapper when possible.
    #[link_name = "Rf_error"]
    pub fn Rf_error_unchecked(arg1: *const ::std::os::raw::c_char, ...) -> !;
    /// Unchecked variadic `Rf_errorcall`; call checked wrapper when possible.
    #[link_name = "Rf_errorcall"]
    pub fn Rf_errorcall_unchecked(arg1: SEXP, arg2: *const ::std::os::raw::c_char, ...) -> !;
    /// Unchecked variadic `Rf_warning`; call checked wrapper when possible.
    #[link_name = "Rf_warning"]
    pub fn Rf_warning_unchecked(arg1: *const ::std::os::raw::c_char, ...);
    /// Unchecked variadic `Rprintf`; call checked wrapper when possible.
    #[link_name = "Rprintf"]
    pub fn Rprintf_unchecked(arg1: *const ::std::os::raw::c_char, ...);
    /// Unchecked variadic `REprintf`; call checked wrapper when possible.
    #[link_name = "REprintf"]
    pub fn REprintf_unchecked(arg1: *const ::std::os::raw::c_char, ...);
}

// Error message access (non-API, declared in Rinternals.h but flagged by R CMD check)
#[cfg(feature = "nonapi")]
unsafe extern "C-unwind" {
    /// Get the current R error message buffer.
    ///
    /// Returns a pointer to R's internal error message buffer.
    /// Used by Rserve and other embedding applications.
    ///
    /// # Safety
    ///
    /// - The returned pointer is only valid until the next R error
    /// - Must not be modified
    /// - Should be copied if needed beyond the immediate scope
    ///
    /// # Feature Gate
    ///
    /// This is a non-API function and requires the `nonapi` feature.
    #[allow(non_snake_case, dead_code)] // used by worker.rs under worker-thread feature
    pub(crate) fn R_curErrorBuf() -> *const ::std::os::raw::c_char;
}

// Console hooks (non-API; declared in Rinterface.h)
#[cfg(feature = "nonapi")]
unsafe extern "C-unwind" {
    #[expect(dead_code, reason = "declared for future use")]
    pub(crate) static ptr_R_WriteConsoleEx: Option<
        unsafe extern "C-unwind" fn(
            *const ::std::os::raw::c_char,
            ::std::os::raw::c_int,
            ::std::os::raw::c_int,
        ),
    >;
}

/// Checked wrapper for `Rf_error` - panics if called from non-main thread.
/// Common usage: `Rf_error(c"%s".as_ptr(), message.as_ptr())`
///
/// # Safety
///
/// - Must be called from the R main thread
/// - `fmt` and `arg1` must be valid null-terminated C strings
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn Rf_error(
    fmt: *const ::std::os::raw::c_char,
    arg1: *const ::std::os::raw::c_char,
) -> ! {
    if !crate::worker::is_r_main_thread() {
        panic!("Rf_error called from non-main thread");
    }
    unsafe { Rf_error_unchecked(fmt, arg1) }
}

/// Checked wrapper for `Rf_errorcall` - panics if called from non-main thread.
///
/// # Safety
///
/// - Must be called from the R main thread
/// - `call` must be a valid SEXP or R_NilValue
/// - `fmt` and `arg1` must be valid null-terminated C strings
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn Rf_errorcall(
    call: SEXP,
    fmt: *const ::std::os::raw::c_char,
    arg1: *const ::std::os::raw::c_char,
) -> ! {
    if !crate::worker::is_r_main_thread() {
        panic!("Rf_errorcall called from non-main thread");
    }
    unsafe { Rf_errorcall_unchecked(call, fmt, arg1) }
}

/// Checked wrapper for `Rf_warning` - panics if called from non-main thread.
///
/// # Safety
///
/// - Must be called from the R main thread
/// - `fmt` and `arg1` must be valid null-terminated C strings
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn Rf_warning(fmt: *const ::std::os::raw::c_char, arg1: *const ::std::os::raw::c_char) {
    if !crate::worker::is_r_main_thread() {
        panic!("Rf_warning called from non-main thread");
    }
    unsafe { Rf_warning_unchecked(fmt, arg1) }
}

/// Checked wrapper for `Rprintf` - panics if called from non-main thread.
///
/// # Safety
///
/// - Must be called from the R main thread
/// - `fmt` and `arg1` must be valid null-terminated C strings
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn Rprintf(fmt: *const ::std::os::raw::c_char, arg1: *const ::std::os::raw::c_char) {
    if !crate::worker::is_r_main_thread() {
        panic!("Rprintf called from non-main thread");
    }
    unsafe { Rprintf_unchecked(fmt, arg1) }
}

/// Print to R's stderr (via R_ShowMessage or error console).
///
/// # Safety
///
/// - Must be called from the R main thread
/// - `fmt` and `arg1` must be valid null-terminated C strings
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn REprintf(fmt: *const ::std::os::raw::c_char, arg1: *const ::std::os::raw::c_char) {
    if !crate::worker::is_r_main_thread() {
        panic!("REprintf called from non-main thread");
    }
    unsafe { REprintf_unchecked(fmt, arg1) }
}

// Imported R symbols and functions with runtime thread checks enabled.
#[allow(missing_docs)]
#[r_ffi_checked]
#[allow(clashing_extern_declarations)]
unsafe extern "C-unwind" {
    /// The canonical R `NULL` value.
    pub static R_NilValue: SEXP;

    #[doc(alias = "NA_STRING")]
    /// Missing string singleton — encapsulated by SEXP::na_string()
    pub static R_NaString: SEXP;
    /// Empty string CHARSXP — encapsulated by SEXP::blank_string()
    pub static R_BlankString: SEXP;
    /// Symbol for `names` attribute.
    // Attribute symbols — encapsulated by SexpExt methods and SEXP::*_symbol()
    pub static R_NamesSymbol: SEXP;
    pub static R_DimSymbol: SEXP;
    pub static R_DimNamesSymbol: SEXP;
    pub static R_ClassSymbol: SEXP;
    pub static R_RowNamesSymbol: SEXP;
    pub static R_LevelsSymbol: SEXP;
    pub static R_TspSymbol: SEXP;

    /// Global environment (`.GlobalEnv`).
    pub static R_GlobalEnv: SEXP;
    /// Base package namespace environment.
    pub static R_BaseEnv: SEXP;
    /// Empty root environment.
    pub static R_EmptyEnv: SEXP;
    /// Base package namespace — encapsulated by SEXP::base_namespace()
    pub static R_BaseNamespace: SEXP;

    /// The "missing argument" sentinel value.
    ///
    /// When an R function is called without providing a value for a formal
    /// argument, R passes `R_MissingArg` as a placeholder. This is different
    /// from `R_NilValue` (NULL) - a missing argument means "not provided",
    /// while NULL is an explicit value.
    ///
    /// In R: `f <- function(x) missing(x); f()` returns `TRUE`.
    /// Encapsulated by SEXP::missing_arg()
    pub static R_MissingArg: SEXP;

    // Issue #112 cat. 10: kept pub(crate) — single-caller utilities; wrapping adds no value
    // Rinterface.h
    pub(crate) fn R_FlushConsole();

    // Special logical values (from internal Defn.h, not public API)
    // These are gated behind `nonapi` feature as they may change across R versions.
    #[cfg(feature = "nonapi")]
    /// Non-API TRUE singleton.
    pub static R_TrueValue: SEXP;
    #[cfg(feature = "nonapi")]
    /// Non-API FALSE singleton.
    pub static R_FalseValue: SEXP;
    #[cfg(feature = "nonapi")]
    /// Non-API NA logical singleton.
    pub static R_LogicalNAValue: SEXP;

    // Rinternals.h
    #[doc(alias = "mkChar")]
    pub fn Rf_mkChar(s: *const ::std::os::raw::c_char) -> SEXP;
    #[doc(alias = "mkCharLen")]
    pub fn Rf_mkCharLen(s: *const ::std::os::raw::c_char, len: i32) -> SEXP;
    #[doc(alias = "mkCharLenCE")]
    pub fn Rf_mkCharLenCE(
        x: *const ::std::os::raw::c_char,
        len: ::std::os::raw::c_int,
        ce: cetype_t,
    ) -> SEXP;
    #[doc(alias = "xlength")]
    #[doc(alias = "XLENGTH")]
    pub fn Rf_xlength(x: SEXP) -> R_xlen_t;
    #[doc(alias = "translateCharUTF8")]
    pub fn Rf_translateCharUTF8(x: SEXP) -> *const ::std::os::raw::c_char;
    #[doc(alias = "getCharCE")]
    pub fn Rf_getCharCE(x: SEXP) -> cetype_t;
    #[doc(alias = "charIsASCII")]
    pub fn Rf_charIsASCII(x: SEXP) -> Rboolean;
    #[doc(alias = "charIsUTF8")]
    pub fn Rf_charIsUTF8(x: SEXP) -> Rboolean;
    #[doc(alias = "charIsLatin1")]
    pub fn Rf_charIsLatin1(x: SEXP) -> Rboolean;

    // Issue #112 cat. 3: kept pub(crate) — only called from unwind_protect.rs; users go through with_r_unwind_protect
    pub(crate) fn R_MakeUnwindCont() -> SEXP;
    pub(crate) fn R_ContinueUnwind(cont: SEXP) -> !;
    pub(crate) fn R_UnwindProtect(
        fun: ::std::option::Option<
            unsafe extern "C-unwind" fn(*mut ::std::os::raw::c_void) -> SEXP,
        >,
        fun_data: *mut ::std::os::raw::c_void,
        cleanfun: ::std::option::Option<
            unsafe extern "C-unwind" fn(*mut ::std::os::raw::c_void, Rboolean),
        >,
        cleanfun_data: *mut ::std::os::raw::c_void,
        cont: SEXP,
    ) -> SEXP;

    /// Version of `R_UnwindProtect` that accepts `extern "C-unwind"` function pointers
    #[link_name = "R_UnwindProtect"]
    pub(crate) fn R_UnwindProtect_C_unwind(
        fun: ::std::option::Option<
            unsafe extern "C-unwind" fn(*mut ::std::os::raw::c_void) -> SEXP,
        >,
        fun_data: *mut ::std::os::raw::c_void,
        cleanfun: ::std::option::Option<
            unsafe extern "C-unwind" fn(*mut ::std::os::raw::c_void, Rboolean),
        >,
        cleanfun_data: *mut ::std::os::raw::c_void,
        cont: SEXP,
    ) -> SEXP;

    // Rinternals.h
    // Issue #112 cat. 2: kept pub(crate) — ExternalPtr<T> encapsulates these for users; raw access needed within externalptr.rs
    #[doc = " External pointer interface"]
    pub(crate) fn R_MakeExternalPtr(p: *mut ::std::os::raw::c_void, tag: SEXP, prot: SEXP) -> SEXP;
    pub fn R_ExternalPtrAddr(s: SEXP) -> *mut ::std::os::raw::c_void;
    pub(crate) fn R_ExternalPtrTag(s: SEXP) -> SEXP;
    pub(crate) fn R_ExternalPtrProtected(s: SEXP) -> SEXP;
    pub(crate) fn R_ClearExternalPtr(s: SEXP);
    pub(crate) fn R_SetExternalPtrAddr(s: SEXP, p: *mut ::std::os::raw::c_void);
    pub(crate) fn R_SetExternalPtrTag(s: SEXP, tag: SEXP);
    pub(crate) fn R_SetExternalPtrProtected(s: SEXP, p: SEXP);
    #[doc = " Added in R 3.4.0"]
    pub fn R_MakeExternalPtrFn(p: DL_FUNC, tag: SEXP, prot: SEXP) -> SEXP;
    pub fn R_ExternalPtrAddrFn(s: SEXP) -> DL_FUNC;
    pub fn R_RegisterFinalizer(s: SEXP, fun: SEXP);
    pub(crate) fn R_RegisterCFinalizer(s: SEXP, fun: R_CFinalizer_t);
    pub fn R_RegisterFinalizerEx(s: SEXP, fun: SEXP, onexit: Rboolean);
    pub(crate) fn R_RegisterCFinalizerEx(s: SEXP, fun: R_CFinalizer_t, onexit: Rboolean);

    // R_ext/Rdynload.h - C-callable interface
    // Issue #112 cat. 10: kept pub(crate) — cross-package ABI helpers used from mx_abi.rs; wrapping adds no value
    /// Register a C-callable function for cross-package access.
    pub(crate) fn R_RegisterCCallable(
        package: *const ::std::os::raw::c_char,
        name: *const ::std::os::raw::c_char,
        fptr: DL_FUNC,
    );
    /// Get a C-callable function from another package.
    pub(crate) fn R_GetCCallable(
        package: *const ::std::os::raw::c_char,
        name: *const ::std::os::raw::c_char,
    ) -> DL_FUNC;

    // region: GC protection
    //
    // R has two GC protection mechanisms with very different cost profiles:
    //
    // ## Protect stack (`Rf_protect` / `Rf_unprotect`)
    //
    // A pre-allocated array (`R_PPStack`) with an integer index (`R_PPStackTop`).
    // Protect pushes: `R_PPStack[R_PPStackTop++] = s`.
    // Unprotect pops: `R_PPStackTop -= n`.
    // **No heap allocation. No GC pressure. Essentially a single memory write.**
    // Use this for temporary protection within a function.
    // Requires LIFO discipline — nested scopes are fine, interleaved are not.
    //
    // ## Precious list (`R_PreserveObject` / `R_ReleaseObject`)
    //
    // A global linked list of CONSXP cells (`R_PreciousList`).
    // Preserve: `CONS(object, R_PreciousList)` — **allocates a cons cell every call**.
    // Release: linear scan of the entire list to find and unlink the object — **O(n)**.
    // (Optional `R_HASH_PRECIOUS` env var enables a 1069-bucket hash table, improving
    // Release to O(bucket_size), but Preserve still allocates.)
    // Use this only for long-lived objects that outlive any single protect scope.
    //
    // ## Cost summary
    //
    // | Operation            | Cost               | Allocates? |
    // |----------------------|--------------------|------------|
    // | `Rf_protect`         | array write        | no         |
    // | `Rf_unprotect(n)`    | integer subtract   | no         |
    // | `Rf_unprotect_ptr`   | scan + shift       | no         |
    // | `R_PreserveObject`   | cons cell alloc    | **yes**    |
    // | `R_ReleaseObject`    | linked list scan   | no (O(n))  |
    // | `R_ProtectWithIndex` | array write + save | no         |
    // | `R_Reprotect`        | array index write  | no         |

    /// Add a SEXP to the protect stack, preventing GC collection.
    ///
    /// **Cost: O(1)** — single array write (`R_PPStack[top++] = s`). No allocation.
    ///
    /// Must be balanced by a corresponding `Rf_unprotect`. The protect stack is
    /// LIFO — nested scopes are safe, but interleaved usage from different scopes
    /// will cause incorrect unprotection.
    #[doc(alias = "PROTECT")]
    #[doc(alias = "protect")]
    pub fn Rf_protect(s: SEXP) -> SEXP;

    /// Pop the top `l` entries from the protect stack.
    ///
    /// **Cost: O(1)** — single integer subtract (`R_PPStackTop -= l`). No allocation.
    ///
    /// The popped SEXPs become eligible for GC. Must match the number of
    /// `Rf_protect` calls in the current scope (LIFO order).
    #[doc(alias = "UNPROTECT")]
    #[doc(alias = "unprotect")]
    pub fn Rf_unprotect(l: ::std::os::raw::c_int);

    /// Remove a specific SEXP from anywhere in the protect stack.
    ///
    /// **Cost: O(k)** — scans backwards from top (k = distance from top), then
    /// shifts remaining entries down. No allocation. R source comment:
    /// *"should be among the top few items"*.
    ///
    /// Unlike `Rf_unprotect`, this is order-independent — it finds and removes
    /// the specific pointer regardless of stack position. Useful when LIFO
    /// discipline cannot be maintained, but more expensive than `Rf_unprotect`.
    #[doc(alias = "UNPROTECT_PTR")]
    pub fn Rf_unprotect_ptr(s: SEXP);

    /// Add a SEXP to the global precious list, preventing GC indefinitely.
    ///
    /// **Cost: O(1) but allocates a CONSXP cell** — creates GC pressure on every
    /// call. The precious list is a global linked list (`R_PreciousList`).
    ///
    /// Use only for long-lived objects (e.g., ExternalPtr stored across R calls).
    /// For temporary protection within a function, prefer `Rf_protect`.
    pub fn R_PreserveObject(object: SEXP);

    /// Remove a SEXP from the global precious list, allowing GC.
    ///
    /// **Cost: O(n)** — linear scan of the entire precious list to find and unlink
    /// the cons cell. With `R_HASH_PRECIOUS` env var, O(bucket_size) average
    /// via a 1069-bucket hash table, but this is off by default.
    pub fn R_ReleaseObject(object: SEXP);

    // endregion
    // Vector allocation functions
    #[doc(alias = "allocVector")]
    pub fn Rf_allocVector(sexptype: SEXPTYPE, length: R_xlen_t) -> SEXP;
    #[doc(alias = "allocMatrix")]
    pub fn Rf_allocMatrix(
        sexptype: SEXPTYPE,
        nrow: ::std::os::raw::c_int,
        ncol: ::std::os::raw::c_int,
    ) -> SEXP;
    #[doc(alias = "allocArray")]
    pub fn Rf_allocArray(sexptype: SEXPTYPE, dims: SEXP) -> SEXP;
    #[doc(alias = "alloc3DArray")]
    pub fn Rf_alloc3DArray(
        sexptype: SEXPTYPE,
        nrow: ::std::os::raw::c_int,
        ncol: ::std::os::raw::c_int,
        nface: ::std::os::raw::c_int,
    ) -> SEXP;

    // Pairlist allocation
    // Issue #112 cat. 10: kept pub(crate) — 2 callers in expression.rs/dots.rs; wrapping adds no value
    #[doc(alias = "allocList")]
    pub(crate) fn Rf_allocList(n: ::std::os::raw::c_int) -> SEXP;
    #[doc(alias = "allocLang")]
    pub fn Rf_allocLang(n: ::std::os::raw::c_int) -> SEXP;
    #[doc(alias = "allocS4Object")]
    pub fn Rf_allocS4Object() -> SEXP;

    // Pairlist construction — encapsulated by PairListExt trait
    pub fn Rf_cons(car: SEXP, cdr: SEXP) -> SEXP;
    pub fn Rf_lcons(car: SEXP, cdr: SEXP) -> SEXP;

    // Attribute manipulation — encapsulated by SexpExt methods
    #[doc(alias = "setAttrib")]
    pub fn Rf_setAttrib(vec: SEXP, name: SEXP, val: SEXP) -> SEXP;

    // Rinternals.h — scalar constructors; encapsulated by SEXP::scalar_*() / scalar_*_unchecked()
    #[doc(alias = "ScalarComplex")]
    pub fn Rf_ScalarComplex(x: Rcomplex) -> SEXP;
    #[doc(alias = "ScalarInteger")]
    pub fn Rf_ScalarInteger(x: ::std::os::raw::c_int) -> SEXP;
    #[doc(alias = "ScalarLogical")]
    pub fn Rf_ScalarLogical(x: ::std::os::raw::c_int) -> SEXP;
    #[doc(alias = "ScalarRaw")]
    pub fn Rf_ScalarRaw(x: Rbyte) -> SEXP;
    #[doc(alias = "ScalarReal")]
    pub fn Rf_ScalarReal(x: f64) -> SEXP;
    #[doc(alias = "ScalarString")]
    pub fn Rf_ScalarString(x: SEXP) -> SEXP;

    // Rinternals.h
    /// Non-API function - use DATAPTR_RO or DATAPTR_OR_NULL instead.
    /// Only available with `nonapi` feature.
    #[cfg(feature = "nonapi")]
    pub(crate) fn DATAPTR(x: SEXP) -> *mut ::std::os::raw::c_void;
    pub fn DATAPTR_RO(x: SEXP) -> *const ::std::os::raw::c_void;
    pub fn DATAPTR_OR_NULL(x: SEXP) -> *const ::std::os::raw::c_void;

    // region: Cons Cell (Pairlist) Accessors
    //
    // R's pairlists (LISTSXP) are cons cells like in Lisp/Scheme. Each node has:
    // - CAR: The value/head element
    // - CDR: The rest/tail of the list (another pairlist or R_NilValue)
    // - TAG: An optional name (symbol) for named lists/arguments
    //
    // Example R pairlist: list(a = 1, b = 2, 3)
    // - First node:  CAR=1,    TAG="a",  CDR=<next node>
    // - Second node: CAR=2,    TAG="b",  CDR=<next node>
    // - Third node:  CAR=3,    TAG=NULL, CDR=R_NilValue
    //
    // Pairlists are used for:
    // - Function arguments (formal parameters and actual arguments)
    // - Language objects (calls)
    // - Dotted pairs in old-style lists
    //
    // The names CAR/CDR come from Lisp:
    // - CAR = "Contents of Address part of Register"
    // - CDR = "Contents of Decrement part of Register" (pronounced "could-er")
    //
    // Modern R mostly uses generic vectors (VECSXP) instead of pairlists,
    // but pairlists are still used internally for function calls.

    // Pairlist accessors — basic ops encapsulated by PairListExt trait,
    // compound accessors (CAAR, CADR, etc.) module-private since no callers exist.
    pub fn CAR(e: SEXP) -> SEXP;
    pub fn CDR(e: SEXP) -> SEXP;
    pub fn CAAR(e: SEXP) -> SEXP;
    pub fn CDAR(e: SEXP) -> SEXP;
    pub fn CADR(e: SEXP) -> SEXP;
    pub fn CDDR(e: SEXP) -> SEXP;
    pub fn CADDR(e: SEXP) -> SEXP;
    pub fn CADDDR(e: SEXP) -> SEXP;
    pub fn CAD4R(e: SEXP) -> SEXP;
    pub fn TAG(e: SEXP) -> SEXP;
    pub fn SET_TAG(x: SEXP, y: SEXP);
    pub fn SETCAR(x: SEXP, y: SEXP) -> SEXP;
    pub fn SETCDR(x: SEXP, y: SEXP) -> SEXP;
    pub fn SETCADR(x: SEXP, y: SEXP) -> SEXP;
    pub fn SETCADDR(x: SEXP, y: SEXP) -> SEXP;
    pub fn SETCADDDR(x: SEXP, y: SEXP) -> SEXP;
    pub fn SETCAD4R(e: SEXP, y: SEXP) -> SEXP;
    pub fn LOGICAL_OR_NULL(x: SEXP) -> *const ::std::os::raw::c_int;
    pub fn INTEGER_OR_NULL(x: SEXP) -> *const ::std::os::raw::c_int;
    pub fn REAL_OR_NULL(x: SEXP) -> *const f64;
    pub fn COMPLEX_OR_NULL(x: SEXP) -> *const Rcomplex;
    pub fn RAW_OR_NULL(x: SEXP) -> *const Rbyte;

    // Element-wise accessors (ALTREP-aware) — encapsulated by SexpExt methods
    pub fn INTEGER_ELT(x: SEXP, i: R_xlen_t) -> ::std::os::raw::c_int;
    pub fn REAL_ELT(x: SEXP, i: R_xlen_t) -> f64;
    pub fn LOGICAL_ELT(x: SEXP, i: R_xlen_t) -> ::std::os::raw::c_int;
    pub fn COMPLEX_ELT(x: SEXP, i: R_xlen_t) -> Rcomplex;
    pub fn RAW_ELT(x: SEXP, i: R_xlen_t) -> Rbyte;
    pub fn VECTOR_ELT(x: SEXP, i: R_xlen_t) -> SEXP;
    pub fn STRING_ELT(x: SEXP, i: R_xlen_t) -> SEXP;
    pub fn SET_STRING_ELT(x: SEXP, i: R_xlen_t, v: SEXP);
    pub fn SET_LOGICAL_ELT(x: SEXP, i: R_xlen_t, v: ::std::os::raw::c_int);
    pub fn SET_INTEGER_ELT(x: SEXP, i: R_xlen_t, v: ::std::os::raw::c_int);
    pub fn SET_REAL_ELT(x: SEXP, i: R_xlen_t, v: f64);
    pub fn SET_COMPLEX_ELT(x: SEXP, i: R_xlen_t, v: Rcomplex);
    pub fn SET_RAW_ELT(x: SEXP, i: R_xlen_t, v: Rbyte);
    pub fn SET_VECTOR_ELT(x: SEXP, i: R_xlen_t, v: SEXP) -> SEXP;

    // endregion

    // region: SEXP metadata accessors

    /// Get the length of a SEXP as `int` (for short vectors < 2^31).
    ///
    /// For long vectors, use `Rf_xlength()` instead.
    /// Returns 0 for R_NilValue.
    pub fn LENGTH(x: SEXP) -> ::std::os::raw::c_int;

    /// Get the length of a SEXP as `R_xlen_t` (supports long vectors).
    ///
    /// ALTREP-aware: will call ALTREP Length method if needed.
    pub fn XLENGTH(x: SEXP) -> R_xlen_t;

    /// Get the true length (allocated capacity) of a vector.
    ///
    /// May be larger than LENGTH for vectors with reserved space.
    /// ALTREP-aware.
    pub fn TRUELENGTH(x: SEXP) -> R_xlen_t;

    /// Get the attributes pairlist of a SEXP.
    ///
    /// Returns R_NilValue if no attributes.
    pub fn ATTRIB(x: SEXP) -> SEXP;

    /// Set the attributes pairlist of a SEXP.
    ///
    /// # Safety
    ///
    /// `v` must be a pairlist or R_NilValue
    pub fn SET_ATTRIB(x: SEXP, v: SEXP);

    /// Check if SEXP has the "object" bit set (has a class).
    ///
    /// Returns non-zero if object has a class attribute.
    pub fn OBJECT(x: SEXP) -> ::std::os::raw::c_int;

    /// Set the "object" bit.
    pub fn SET_OBJECT(x: SEXP, v: ::std::os::raw::c_int);

    /// Get the LEVELS field (for factors).
    pub fn LEVELS(x: SEXP) -> ::std::os::raw::c_int;

    /// Set the LEVELS field (for factors).
    ///
    /// Returns the value that was set.
    pub fn SETLEVELS(x: SEXP, v: ::std::os::raw::c_int) -> ::std::os::raw::c_int;

    // endregion

    // region: ALTREP support — data2 encapsulated by AltrepSexpExt; data1 via standalone helpers

    // Issue #112 cat. 6: pub(crate) — no AltrepSexpExt method yet; available for future callers
    pub(crate) fn ALTREP_CLASS(x: SEXP) -> SEXP;
    pub fn R_altrep_data1(x: SEXP) -> SEXP;
    pub fn R_altrep_data2(x: SEXP) -> SEXP;
    pub fn R_set_altrep_data1(x: SEXP, v: SEXP);
    pub fn R_set_altrep_data2(x: SEXP, v: SEXP);

    /// Check if a SEXP is an ALTREP object (returns non-zero if true).
    ///
    /// Use `SexpExt::is_altrep()` instead of calling this directly.
    pub fn ALTREP(x: SEXP) -> ::std::os::raw::c_int;

    // endregion

    // region: Vector data accessors (mutable pointers)
    // Issue #112 cat. 5: kept pub(crate) — raw pointer access needed in RNativeType impls and scattered callers;
    //   partial migration to SexpExt::as_mut_slice() tracked in follow-up issue

    /// Get mutable pointer to logical vector data.
    ///
    /// For ALTREP vectors, this may force materialization.
    /// Get mutable pointer to logical vector data.
    ///
    /// For ALTREP vectors, this may force materialization.
    /// Prefer `SexpExt::set_logical_elt()` / `SexpExt::logical_elt()`.
    pub(crate) fn LOGICAL(x: SEXP) -> *mut ::std::os::raw::c_int;

    /// Get mutable pointer to integer vector data.
    ///
    /// For ALTREP vectors, this may force materialization.
    /// Prefer `SexpExt::set_integer_elt()` / `SexpExt::integer_elt()`.
    pub(crate) fn INTEGER(x: SEXP) -> *mut ::std::os::raw::c_int;

    /// Get mutable pointer to real vector data.
    ///
    /// For ALTREP vectors, this may force materialization.
    /// Prefer `SexpExt::set_real_elt()` / `SexpExt::real_elt()`.
    pub(crate) fn REAL(x: SEXP) -> *mut f64;

    /// Get mutable pointer to complex vector data.
    ///
    /// For ALTREP vectors, this may force materialization.
    /// Prefer `SexpExt::set_complex_elt()` / `SexpExt::complex_elt()`.
    pub(crate) fn COMPLEX(x: SEXP) -> *mut Rcomplex;

    /// Get mutable pointer to raw vector data.
    ///
    /// For ALTREP vectors, this may force materialization.
    /// Prefer `SexpExt::set_raw_elt()` / `SexpExt::raw_elt()`.
    pub(crate) fn RAW(x: SEXP) -> *mut Rbyte;

    // endregion

    // region: User interrupt and utilities

    // utils.h
    pub fn R_CheckUserInterrupt();

    // endregion

    // region: Type checking — encapsulated by SexpExt::type_of()

    pub fn TYPEOF(x: SEXP) -> SEXPTYPE;

    // endregion

    // Symbol creation and access
    #[doc(alias = "install")]
    pub fn Rf_install(name: *const ::std::os::raw::c_char) -> SEXP;
    /// Get the print name (CHARSXP) of a symbol (SYMSXP)
    pub fn PRINTNAME(x: SEXP) -> SEXP;
    /// Get the C string pointer from a CHARSXP — encapsulated by SexpExt::r_char()
    #[doc(alias = "CHAR")]
    pub fn R_CHAR(x: SEXP) -> *const ::std::os::raw::c_char;

    // Attribute access
    // Attribute accessors — encapsulated by SexpExt methods
    /// Read an attribute from an object by symbol (e.g. `R_NamesSymbol`).
    ///
    /// Returns `R_NilValue` if the attribute is not set.
    #[doc(alias = "getAttrib")]
    pub fn Rf_getAttrib(vec: SEXP, name: SEXP) -> SEXP;
    /// Set the `names` attribute; returns the updated object.
    #[doc(alias = "namesgets")]
    pub fn Rf_namesgets(vec: SEXP, val: SEXP) -> SEXP;
    /// Set the `dim` attribute; returns the updated object.
    #[doc(alias = "dimgets")]
    pub fn Rf_dimgets(vec: SEXP, val: SEXP) -> SEXP;

    // Duplication
    #[doc(alias = "duplicate")]
    pub fn Rf_duplicate(s: SEXP) -> SEXP;
    #[doc(alias = "shallow_duplicate")]
    pub fn Rf_shallow_duplicate(s: SEXP) -> SEXP;

    // Object comparison
    /// Check if two R objects are identical (deep semantic equality).
    ///
    /// This is the C implementation of R's `identical()` function.
    ///
    /// # Flags
    ///
    /// Use the `IDENT_*` constants below. Flags are inverted: set bit = disable that check.
    ///
    /// **Default from R**: `IDENT_USE_CLOENV` (16) - ignore closure environments
    ///
    /// # Returns
    ///
    /// `TRUE` if identical, `FALSE` otherwise.
    ///
    /// # Performance
    ///
    /// Fast-path: Returns `TRUE` immediately if pointers are equal.
    pub fn R_compute_identical(x: SEXP, y: SEXP, flags: ::std::os::raw::c_int) -> Rboolean;
}

/// Flags for `R_compute_identical` (bitmask, inverted logic: set bit = disable check).
pub const IDENT_NUM_AS_BITS: ::std::os::raw::c_int = 1;
/// Treat all NAs as identical (ignore NA payload differences).
pub const IDENT_NA_AS_BITS: ::std::os::raw::c_int = 2;
/// Compare attributes in order (not as a set).
pub const IDENT_ATTR_BY_ORDER: ::std::os::raw::c_int = 4;
/// Include bytecode in comparison.
pub const IDENT_USE_BYTECODE: ::std::os::raw::c_int = 8;
/// Include closure environments in comparison.
pub const IDENT_USE_CLOENV: ::std::os::raw::c_int = 16;
/// Include source references in comparison.
pub const IDENT_USE_SRCREF: ::std::os::raw::c_int = 32;
/// Compare external pointers as references (not by address).
pub const IDENT_EXTPTR_AS_REF: ::std::os::raw::c_int = 64;

// Additional checked R API declarations used by conversion and reflection code.
#[allow(missing_docs)]
#[r_ffi_checked]
unsafe extern "C-unwind" {
    // Type coercion — encapsulated by SexpExt methods
    #[doc(alias = "asLogical")]
    pub fn Rf_asLogical(x: SEXP) -> ::std::os::raw::c_int;
    #[doc(alias = "asInteger")]
    pub fn Rf_asInteger(x: SEXP) -> ::std::os::raw::c_int;
    #[doc(alias = "asReal")]
    pub fn Rf_asReal(x: SEXP) -> f64;
    #[doc(alias = "asChar")]
    pub fn Rf_asChar(x: SEXP) -> SEXP;
    #[doc(alias = "coerceVector")]
    pub fn Rf_coerceVector(v: SEXP, sexptype: SEXPTYPE) -> SEXP;

    // Matrix utilities — no callers outside ffi.rs
    #[doc(alias = "nrows")]
    pub fn Rf_nrows(x: SEXP) -> ::std::os::raw::c_int;
    #[doc(alias = "ncols")]
    pub fn Rf_ncols(x: SEXP) -> ::std::os::raw::c_int;

    // Inheritance checking — encapsulated by SexpExt::inherits_class()
    #[doc(alias = "inherits")]
    pub fn Rf_inherits(x: SEXP, klass: *const ::std::os::raw::c_char) -> Rboolean;

    // Type checking predicates — encapsulated by SexpExt type-check methods
    #[doc(alias = "isNull")]
    pub fn Rf_isNull(s: SEXP) -> Rboolean;
    #[doc(alias = "isSymbol")]
    pub fn Rf_isSymbol(s: SEXP) -> Rboolean;
    #[doc(alias = "isLogical")]
    pub fn Rf_isLogical(s: SEXP) -> Rboolean;
    #[doc(alias = "isReal")]
    pub fn Rf_isReal(s: SEXP) -> Rboolean;
    #[doc(alias = "isComplex")]
    pub fn Rf_isComplex(s: SEXP) -> Rboolean;
    #[doc(alias = "isExpression")]
    pub fn Rf_isExpression(s: SEXP) -> Rboolean;
    #[doc(alias = "isEnvironment")]
    pub fn Rf_isEnvironment(s: SEXP) -> Rboolean;
    #[doc(alias = "isString")]
    pub fn Rf_isString(s: SEXP) -> Rboolean;

    // Composite type checking (from inline functions)
    #[doc(alias = "isArray")]
    pub fn Rf_isArray(s: SEXP) -> Rboolean;
    #[doc(alias = "isMatrix")]
    pub fn Rf_isMatrix(s: SEXP) -> Rboolean;
    #[doc(alias = "isList")]
    pub fn Rf_isList(s: SEXP) -> Rboolean;
    #[doc(alias = "isNewList")]
    pub fn Rf_isNewList(s: SEXP) -> Rboolean;
    #[doc(alias = "isPairList")]
    pub fn Rf_isPairList(s: SEXP) -> Rboolean;
    #[doc(alias = "isFunction")]
    pub fn Rf_isFunction(s: SEXP) -> Rboolean;
    #[doc(alias = "isPrimitive")]
    pub fn Rf_isPrimitive(s: SEXP) -> Rboolean;
    #[doc(alias = "isLanguage")]
    pub fn Rf_isLanguage(s: SEXP) -> Rboolean;
    #[doc(alias = "isDataFrame")]
    pub fn Rf_isDataFrame(s: SEXP) -> Rboolean;
    #[doc(alias = "isFactor")]
    pub fn Rf_isFactor(s: SEXP) -> Rboolean;
    #[doc(alias = "isInteger")]
    pub fn Rf_isInteger(s: SEXP) -> Rboolean;
    #[doc(alias = "isObject")]
    pub fn Rf_isObject(s: SEXP) -> Rboolean;

    // Pairlist utilities
    #[doc(alias = "elt")]
    pub fn Rf_elt(list: SEXP, i: ::std::os::raw::c_int) -> SEXP;
    #[doc(alias = "lastElt")]
    pub fn Rf_lastElt(list: SEXP) -> SEXP;
    #[doc(alias = "nthcdr")]
    pub fn Rf_nthcdr(list: SEXP, n: ::std::os::raw::c_int) -> SEXP;
    #[doc(alias = "listAppend")]
    pub fn Rf_listAppend(s: SEXP, t: SEXP) -> SEXP;

    // More attribute setters (using R's "gets" suffix convention)
    //
    // See "Attribute access" section above for explanation of the "gets" suffix.
    // These are setter functions equivalent to R's `attr(x) <- value` syntax.

    /// Set the class attribute of a vector.
    ///
    /// Equivalent to R's `class(vec) <- klass` syntax.
    /// The "gets" suffix indicates this is a setter function.
    ///
    /// # Returns
    ///
    /// Returns the modified vector (like all "*gets" functions).
    #[doc(alias = "classgets")]
    pub fn Rf_classgets(vec: SEXP, klass: SEXP) -> SEXP;

    /// Set the dimnames attribute of an array/matrix.
    ///
    /// Equivalent to R's `dimnames(vec) <- val` syntax.
    /// The "gets" suffix indicates this is a setter function.
    ///
    /// # Returns
    ///
    /// Returns the modified vector.
    #[doc(alias = "dimnamesgets")]
    pub fn Rf_dimnamesgets(vec: SEXP, val: SEXP) -> SEXP;
    // Issue #112 cat. 10: kept pub(crate) — 2 callers each in factor.rs/matrix helpers; wrapping adds no value
    #[doc(alias = "GetRowNames")]
    pub(crate) fn Rf_GetRowNames(dimnames: SEXP) -> SEXP;
    #[doc(alias = "GetColNames")]
    pub(crate) fn Rf_GetColNames(dimnames: SEXP) -> SEXP;

    // Environment operations
    #[doc(alias = "findVar")]
    pub fn Rf_findVar(symbol: SEXP, rho: SEXP) -> SEXP;
    #[doc(alias = "findVarInFrame")]
    pub fn Rf_findVarInFrame(rho: SEXP, symbol: SEXP) -> SEXP;
    #[doc(alias = "findVarInFrame3")]
    pub fn Rf_findVarInFrame3(rho: SEXP, symbol: SEXP, doget: Rboolean) -> SEXP;
    #[doc(alias = "defineVar")]
    pub fn Rf_defineVar(symbol: SEXP, value: SEXP, rho: SEXP);
    #[doc(alias = "setVar")]
    pub fn Rf_setVar(symbol: SEXP, value: SEXP, rho: SEXP);
    #[doc(alias = "findFun")]
    pub fn Rf_findFun(symbol: SEXP, rho: SEXP) -> SEXP;

    /// Find a registered namespace by name. **Longjmps on error** — prefer
    /// `REnv::package_namespace()` which wraps this safely.
    #[doc(alias = "FindNamespace")]
    pub fn R_FindNamespace(info: SEXP) -> SEXP;

    // Issue #112 cat. 9: kept pub(crate) — R_GetCurrentEnv used from s4_helpers.rs; R_tryEvalSilent from expression.rs
    /// Return the current execution environment (innermost closure on call
    /// stack, or `R_GlobalEnv` if none).
    #[doc(alias = "GetCurrentEnv")]
    pub(crate) fn R_GetCurrentEnv() -> SEXP;

    // Evaluation
    #[doc(alias = "eval")]
    pub fn Rf_eval(expr: SEXP, rho: SEXP) -> SEXP;
    #[doc(alias = "applyClosure")]
    pub fn Rf_applyClosure(
        call: SEXP,
        op: SEXP,
        args: SEXP,
        rho: SEXP,
        suppliedvars: SEXP,
        check: Rboolean,
    ) -> SEXP;
    pub fn R_tryEval(expr: SEXP, env: SEXP, error_occurred: *mut ::std::os::raw::c_int) -> SEXP;
    pub(crate) fn R_tryEvalSilent(
        expr: SEXP,
        env: SEXP,
        error_occurred: *mut ::std::os::raw::c_int,
    ) -> SEXP;
    pub fn R_forceAndCall(e: SEXP, n: ::std::os::raw::c_int, rho: SEXP) -> SEXP;

    /// Parse R source text into an EXPRSXP (a list of parsed expressions).
    ///
    /// `text` is a STRSXP holding the source, `n` is the number of expressions
    /// to parse (`-1` for all), `status` receives the [`ParseStatus`] outcome,
    /// and `srcfile` is a srcref/`R_NilValue`. Allocates; protect the result.
    ///
    /// Prefer the safe [`crate::expression::r_eval_str`] wrapper, which does the
    /// STRSXP construction, status check, and protection bookkeeping for you.
    #[doc(alias = "ParseVector")]
    pub fn R_ParseVector(
        text: SEXP,
        n: ::std::os::raw::c_int,
        status: *mut ParseStatus,
        srcfile: SEXP,
    ) -> SEXP;
}

/// Outcome of [`R_ParseVector`] (from `R_ext/Parse.h`).
///
/// `PARSE_NULL` is never returned by `R_ParseVector`; the meaningful success
/// value is [`ParseStatus::PARSE_OK`]. The remaining variants indicate parse
/// failures (`PARSE_ERROR`), incomplete input (`PARSE_INCOMPLETE`), or
/// end-of-input (`PARSE_EOF`).
#[allow(non_camel_case_types)]
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseStatus {
    /// Never returned by `R_ParseVector`; the default-initialized sentinel.
    PARSE_NULL,
    /// Parse succeeded.
    PARSE_OK,
    /// Input ended mid-expression (e.g. an unbalanced delimiter).
    PARSE_INCOMPLETE,
    /// A syntax error was encountered.
    PARSE_ERROR,
    /// End of input reached with no further expressions.
    PARSE_EOF,
}

// region: Connections API (R_ext/Connections.h)
//
// Gated behind `connections` feature because R's connection API is explicitly UNSTABLE.
// From R_ext/Connections.h:
//   "IMPORTANT: we do not expect future connection APIs to be
//    backward-compatible so if you use this, you *must* check the
//    version and proceeds only if it matches what you expect.
//
//    We explicitly reserve the right to change the connection
//    implementation without a compatibility layer."
//
// Use with caution and always check R_CONNECTIONS_VERSION.
// Issue #112 cat. 8: kept pub(crate) — feature-gated behind `connections`; behind Connection type for users
#[r_ffi_checked]
#[cfg(feature = "connections")]
unsafe extern "C-unwind" {
    /// Create a new custom connection.
    ///
    /// # WARNING
    ///
    /// This API is UNSTABLE. Check `R_CONNECTIONS_VERSION` before use.
    /// The connection implementation may change without notice.
    ///
    /// # Safety
    ///
    /// - `description`, `mode`, and `class_name` must be valid C strings
    /// - `ptr` must be a valid pointer to store the connection handle
    pub(crate) fn R_new_custom_connection(
        description: *const ::std::os::raw::c_char,
        mode: *const ::std::os::raw::c_char,
        class_name: *const ::std::os::raw::c_char,
        ptr: *mut Rconnection,
    ) -> SEXP;

    /// Read from a connection.
    ///
    /// # WARNING
    ///
    /// This API is UNSTABLE and may change.
    ///
    /// # Safety
    ///
    /// - `con` must be a valid Rconnection handle
    /// - `buf` must be a valid buffer with at least `n` bytes
    pub(crate) fn R_ReadConnection(
        con: Rconnection,
        buf: *mut ::std::os::raw::c_void,
        n: usize,
    ) -> usize;

    /// Write to a connection.
    ///
    /// # WARNING
    ///
    /// This API is UNSTABLE and may change.
    ///
    /// # Safety
    ///
    /// - `con` must be a valid Rconnection handle
    /// - `buf` must contain at least `n` valid bytes
    pub(crate) fn R_WriteConnection(
        con: Rconnection,
        buf: *const ::std::os::raw::c_void,
        n: usize,
    ) -> usize;

    /// Get a connection from a SEXP.
    ///
    /// # WARNING
    ///
    /// This API is UNSTABLE and may change.
    /// Added in R 3.3.0.
    ///
    /// # Safety
    ///
    /// - `sConn` must be a valid connection SEXP
    pub(crate) fn R_GetConnection(sConn: SEXP) -> Rconnection;
}
// endregion: Connections API

/// Check if a SEXP is an S4 object.
///
/// # Safety
///
/// - `arg1` must be a valid SEXP
#[allow(non_snake_case)]
pub unsafe fn Rf_isS4(arg1: SEXP) -> Rboolean {
    unsafe extern "C-unwind" {
        #[link_name = "Rf_isS4"]
        pub fn Rf_isS4_original(arg1: SEXP) -> u32;
    }

    unsafe {
        if Rf_isS4_original(arg1) == 0 {
            Rboolean::FALSE
        } else {
            Rboolean::TRUE
        }
    }
}

// region: registration!

#[repr(C)]
#[derive(Debug)]
/// Opaque dynamic library descriptor from R.
pub struct DllInfo(::std::os::raw::c_void);

/// Generic dynamic library function pointer.
///
/// R defines this as `void *(*)(void)` - a function taking no arguments and
/// returning `void*`. This is used for method registration and external pointer
/// functions. The actual function signatures vary; callers cast to the appropriate
/// concrete function type before calling.
///
/// We use `fn() -> *mut c_void` to match R's signature. The function pointer is
/// stored generically and cast to the appropriate type when called by R.
#[allow(non_camel_case_types)]
pub type DL_FUNC =
    ::std::option::Option<unsafe extern "C-unwind" fn() -> *mut ::std::os::raw::c_void>;

/// Type descriptor for native primitive arguments in .C/.Fortran calls.
///
/// This is used in `R_CMethodDef` and `R_FortranMethodDef` to specify
/// argument types for type checking.
#[allow(non_camel_case_types)]
pub type R_NativePrimitiveArgType = ::std::os::raw::c_uint;

/// Method definition for .C interface routines.
///
/// Used to register C functions callable via `.C()` from R.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct R_CMethodDef {
    /// Exported symbol name.
    pub name: *const ::std::os::raw::c_char,
    /// Function pointer implementing the routine.
    pub fun: DL_FUNC,
    /// Declared arity.
    pub numArgs: ::std::os::raw::c_int,
    /// Optional array of argument types for type checking. May be null.
    pub types: *const R_NativePrimitiveArgType,
}

/// Method definition for .Fortran interface routines.
///
/// Structurally identical to `R_CMethodDef`.
#[allow(non_camel_case_types)]
pub type R_FortranMethodDef = R_CMethodDef;

/// Method definition for .Call interface routines.
///
/// Used to register C functions callable via `.Call()` from R.
/// Unlike `.C()` routines, `.Call()` functions receive and return SEXP values directly.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct R_CallMethodDef {
    /// Exported symbol name.
    pub name: *const ::std::os::raw::c_char,
    /// Function pointer implementing the routine.
    pub fun: DL_FUNC,
    /// Declared arity.
    pub numArgs: ::std::os::raw::c_int,
}

// SAFETY: `name` points to a static CStr literal, `fun` is a function pointer.
// Both are valid for program lifetime and safe to read from any thread.
unsafe impl Sync for R_CallMethodDef {}
unsafe impl Send for R_CallMethodDef {}

/// Method definition for .External interface routines.
///
/// Structurally identical to `R_CallMethodDef`.
#[allow(non_camel_case_types)]
pub type R_ExternalMethodDef = R_CallMethodDef;

// Checked routine registration API declarations.
// Issue #112 cat. 7: kept pub(crate) — only called from init.rs during package init; not worth a wrapper type
#[allow(missing_docs)]
#[r_ffi_checked]
#[allow(clashing_extern_declarations)]
unsafe extern "C-unwind" {
    pub(crate) fn R_registerRoutines(
        info: *mut DllInfo,
        croutines: *const R_CMethodDef,
        callRoutines: *const R_CallMethodDef,
        fortranRoutines: *const R_FortranMethodDef,
        externalRoutines: *const R_ExternalMethodDef,
    ) -> ::std::os::raw::c_int;

    pub(crate) fn R_useDynamicSymbols(info: *mut DllInfo, value: Rboolean) -> Rboolean;
    pub(crate) fn R_forceSymbols(info: *mut DllInfo, value: Rboolean) -> Rboolean;
}

// endregion

// region: Non-API encoding/locale state (Defn.h)

/// Non-API encoding / locale helpers from R's `Defn.h`.
///
/// These are not part of the stable R API and may break across R versions.
///
/// Only symbols R's shared library actually **exports** may be declared here.
/// `Defn.h` marks most locale globals `extern0` (= `attribute_hidden`) —
/// referencing one of those (e.g. `known_to_be_utf8`, `latin1locale`,
/// `R_nativeEncoding`) compiles fine but aborts `dyn.load` of any binary that
/// carries the reference: data relocations resolve eagerly at load, whether or
/// not the code path ever runs. That made every `nonapi` build un-loadable
/// (caught by the feature-legs CI, audit A5). `utf8locale`, `mbcslocale`, and
/// `known_to_be_latin1` are plain `extern` and exported (verified against
/// R 4.6's libR).
#[cfg(feature = "nonapi")]
pub mod nonapi_encoding {
    use super::r_ffi_checked;

    // Issue #112 cat. 10: kept pub(crate) — nonapi encoding helpers; single-caller utilities in encoding.rs
    #[r_ffi_checked]
    #[allow(clashing_extern_declarations)]
    unsafe extern "C-unwind" {
        // Locale flags (exported, non-hidden)
        pub(crate) static utf8locale: super::Rboolean;
        pub(crate) static mbcslocale: super::Rboolean;
        pub(crate) static known_to_be_latin1: super::Rboolean;
    }
}

// endregion

// region: Non-API stack checking variables (Rinterface.h)

/// Non-API stack checking variables from `Rinterface.h`.
///
/// R uses these to detect stack overflow. When calling R from a thread other
/// than the main R thread, stack checking will fail because these values are
/// set for the main thread's stack.
///
/// # Usage
///
/// To safely call R from a worker thread, disable stack checking:
/// ```ignore
/// #[cfg(feature = "nonapi")]
/// unsafe {
///     use miniextendr_api::sys::nonapi_stack::*;
///     let saved = get_r_cstack_limit();
///     set_r_cstack_limit(usize::MAX); // disable checking
///     // ... call R APIs ...
///     set_r_cstack_limit(saved); // restore
/// }
/// ```
///
/// Or use the higher-level [`StackCheckGuard`](crate::thread::StackCheckGuard) which handles this automatically.
///
/// Setting `R_CStackLimit` to `usize::MAX` (i.e., `-1` as `uintptr_t`) disables
/// stack checking entirely.
#[cfg(feature = "nonapi")]
pub mod nonapi_stack {
    unsafe extern "C" {
        /// Top of the stack (set during `Rf_initialize_R` for main thread).
        ///
        /// On Unix, determined via `__libc_stack_end`, `KERN_USRSTACK`, or
        /// `thr_stksegment`. On Windows, via `VirtualQuery`.
        #[allow(non_upper_case_globals)]
        pub(crate) static R_CStackStart: usize;

        /// Stack size limit. Set to `usize::MAX` to disable stack checking.
        ///
        /// From R source: `if(R_CStackStart == -1) R_CStackLimit = -1; /* never set */`
        #[allow(non_upper_case_globals)]
        pub static R_CStackLimit: usize;

        /// Stack growth direction: 1 = grows up, -1 = grows down.
        ///
        /// Most systems (x86, ARM) grow down (-1).
        #[allow(non_upper_case_globals)]
        pub(crate) static R_CStackDir: ::std::os::raw::c_int;
    }

    /// Write to `R_CStackLimit`.
    ///
    /// # Safety
    /// Must be called from R's main thread.
    #[inline]
    pub unsafe fn set_r_cstack_limit(value: usize) {
        unsafe {
            let ptr = (&raw const R_CStackLimit).cast_mut();
            ptr.write(value);
        }
    }

    // Issue #112 cat. 10: kept pub(crate) — nonapi stack helpers; used from thread.rs; wrapping adds no value
    /// Read `R_CStackLimit`.
    #[inline]
    pub(crate) fn get_r_cstack_limit() -> usize {
        unsafe { R_CStackLimit }
    }

    /// Read `R_CStackStart`.
    #[inline]
    pub(crate) fn get_r_cstack_start() -> usize {
        unsafe { R_CStackStart }
    }

    /// Read `R_CStackDir`.
    #[inline]
    pub(crate) fn get_r_cstack_dir() -> ::std::os::raw::c_int {
        unsafe { R_CStackDir }
    }
}

// endregion

// region: Inline Helper Functions (Rust implementations of R's inline functions)

/// Create a length-1 string vector from a C string.
///
/// Rust equivalent of R's inline `Rf_mkString(s)`, which is
/// shorthand for `ScalarString(mkChar(s))`.
///
/// # Safety
///
/// - `s` must be a valid null-terminated C string
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "mkString")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_mkString(s: *const ::std::os::raw::c_char) -> SEXP {
    unsafe {
        let charsxp = Rf_mkChar(s);
        let protected = Rf_protect(charsxp);
        let result = Rf_ScalarString(protected);
        Rf_unprotect(1);
        result
    }
}

/// Build a pairlist with 1 element.
///
/// Rust equivalent of R's inline `Rf_list1(s)`.
///
/// # Safety
///
/// - `s` must be a valid SEXP
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "list1")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_list1(s: SEXP) -> SEXP {
    unsafe { Rf_cons(s, R_NilValue) }
}

/// Build a pairlist with 2 elements.
///
/// Rust equivalent of R's inline `Rf_list2(s, t)`.
///
/// # Safety
///
/// - Both SEXPs must be valid
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "list2")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_list2(s: SEXP, t: SEXP) -> SEXP {
    unsafe { Rf_cons(s, Rf_cons(t, R_NilValue)) }
}

/// Build a pairlist with 3 elements.
///
/// Rust equivalent of R's inline `Rf_list3(s, t, u)`.
///
/// # Safety
///
/// - All SEXPs must be valid
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "list3")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_list3(s: SEXP, t: SEXP, u: SEXP) -> SEXP {
    unsafe { Rf_cons(s, Rf_cons(t, Rf_cons(u, R_NilValue))) }
}

/// Build a pairlist with 4 elements.
///
/// Rust equivalent of R's inline `Rf_list4(s, t, u, v)`.
///
/// # Safety
///
/// - All SEXPs must be valid
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "list4")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_list4(s: SEXP, t: SEXP, u: SEXP, v: SEXP) -> SEXP {
    unsafe { Rf_cons(s, Rf_cons(t, Rf_cons(u, Rf_cons(v, R_NilValue)))) }
}

/// Build a language object (call) with 1 element (the function).
///
/// Rust equivalent of R's inline `Rf_lang1(s)`.
/// Creates a call like `f()` where `s` is the function.
///
/// # Safety
///
/// - `s` must be a valid SEXP (typically a symbol or closure)
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "lang1")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_lang1(s: SEXP) -> SEXP {
    unsafe { Rf_lcons(s, R_NilValue) }
}

/// Build a language object (call) with function and 1 argument.
///
/// Rust equivalent of R's inline `Rf_lang2(s, t)`.
/// Creates a call like `f(arg)` where `s` is the function and `t` is the argument.
///
/// # Safety
///
/// - Both SEXPs must be valid
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "lang2")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_lang2(s: SEXP, t: SEXP) -> SEXP {
    unsafe { Rf_lcons(s, Rf_list1(t)) }
}

/// Build a language object (call) with function and 2 arguments.
///
/// Rust equivalent of R's inline `Rf_lang3(s, t, u)`.
/// Creates a call like `f(arg1, arg2)`.
///
/// # Safety
///
/// - All SEXPs must be valid
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "lang3")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_lang3(s: SEXP, t: SEXP, u: SEXP) -> SEXP {
    unsafe { Rf_lcons(s, Rf_list2(t, u)) }
}

/// Build a language object (call) with function and 3 arguments.
///
/// Rust equivalent of R's inline `Rf_lang4(s, t, u, v)`.
/// Creates a call like `f(arg1, arg2, arg3)`.
///
/// # Safety
///
/// - All SEXPs must be valid
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "lang4")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_lang4(s: SEXP, t: SEXP, u: SEXP, v: SEXP) -> SEXP {
    unsafe { Rf_lcons(s, Rf_list3(t, u, v)) }
}

/// Build a language object (call) with function and 4 arguments.
///
/// Rust equivalent of R's inline `Rf_lang5(s, t, u, v, w)`.
/// Creates a call like `f(arg1, arg2, arg3, arg4)`.
///
/// # Safety
///
/// - All SEXPs must be valid
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "lang5")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_lang5(s: SEXP, t: SEXP, u: SEXP, v: SEXP, w: SEXP) -> SEXP {
    unsafe { Rf_lcons(s, Rf_list4(t, u, v, w)) }
}

/// Build a language object (call) with function and 5 arguments.
///
/// Rust equivalent of R's inline `Rf_lang6(s, t, u, v, w, x)`.
/// Creates a call like `f(arg1, arg2, arg3, arg4, arg5)`.
///
/// # Safety
///
/// - All SEXPs must be valid
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "lang6")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_lang6(s: SEXP, t: SEXP, u: SEXP, v: SEXP, w: SEXP, x: SEXP) -> SEXP {
    unsafe {
        let protected = Rf_protect(s);
        let list = Rf_cons(t, Rf_list4(u, v, w, x));
        let result = Rf_lcons(protected, list);
        Rf_unprotect(1);
        result
    }
}

// endregion

// region: RNG functions (R_ext/Random.h)

/// RNG type enum from R_ext/Random.h
#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum RNGtype {
    /// Wichmann-Hill generator.
    WICHMANN_HILL = 0,
    /// Marsaglia-Multicarry generator.
    MARSAGLIA_MULTICARRY = 1,
    /// Super-Duper generator.
    SUPER_DUPER = 2,
    /// Mersenne Twister generator.
    MERSENNE_TWISTER = 3,
    /// Knuth TAOCP generator.
    KNUTH_TAOCP = 4,
    /// User-supplied uniform generator.
    USER_UNIF = 5,
    /// Knuth TAOCP 2002 variant.
    KNUTH_TAOCP2 = 6,
    /// L'Ecuyer-CMRG generator.
    LECUYER_CMRG = 7,
}

/// Normal distribution generator type enum from R_ext/Random.h
#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum N01type {
    /// Legacy buggy Kinderman-Ramage method.
    BUGGY_KINDERMAN_RAMAGE = 0,
    /// Ahrens-Dieter method.
    AHRENS_DIETER = 1,
    /// Box-Muller transform.
    BOX_MULLER = 2,
    /// User-supplied normal generator.
    USER_NORM = 3,
    /// Inversion method.
    INVERSION = 4,
    /// Fixed Kinderman-Ramage method.
    KINDERMAN_RAMAGE = 5,
}

/// Discrete uniform sample method enum from R_ext/Random.h
#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Sampletype {
    /// Rounding method for integer sampling.
    ROUNDING = 0,
    /// Rejection sampling method.
    REJECTION = 1,
}

#[r_ffi_checked]
unsafe extern "C-unwind" {
    /// Save the current RNG state from R's global state.
    ///
    /// Must be called before using `unif_rand()`, `norm_rand()`, etc.
    /// The state is restored with `PutRNGstate()`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// unsafe {
    ///     GetRNGstate();
    ///     let x = unif_rand();
    ///     let y = norm_rand();
    ///     PutRNGstate();
    /// }
    /// ```
    pub fn GetRNGstate();

    /// Restore the RNG state to R's global state.
    ///
    /// Must be called after using `unif_rand()`, `norm_rand()`, etc.
    /// to ensure R's `.Random.seed` is updated.
    pub fn PutRNGstate();

    /// Generate a uniform random number in (0, 1).
    ///
    /// # Important
    ///
    /// Must call `GetRNGstate()` before and `PutRNGstate()` after.
    pub fn unif_rand() -> f64;

    /// Generate a standard normal random number (mean 0, sd 1).
    ///
    /// # Important
    ///
    /// Must call `GetRNGstate()` before and `PutRNGstate()` after.
    pub fn norm_rand() -> f64;

    /// Generate an exponential random number with rate 1.
    ///
    /// # Important
    ///
    /// Must call `GetRNGstate()` before and `PutRNGstate()` after.
    pub fn exp_rand() -> f64;

    /// Generate a uniform random index in [0, dn).
    ///
    /// Used for sampling without bias for large n.
    ///
    /// # Important
    ///
    /// Must call `GetRNGstate()` before and `PutRNGstate()` after.
    pub fn R_unif_index(dn: f64) -> f64;

    /// Get the current discrete uniform sample method.
    pub fn R_sample_kind() -> Sampletype;
}

// endregion

// region: Memory allocation (R_ext/Memory.h)

#[r_ffi_checked]
unsafe extern "C-unwind" {
    /// Get the current R memory stack watermark.
    ///
    /// Use with `vmaxset()` to restore memory stack state.
    /// Memory allocated with `R_alloc()` between `vmaxget()` and `vmaxset()`
    /// will be freed when `vmaxset()` is called.
    ///
    /// # Example
    ///
    /// ```ignore
    /// unsafe {
    ///     let watermark = vmaxget();
    ///     let buf = R_alloc(100, 1);
    ///     // ... use buf ...
    ///     vmaxset(watermark); // frees buf
    /// }
    /// ```
    pub fn vmaxget() -> *mut ::std::os::raw::c_void;

    /// Set the R memory stack watermark, freeing memory allocated since the mark.
    ///
    /// # Safety
    ///
    /// `ovmax` must be a value returned by `vmaxget()` called earlier in the
    /// same R evaluation context.
    pub fn vmaxset(ovmax: *const ::std::os::raw::c_void);

    /// Run the R garbage collector.
    ///
    /// Forces a full garbage collection cycle.
    pub fn R_gc();

    /// Check if the garbage collector is currently running.
    ///
    /// Returns non-zero if GC is in progress.
    pub fn R_gc_running() -> ::std::os::raw::c_int;

    /// Allocate memory on R's memory stack.
    ///
    /// This memory is automatically freed when the calling R function returns,
    /// or can be freed earlier with `vmaxset()`.
    ///
    /// # Parameters
    ///
    /// - `nelem`: Number of elements to allocate
    /// - `eltsize`: Size of each element in bytes
    ///
    /// # Returns
    ///
    /// Pointer to allocated memory (as `char*` for compatibility with S).
    pub fn R_alloc(nelem: usize, eltsize: ::std::os::raw::c_int) -> *mut ::std::os::raw::c_char;

    /// Allocate an array of long doubles on R's memory stack.
    ///
    /// # Parameters
    ///
    /// - `nelem`: Number of long double elements to allocate
    pub fn R_allocLD(nelem: usize) -> *mut f64; // Note: f64 is close enough for most uses

    /// S compatibility: allocate zeroed memory on R's memory stack.
    ///
    /// # Parameters
    ///
    /// - `nelem`: Number of elements
    /// - `eltsize`: Size of each element
    pub fn S_alloc(
        nelem: ::std::os::raw::c_long,
        eltsize: ::std::os::raw::c_int,
    ) -> *mut ::std::os::raw::c_char;

    /// S compatibility: reallocate memory on R's memory stack.
    ///
    /// # Safety
    ///
    /// `ptr` must have been allocated by `S_alloc`.
    pub fn S_realloc(
        ptr: *mut ::std::os::raw::c_char,
        newsize: ::std::os::raw::c_long,
        oldsize: ::std::os::raw::c_long,
        eltsize: ::std::os::raw::c_int,
    ) -> *mut ::std::os::raw::c_char;

    /// GC-aware malloc.
    ///
    /// Triggers GC if allocation fails, then retries.
    /// Memory must be freed with `free()`.
    pub fn R_malloc_gc(size: usize) -> *mut ::std::os::raw::c_void;

    /// GC-aware calloc.
    ///
    /// Triggers GC if allocation fails, then retries.
    /// Memory must be freed with `free()`.
    pub fn R_calloc_gc(nelem: usize, eltsize: usize) -> *mut ::std::os::raw::c_void;

    /// GC-aware realloc.
    ///
    /// Triggers GC if allocation fails, then retries.
    /// Memory must be freed with `free()`.
    pub fn R_realloc_gc(
        ptr: *mut ::std::os::raw::c_void,
        size: usize,
    ) -> *mut ::std::os::raw::c_void;
}

// endregion

// region: Sorting and utility functions (R_ext/Utils.h)

#[r_ffi_checked]
unsafe extern "C-unwind" {
    /// Sort an integer vector in place (ascending order).
    ///
    /// # Parameters
    ///
    /// - `x`: Pointer to integer array
    /// - `n`: Number of elements
    pub fn R_isort(x: *mut ::std::os::raw::c_int, n: ::std::os::raw::c_int);

    /// Sort a double vector in place (ascending order).
    ///
    /// # Parameters
    ///
    /// - `x`: Pointer to double array
    /// - `n`: Number of elements
    pub fn R_rsort(x: *mut f64, n: ::std::os::raw::c_int);

    /// Sort a complex vector in place.
    ///
    /// # Parameters
    ///
    /// - `x`: Pointer to Rcomplex array
    /// - `n`: Number of elements
    pub fn R_csort(x: *mut Rcomplex, n: ::std::os::raw::c_int);

    /// Sort doubles in descending order, carrying along an index array.
    ///
    /// # Parameters
    ///
    /// - `a`: Pointer to double array (sorted in place, descending)
    /// - `ib`: Pointer to integer array (permuted alongside `a`)
    /// - `n`: Number of elements
    #[doc(alias = "Rf_revsort")]
    pub fn revsort(a: *mut f64, ib: *mut ::std::os::raw::c_int, n: ::std::os::raw::c_int);

    /// Sort doubles with index array.
    ///
    /// # Parameters
    ///
    /// - `x`: Pointer to double array (sorted in place)
    /// - `indx`: Pointer to integer array (permuted alongside `x`)
    /// - `n`: Number of elements
    pub fn rsort_with_index(
        x: *mut f64,
        indx: *mut ::std::os::raw::c_int,
        n: ::std::os::raw::c_int,
    );

    /// Partial sort integers (moves k-th smallest to position k).
    ///
    /// # Parameters
    ///
    /// - `x`: Pointer to integer array
    /// - `n`: Number of elements
    /// - `k`: Target position (0-indexed)
    #[doc(alias = "Rf_iPsort")]
    pub fn iPsort(
        x: *mut ::std::os::raw::c_int,
        n: ::std::os::raw::c_int,
        k: ::std::os::raw::c_int,
    );

    /// Partial sort doubles (moves k-th smallest to position k).
    ///
    /// # Parameters
    ///
    /// - `x`: Pointer to double array
    /// - `n`: Number of elements
    /// - `k`: Target position (0-indexed)
    #[doc(alias = "Rf_rPsort")]
    pub fn rPsort(x: *mut f64, n: ::std::os::raw::c_int, k: ::std::os::raw::c_int);

    /// Partial sort complex numbers.
    ///
    /// # Parameters
    ///
    /// - `x`: Pointer to Rcomplex array
    /// - `n`: Number of elements
    /// - `k`: Target position (0-indexed)
    #[doc(alias = "Rf_cPsort")]
    pub fn cPsort(x: *mut Rcomplex, n: ::std::os::raw::c_int, k: ::std::os::raw::c_int);

    /// Quicksort doubles in place.
    ///
    /// # Parameters
    ///
    /// - `v`: Pointer to double array
    /// - `i`: Start index (1-indexed for R compatibility)
    /// - `j`: End index (1-indexed)
    pub fn R_qsort(v: *mut f64, i: usize, j: usize);

    /// Quicksort doubles with index array.
    ///
    /// # Parameters
    ///
    /// - `v`: Pointer to double array
    /// - `indx`: Pointer to index array (permuted alongside v)
    /// - `i`: Start index (1-indexed)
    /// - `j`: End index (1-indexed)
    pub fn R_qsort_I(
        v: *mut f64,
        indx: *mut ::std::os::raw::c_int,
        i: ::std::os::raw::c_int,
        j: ::std::os::raw::c_int,
    );

    /// Quicksort integers in place.
    ///
    /// # Parameters
    ///
    /// - `iv`: Pointer to integer array
    /// - `i`: Start index (1-indexed)
    /// - `j`: End index (1-indexed)
    pub fn R_qsort_int(iv: *mut ::std::os::raw::c_int, i: usize, j: usize);

    /// Quicksort integers with index array.
    ///
    /// # Parameters
    ///
    /// - `iv`: Pointer to integer array
    /// - `indx`: Pointer to index array
    /// - `i`: Start index (1-indexed)
    /// - `j`: End index (1-indexed)
    pub fn R_qsort_int_I(
        iv: *mut ::std::os::raw::c_int,
        indx: *mut ::std::os::raw::c_int,
        i: ::std::os::raw::c_int,
        j: ::std::os::raw::c_int,
    );

    /// Expand a filename, resolving `~` and environment variables.
    ///
    /// # Returns
    ///
    /// Pointer to expanded path (in R's internal buffer, do not free).
    pub fn R_ExpandFileName(s: *const ::std::os::raw::c_char) -> *const ::std::os::raw::c_char;

    /// Convert string to double, always using '.' as decimal point.
    ///
    /// Also accepts "NA" as input, returning NA_REAL.
    pub fn R_atof(str: *const ::std::os::raw::c_char) -> f64;

    /// Convert string to double with end pointer, using '.' as decimal point.
    ///
    /// Like `strtod()` but locale-independent.
    pub fn R_strtod(c: *const ::std::os::raw::c_char, end: *mut *mut ::std::os::raw::c_char)
    -> f64;

    /// Generate a temporary filename.
    ///
    /// # Parameters
    ///
    /// - `prefix`: Filename prefix
    /// - `tempdir`: Directory for temp file
    ///
    /// # Returns
    ///
    /// Newly allocated string (must be freed with `R_free_tmpnam`).
    pub fn R_tmpnam(
        prefix: *const ::std::os::raw::c_char,
        tempdir: *const ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_char;

    /// Generate a temporary filename with extension.
    ///
    /// # Parameters
    ///
    /// - `prefix`: Filename prefix
    /// - `tempdir`: Directory for temp file
    /// - `fileext`: File extension (e.g., ".txt")
    ///
    /// # Returns
    ///
    /// Newly allocated string (must be freed with `R_free_tmpnam`).
    pub fn R_tmpnam2(
        prefix: *const ::std::os::raw::c_char,
        tempdir: *const ::std::os::raw::c_char,
        fileext: *const ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_char;

    /// Free a temporary filename allocated by `R_tmpnam` or `R_tmpnam2`.
    pub fn R_free_tmpnam(name: *mut ::std::os::raw::c_char);

    /// Check for R stack overflow.
    ///
    /// Throws an R error if stack is nearly exhausted.
    pub fn R_CheckStack();

    /// Check for R stack overflow with extra space requirement.
    ///
    /// # Parameters
    ///
    /// - `extra`: Additional bytes needed
    pub fn R_CheckStack2(extra: usize);

    /// Find the interval containing a value (binary search).
    ///
    /// Used for interpolation and binning.
    ///
    /// # Parameters
    ///
    /// - `xt`: Sorted breakpoints array
    /// - `n`: Number of breakpoints
    /// - `x`: Value to find
    /// - `rightmost_closed`: If TRUE, rightmost interval is closed
    /// - `all_inside`: If TRUE, out-of-bounds values map to endpoints
    /// - `ilo`: Initial guess for interval (1-indexed)
    /// - `mflag`: Output flag (see R documentation)
    ///
    /// # Returns
    ///
    /// Interval index (1-indexed).
    pub fn findInterval(
        xt: *const f64,
        n: ::std::os::raw::c_int,
        x: f64,
        rightmost_closed: Rboolean,
        all_inside: Rboolean,
        ilo: ::std::os::raw::c_int,
        mflag: *mut ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;

    /// Extended interval finding with left-open option.
    #[allow(clippy::too_many_arguments)]
    pub fn findInterval2(
        xt: *const f64,
        n: ::std::os::raw::c_int,
        x: f64,
        rightmost_closed: Rboolean,
        all_inside: Rboolean,
        left_open: Rboolean,
        ilo: ::std::os::raw::c_int,
        mflag: *mut ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;

    /// Find column maxima in a matrix.
    ///
    /// # Parameters
    ///
    /// - `matrix`: Column-major matrix data
    /// - `nr`: Number of rows
    /// - `nc`: Number of columns
    /// - `maxes`: Output array for column maxima indices (1-indexed)
    /// - `ties_meth`: How to handle ties (1=first, 2=random, 3=last)
    pub fn R_max_col(
        matrix: *const f64,
        nr: *const ::std::os::raw::c_int,
        nc: *const ::std::os::raw::c_int,
        maxes: *mut ::std::os::raw::c_int,
        ties_meth: *const ::std::os::raw::c_int,
    );

    /// Check if a string represents FALSE in R.
    ///
    /// Recognizes "FALSE", "false", "False", "F", "f", etc.
    #[doc(alias = "Rf_StringFalse")]
    pub fn StringFalse(s: *const ::std::os::raw::c_char) -> Rboolean;

    /// Check if a string represents TRUE in R.
    ///
    /// Recognizes "TRUE", "true", "True", "T", "t", etc.
    #[doc(alias = "Rf_StringTrue")]
    pub fn StringTrue(s: *const ::std::os::raw::c_char) -> Rboolean;

    /// Check if a string is blank (empty or only whitespace).
    #[doc(alias = "Rf_isBlankString")]
    pub fn isBlankString(s: *const ::std::os::raw::c_char) -> Rboolean;
}

// endregion

// region: Additional Rinternals.h functions

#[r_ffi_checked]
unsafe extern "C-unwind" {
    // String/character functions

    /// Create a CHARSXP with specified encoding.
    ///
    /// # Parameters
    ///
    /// - `s`: C string
    /// - `encoding`: Character encoding (CE_UTF8, CE_LATIN1, etc.)
    // Issue #112 cat. 10: kept pub(crate) — 2 callers in encoding.rs; wrapping adds no value
    #[doc(alias = "mkCharCE")]
    pub(crate) fn Rf_mkCharCE(s: *const ::std::os::raw::c_char, encoding: cetype_t) -> SEXP;

    /// Get the number of characters in a string/character.
    ///
    /// # Parameters
    ///
    /// - `x`: A string SEXP
    /// - `ntype`: Type of count (0=bytes, 1=chars, 2=width)
    /// - `allowNA`: Whether to allow NA values
    /// - `keepNA`: Whether to keep NA in result
    /// - `msg_name`: Name for error messages
    ///
    /// # Returns
    ///
    /// Character count or -1 on error.
    pub fn R_nchar(
        x: SEXP,
        ntype: ::std::os::raw::c_int,
        allowNA: Rboolean,
        keepNA: Rboolean,
        msg_name: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;

    /// Convert SEXPTYPE to C string name.
    ///
    /// Returns a string like "INTSXP", "REALSXP", etc.
    #[doc(alias = "type2char")]
    pub fn Rf_type2char(sexptype: SEXPTYPE) -> *const ::std::os::raw::c_char;

    /// Print an R value to the console.
    ///
    /// Uses R's standard print method for the object.
    #[doc(alias = "PrintValue")]
    pub fn Rf_PrintValue(x: SEXP);

    // Environment functions

    /// Create a new environment.
    ///
    /// # Parameters
    ///
    /// - `enclos`: Enclosing environment
    /// - `hash`: Whether to use a hash table
    /// - `size`: Initial hash table size (if hash is TRUE)
    // Issue #112 cat. 10: kept pub(crate) — 2 callers in environment.rs; wrapping adds no value
    pub(crate) fn R_NewEnv(enclos: SEXP, hash: Rboolean, size: ::std::os::raw::c_int) -> SEXP;

    /// Check if a variable exists in an environment frame.
    ///
    /// Does not search enclosing environments.
    pub fn R_existsVarInFrame(rho: SEXP, symbol: SEXP) -> Rboolean;

    /// Remove a variable from an environment frame.
    ///
    /// # Returns
    ///
    /// The removed value, or R_NilValue if not found.
    pub fn R_removeVarFromFrame(symbol: SEXP, env: SEXP) -> SEXP;

    /// Get the top-level environment.
    ///
    /// Walks up enclosing environments until reaching a top-level env
    /// (global, namespace, or base).
    #[doc(alias = "topenv")]
    pub fn Rf_topenv(target: SEXP, envir: SEXP) -> SEXP;

    // Matching functions

    /// Match elements of first vector in second vector.
    ///
    /// Like R's `match()` function.
    ///
    /// # Parameters
    ///
    /// - `x`: Vector of values to match
    /// - `table`: Vector to match against
    /// - `nomatch`: Value to return for non-matches
    ///
    /// # Returns
    ///
    /// Integer vector of match positions (1-indexed, nomatch for non-matches).
    #[doc(alias = "match")]
    pub fn Rf_match(x: SEXP, table: SEXP, nomatch: ::std::os::raw::c_int) -> SEXP;

    // Duplication and copying

    /// Copy most attributes from source to target.
    ///
    /// Copies all attributes except names, dim, and dimnames.
    #[doc(alias = "copyMostAttrib")]
    pub fn Rf_copyMostAttrib(source: SEXP, target: SEXP);

    /// Find first duplicated element.
    ///
    /// # Parameters
    ///
    /// - `x`: Vector to search
    /// - `fromLast`: If TRUE, search from end
    ///
    /// # Returns
    ///
    /// 0 if no duplicates, otherwise 1-indexed position of first duplicate.
    #[doc(alias = "any_duplicated")]
    pub fn Rf_any_duplicated(x: SEXP, fromLast: Rboolean) -> R_xlen_t;

    // S4 functions

    /// Convert to an S4 object.
    ///
    /// # Parameters
    ///
    /// - `object`: Object to convert
    /// - `flag`: Conversion flag
    #[doc(alias = "asS4")]
    pub fn Rf_asS4(object: SEXP, flag: Rboolean, complete: ::std::os::raw::c_int) -> SEXP;

    /// Get the S3 class of an S4 object.
    #[doc(alias = "S3Class")]
    pub fn Rf_S3Class(object: SEXP) -> SEXP;

    // Option access

    /// Get an R option value.
    ///
    /// Equivalent to `getOption("name")` in R.
    ///
    /// # Parameters
    ///
    /// - `tag`: Symbol for option name
    #[doc(alias = "GetOption1")]
    pub fn Rf_GetOption1(tag: SEXP) -> SEXP;

    /// Get the `digits` option.
    ///
    /// Returns the value of `getOption("digits")`.
    #[doc(alias = "GetOptionDigits")]
    pub fn Rf_GetOptionDigits() -> ::std::os::raw::c_int;

    /// Get the `width` option.
    ///
    /// Returns the value of `getOption("width")`.
    #[doc(alias = "GetOptionWidth")]
    pub(crate) fn Rf_GetOptionWidth() -> ::std::os::raw::c_int;

    // Factor functions

    /// Check if a factor is ordered.
    #[doc(alias = "isOrdered")]
    pub fn Rf_isOrdered(s: SEXP) -> Rboolean;

    /// Check if a factor is unordered.
    #[doc(alias = "isUnordered")]
    pub fn Rf_isUnordered(s: SEXP) -> Rboolean;

    /// Check if a vector is unsorted.
    ///
    /// # Parameters
    ///
    /// - `x`: Vector to check
    /// - `strictly`: If TRUE, check for strictly increasing
    #[doc(alias = "isUnsorted")]
    pub fn Rf_isUnsorted(x: SEXP, strictly: Rboolean) -> ::std::os::raw::c_int;

    // Expression and evaluation

    /// Substitute in an expression.
    ///
    /// Like R's `substitute()` function.
    #[doc(alias = "substitute")]
    pub fn Rf_substitute(lang: SEXP, rho: SEXP) -> SEXP;

    /// Set vector length.
    ///
    /// For short vectors (length < 2^31).
    #[doc(alias = "lengthgets")]
    pub fn Rf_lengthgets(x: SEXP, newlen: R_xlen_t) -> SEXP;

    /// Set vector length (long vector version).
    #[doc(alias = "xlengthgets")]
    pub fn Rf_xlengthgets(x: SEXP, newlen: R_xlen_t) -> SEXP;

    // Protection (indexed — see cost table in the "GC protection" region above)

    /// Protect a SEXP and record its stack index for later `R_Reprotect`.
    ///
    /// **Cost: O(1)** — same array write as `Rf_protect`, plus stores the index.
    /// No allocation. Use when you need to replace a protected value in-place
    /// (e.g., inside a loop that allocates) without unprotect/re-protect churn.
    #[doc(alias = "PROTECT_WITH_INDEX")]
    pub fn R_ProtectWithIndex(s: SEXP, index: *mut ::std::os::raw::c_int);

    /// Replace the SEXP at a previously recorded protect stack index.
    ///
    /// **Cost: O(1)** — direct array write (`R_PPStack[index] = s`). No allocation.
    ///
    /// # Safety
    ///
    /// `index` must be from a previous `R_ProtectWithIndex` call and the
    /// stack must not have been unprotected past that index.
    #[doc(alias = "REPROTECT")]
    pub fn R_Reprotect(s: SEXP, index: ::std::os::raw::c_int);

    // Weak references

    /// Create a weak reference.
    ///
    /// # Parameters
    ///
    /// - `key`: The key object (weak reference target)
    /// - `val`: The value to associate
    /// - `fin`: Finalizer function (or R_NilValue)
    /// - `onexit`: Whether to run finalizer on R exit
    pub fn R_MakeWeakRef(key: SEXP, val: SEXP, fin: SEXP, onexit: Rboolean) -> SEXP;

    /// Create a weak reference with C finalizer.
    pub fn R_MakeWeakRefC(key: SEXP, val: SEXP, fin: R_CFinalizer_t, onexit: Rboolean) -> SEXP;

    /// Get the key from a weak reference.
    pub fn R_WeakRefKey(w: SEXP) -> SEXP;

    /// Get the value from a weak reference.
    pub fn R_WeakRefValue(w: SEXP) -> SEXP;

    /// Run pending finalizers.
    pub fn R_RunPendingFinalizers();

    // Conversion list/vector

    /// Convert a pairlist to a generic vector (list).
    #[doc(alias = "PairToVectorList")]
    pub fn Rf_PairToVectorList(x: SEXP) -> SEXP;

    /// Convert a generic vector (list) to a pairlist.
    #[doc(alias = "VectorToPairList")]
    pub fn Rf_VectorToPairList(x: SEXP) -> SEXP;

    // Install with CHARSXP

    /// Install a symbol from a CHARSXP.
    ///
    /// Like `Rf_install()` but takes a CHARSXP instead of C string.
    #[doc(alias = "installChar")]
    pub fn Rf_installChar(x: SEXP) -> SEXP;
}

// endregion
