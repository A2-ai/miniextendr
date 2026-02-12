//! Tagged error value transport for `#[miniextendr(error_in_r)]` mode.
//!
//! When `error_in_r` is enabled, Rust-origin failures (panics, `Result::Err`,
//! `Option::None`) are converted to a tagged SEXP value instead of raising an
//! R error immediately. The generated R wrapper inspects this tagged value and
//! escalates it to a proper R error condition past the Rust boundary.
//!
//! This ensures Rust destructors run cleanly before R sees the error.
//!
//! # Error value structure
//!
//! The tagged SEXP is a named list with:
//! - `error`: error message (character scalar)
//! - `kind`: error kind string (`"panic"`, `"result_err"`, `"none_err"`)
//! - class attribute: `"rust_error_value"`
//! - `__rust_error__` attribute: `TRUE`

use std::sync::atomic::{AtomicBool, Ordering};

use crate::ffi::{self, SEXP};

/// Build a tagged error-value SEXP for transport across the Rust→R boundary.
///
/// # Safety
///
/// Must be called from R's main thread (standard R API constraint).
/// The returned SEXP is unprotected — caller must protect if needed.
///
/// # Arguments
///
/// * `message` - Human-readable error message
/// * `kind` - Machine-readable error kind: `"panic"`, `"result_err"`, `"none_err"`,
///   or `"other_rust_error"`
pub fn make_rust_error_value(message: &str, kind: &str) -> SEXP {
    unsafe {
        // Allocate a list of length 2: (error, kind)
        let list = ffi::Rf_allocVector(ffi::SEXPTYPE::VECSXP, 2);
        ffi::Rf_protect(list);

        // Set list element 0: error message
        let msg_cstr = std::ffi::CString::new(message)
            .unwrap_or_else(|_| std::ffi::CString::new("<invalid error message>").unwrap());
        let msg_charsxp = ffi::Rf_mkCharCE(msg_cstr.as_ptr(), ffi::CE_UTF8);
        let msg_strsxp = ffi::Rf_ScalarString(msg_charsxp);
        ffi::SET_VECTOR_ELT(list, 0, msg_strsxp);

        // Set list element 1: kind string
        let kind_cstr = std::ffi::CString::new(kind)
            .unwrap_or_else(|_| std::ffi::CString::new("other_rust_error").unwrap());
        let kind_charsxp = ffi::Rf_mkCharCE(kind_cstr.as_ptr(), ffi::CE_UTF8);
        let kind_strsxp = ffi::Rf_ScalarString(kind_charsxp);
        ffi::SET_VECTOR_ELT(list, 1, kind_strsxp);

        // Set names: c("error", "kind")
        let names = ffi::Rf_allocVector(ffi::SEXPTYPE::STRSXP, 2);
        ffi::Rf_protect(names);
        ffi::SET_STRING_ELT(names, 0, ffi::Rf_mkCharCE(c"error".as_ptr(), ffi::CE_UTF8));
        ffi::SET_STRING_ELT(names, 1, ffi::Rf_mkCharCE(c"kind".as_ptr(), ffi::CE_UTF8));
        ffi::Rf_setAttrib(list, ffi::R_NamesSymbol, names);

        // Set class: "rust_error_value"
        let class = ffi::Rf_allocVector(ffi::SEXPTYPE::STRSXP, 1);
        ffi::Rf_protect(class);
        ffi::SET_STRING_ELT(
            class,
            0,
            ffi::Rf_mkCharCE(c"rust_error_value".as_ptr(), ffi::CE_UTF8),
        );
        ffi::Rf_setAttrib(list, ffi::R_ClassSymbol, class);

        // Set __rust_error__ attribute = TRUE (secondary marker)
        let attr_sym = ffi::Rf_install(c"__rust_error__".as_ptr());
        ffi::Rf_setAttrib(list, attr_sym, ffi::Rf_ScalarLogical(1));

        ffi::Rf_unprotect(3);
        list
    }
}

// =============================================================================
// R-origin error boundary crossing detection
// =============================================================================

/// Process-global flag: set when an R-origin error (longjmp) is detected
/// crossing a Rust protection boundary.
static R_ERROR_CROSSED_RUST_BOUNDARY: AtomicBool = AtomicBool::new(false);

/// One-shot guard: once the warning has been emitted, don't emit again.
static R_ERROR_WARNING_EMITTED: AtomicBool = AtomicBool::new(false);

/// Mark that an R-origin error crossed a Rust protection boundary.
///
/// Called from `unwind_protect` cleanup handler and `worker` cleanup handler
/// when an R longjmp is detected during Rust execution.
pub fn mark_r_error_crossed_rust_boundary() {
    R_ERROR_CROSSED_RUST_BOUNDARY.store(true, Ordering::Release);
}

/// Check and reset the R-error-crossed-boundary flag.
///
/// Returns `true` if an R error crossed a Rust boundary since the last check.
/// The flag is atomically reset to `false`.
pub fn take_r_error_crossed_rust_boundary_flag() -> bool {
    R_ERROR_CROSSED_RUST_BOUNDARY.swap(false, Ordering::AcqRel)
}

/// Check if the one-shot warning should be emitted.
///
/// Returns `true` exactly once per process: when `take_r_error_crossed_rust_boundary_flag()`
/// returns true AND the warning hasn't been emitted yet.
pub fn should_emit_r_error_boundary_warning() -> bool {
    if take_r_error_crossed_rust_boundary_flag()
        && !R_ERROR_WARNING_EMITTED.swap(true, Ordering::AcqRel)
    {
        return true;
    }
    false
}

/// C-callable function for generated R wrappers to check the boundary warning.
///
/// Returns `TRUE` (1) exactly once when warning should be emitted, `FALSE` (0) otherwise.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn miniextendr_check_r_error_boundary() -> ffi::SEXP {
    let should_warn = should_emit_r_error_boundary_warning();
    unsafe { ffi::Rf_ScalarLogical(if should_warn { 1 } else { 0 }) }
}
