//! Tagged error value transport for `#[miniextendr(error_in_r)]` mode.
//!
//! When `error_in_r` is enabled, Rust-origin failures (panics, `Result::Err`,
//! `Option::None`) are converted to a tagged SEXP value instead of raising an
//! R error immediately. The generated R wrapper inspects this tagged value and
//! escalates it to a proper R error condition past the Rust boundary.
//!
//! This ensures Rust destructors run cleanly before R sees the error.
//!
//! # Error value structure (legacy — `make_rust_error_value`)
//!
//! The tagged SEXP is a named list with:
//! - `error`: error message (character scalar)
//! - `kind`: error kind string (`"panic"`, `"result_err"`, `"none_err"`)
//! - `call`: the R call SEXP (or `NULL` if not available)
//! - class attribute: `"rust_error_value"`
//! - `__rust_error__` attribute: `TRUE`
//!
//! # Condition value structure (`make_rust_condition_value`)
//!
//! The tagged SEXP is a 4-element named list:
//! - `error`: error message (character scalar)
//! - `kind`: condition kind string (`"error"`, `"warning"`, `"message"`, `"condition"`)
//! - `class`: optional user-supplied custom class (character scalar or `NULL`)
//! - `call`: the R call SEXP (or `NULL` if not available)
//! - class attribute: `"rust_error_value"`
//! - `__rust_error__` attribute: `TRUE`
//!
//! The 3-element legacy form produces `NULL` when accessed as `$class`, so the
//! R-side switch is compatible with both forms.

use crate::cached_class::{
    condition_names_sexp, error_names_sexp, rust_error_attr_symbol, rust_error_class_sexp,
};
use crate::ffi::{self, SEXP, SexpExt};

/// Build a tagged error-value SEXP for transport across the Rust→R boundary.
///
/// Used for legacy error kinds (`"panic"`, `"result_err"`, `"none_err"`).
/// For user-facing conditions raised by `error!()` / `warning!()` / etc., use
/// [`make_rust_condition_value`] which carries the optional custom class.
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
        list.set_vector_elt(0, SEXP::scalar_string(msg_charsxp));

        // Set list element 1: kind string
        let kind_cstr = std::ffi::CString::new(kind)
            .unwrap_or_else(|_| std::ffi::CString::new("other_rust_error").unwrap());
        let kind_charsxp = ffi::Rf_mkCharCE(kind_cstr.as_ptr(), ffi::CE_UTF8);
        list.set_vector_elt(1, SEXP::scalar_string(kind_charsxp));

        // Set list element 2: call SEXP
        let call_sexp = call.unwrap_or(SEXP::nil());
        list.set_vector_elt(2, call_sexp);

        // Names, class, and attribute symbol are all cached — zero allocation
        list.set_names(error_names_sexp());
        list.set_class(rust_error_class_sexp());
        list.set_attr(rust_error_attr_symbol(), SEXP::scalar_logical(true));

        ffi::Rf_unprotect(1);
        list
    }
}

/// Build a tagged condition-value SEXP for user-facing conditions raised via
/// `error!()`, `warning!()`, `message!()`, or `condition!()` macros.
///
/// Similar to [`make_rust_error_value`] but writes a 4-element list that includes
/// an optional custom class (from `error!(class = "my_error", "...")`). The R-side
/// switch in `error_in_r_check_lines` reads `.val$class` to prepend user classes
/// before the standard `rust_*` layering.
///
/// # Safety
///
/// Must be called from R's main thread (standard R API constraint).
/// The returned SEXP is unprotected — caller must protect if needed.
///
/// # Arguments
///
/// * `message` - Human-readable condition message
/// * `kind` - Condition kind: `"error"`, `"warning"`, `"message"`, or `"condition"`
/// * `class` - Optional user-supplied class name to prepend to the layered vector
/// * `call` - Optional R call SEXP for error context. When `None`, uses `R_NilValue`.
pub fn make_rust_condition_value(
    message: &str,
    kind: &str,
    class: Option<&str>,
    call: Option<SEXP>,
) -> SEXP {
    unsafe {
        // Allocate a list of length 4: (error, kind, class, call)
        let list = ffi::Rf_allocVector(ffi::SEXPTYPE::VECSXP, 4);
        ffi::Rf_protect(list);

        // Set list element 0: error message
        let msg_cstr = std::ffi::CString::new(message)
            .unwrap_or_else(|_| std::ffi::CString::new("<invalid error message>").unwrap());
        let msg_charsxp = ffi::Rf_mkCharCE(msg_cstr.as_ptr(), ffi::CE_UTF8);
        list.set_vector_elt(0, SEXP::scalar_string(msg_charsxp));

        // Set list element 1: kind string
        let kind_cstr = std::ffi::CString::new(kind)
            .unwrap_or_else(|_| std::ffi::CString::new("other_rust_error").unwrap());
        let kind_charsxp = ffi::Rf_mkCharCE(kind_cstr.as_ptr(), ffi::CE_UTF8);
        list.set_vector_elt(1, SEXP::scalar_string(kind_charsxp));

        // Set list element 2: optional custom class (NULL when not provided)
        let class_sexp = if let Some(class_name) = class {
            let class_cstr = std::ffi::CString::new(class_name)
                .unwrap_or_else(|_| std::ffi::CString::new("rust_condition").unwrap());
            let class_charsxp = ffi::Rf_mkCharCE(class_cstr.as_ptr(), ffi::CE_UTF8);
            SEXP::scalar_string(class_charsxp)
        } else {
            SEXP::nil()
        };
        list.set_vector_elt(2, class_sexp);

        // Set list element 3: call SEXP
        let call_sexp = call.unwrap_or(SEXP::nil());
        list.set_vector_elt(3, call_sexp);

        // Names, class, and attribute symbol are all cached — zero allocation
        list.set_names(condition_names_sexp());
        list.set_class(rust_error_class_sexp());
        list.set_attr(rust_error_attr_symbol(), SEXP::scalar_logical(true));

        ffi::Rf_unprotect(1);
        list
    }
}
