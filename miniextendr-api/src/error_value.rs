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
//! - `call`: the R call SEXP (or `NULL` if not available)
//! - class attribute: `"rust_error_value"`
//! - `__rust_error__` attribute: `TRUE`

use crate::ffi::{self, SEXP, SexpExt};

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
/// * `call` - Optional R call SEXP for error context. When `None`, uses `R_NilValue`.
pub fn make_rust_error_value(message: &str, kind: &str, call: Option<SEXP>) -> SEXP {
    unsafe {
        // Allocate a list of length 3: (error, kind, call)
        let list = ffi::Rf_allocVector(ffi::SEXPTYPE::VECSXP, 3);
        ffi::Rf_protect(list);

        // Set list element 0: error message
        let msg_cstr = std::ffi::CString::new(message)
            .unwrap_or_else(|_| std::ffi::CString::new("<invalid error message>").unwrap());
        let msg_charsxp = ffi::Rf_mkCharCE(msg_cstr.as_ptr(), ffi::CE_UTF8);
        ffi::SET_VECTOR_ELT(list, 0, SEXP::scalar_string(msg_charsxp));

        // Set list element 1: kind string
        let kind_cstr = std::ffi::CString::new(kind)
            .unwrap_or_else(|_| std::ffi::CString::new("other_rust_error").unwrap());
        let kind_charsxp = ffi::Rf_mkCharCE(kind_cstr.as_ptr(), ffi::CE_UTF8);
        ffi::SET_VECTOR_ELT(list, 1, SEXP::scalar_string(kind_charsxp));

        // Set list element 2: call SEXP
        let call_sexp = call.unwrap_or(SEXP::null());
        ffi::SET_VECTOR_ELT(list, 2, call_sexp);

        // Set names: c("error", "kind", "call")
        let names = ffi::Rf_allocVector(ffi::SEXPTYPE::STRSXP, 3);
        ffi::Rf_protect(names);
        ffi::SET_STRING_ELT(names, 0, ffi::Rf_mkCharCE(c"error".as_ptr(), ffi::CE_UTF8));
        ffi::SET_STRING_ELT(names, 1, ffi::Rf_mkCharCE(c"kind".as_ptr(), ffi::CE_UTF8));
        ffi::SET_STRING_ELT(names, 2, ffi::Rf_mkCharCE(c"call".as_ptr(), ffi::CE_UTF8));
        list.set_names(names);

        // Set class: "rust_error_value"
        let class = ffi::Rf_allocVector(ffi::SEXPTYPE::STRSXP, 1);
        ffi::Rf_protect(class);
        ffi::SET_STRING_ELT(
            class,
            0,
            ffi::Rf_mkCharCE(c"rust_error_value".as_ptr(), ffi::CE_UTF8),
        );
        list.set_class(class);

        // Set __rust_error__ attribute = TRUE (secondary marker)
        let attr_sym = ffi::Rf_install(c"__rust_error__".as_ptr());
        list.set_attr(attr_sym, SEXP::scalar_logical(true));

        ffi::Rf_unprotect(3);
        list
    }
}
