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
//! - class attribute: `"rust_condition_value"`
//! - `__rust_condition__` attribute: `TRUE`
//!
//! # Condition value structure (`make_rust_condition_value`)
//!
//! The tagged SEXP is a 4-element named list:
//! - `error`: error message (character scalar)
//! - `kind`: condition kind string (`"error"`, `"warning"`, `"message"`, `"condition"`)
//! - `class`: optional user-supplied custom class (character scalar or `NULL`)
//! - `call`: the R call SEXP (or `NULL` if not available)
//! - class attribute: `"rust_condition_value"`
//! - `__rust_condition__` attribute: `TRUE`
//!
//! The 3-element legacy form produces `NULL` when accessed as `$class`, so the
//! R-side switch is compatible with both forms.

use crate::cached_class::{
    condition_names_sexp, error_names_sexp, rust_condition_attr_symbol, rust_condition_class_sexp,
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
        // PROTECT discipline: every fresh allocation that's live across another
        // allocation must be protected. SET_VECTOR_ELT and SETATTRIB can both
        // trigger old-to-new GC barriers; R-devel's GC fires more aggressively
        // here than R-release/oldrel, so unprotected intermediates corrupt the
        // heap on R-devel even when R 4.5/4.4 happen to survive (PR #344 fix).
        let list = ffi::Rf_allocVector(ffi::SEXPTYPE::VECSXP, 3);
        ffi::Rf_protect(list);
        let mut prot = 1;

        // Element 0: error message (scalar_string allocates a fresh STRSXP)
        let msg_cstr = std::ffi::CString::new(message)
            .unwrap_or_else(|_| std::ffi::CString::new("<invalid error message>").unwrap());
        let msg_charsxp = ffi::Rf_mkCharCE(msg_cstr.as_ptr(), ffi::CE_UTF8);
        let msg_sexp = SEXP::scalar_string(msg_charsxp);
        ffi::Rf_protect(msg_sexp);
        prot += 1;
        list.set_vector_elt(0, msg_sexp);

        // Element 1: kind string
        let kind_cstr = std::ffi::CString::new(kind)
            .unwrap_or_else(|_| std::ffi::CString::new("other_rust_error").unwrap());
        let kind_charsxp = ffi::Rf_mkCharCE(kind_cstr.as_ptr(), ffi::CE_UTF8);
        let kind_sexp = SEXP::scalar_string(kind_charsxp);
        ffi::Rf_protect(kind_sexp);
        prot += 1;
        list.set_vector_elt(1, kind_sexp);

        // Element 2: caller-owned SEXP — already protected by the caller (or R_NilValue)
        list.set_vector_elt(2, call.unwrap_or(SEXP::nil()));

        // Names / class symbols are cached. The TRUE marker on set_attr is a
        // fresh LGLSXP — protect across the SETATTRIB call.
        list.set_names(error_names_sexp());
        list.set_class(rust_condition_class_sexp());
        let true_marker = SEXP::scalar_logical(true);
        ffi::Rf_protect(true_marker);
        prot += 1;
        list.set_attr(rust_condition_attr_symbol(), true_marker);

        ffi::Rf_unprotect(prot);
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
        // See `make_rust_error_value` for the PROTECT-discipline rationale.
        let list = ffi::Rf_allocVector(ffi::SEXPTYPE::VECSXP, 4);
        ffi::Rf_protect(list);
        let mut prot = 1;

        // Element 0: error message
        let msg_cstr = std::ffi::CString::new(message)
            .unwrap_or_else(|_| std::ffi::CString::new("<invalid error message>").unwrap());
        let msg_charsxp = ffi::Rf_mkCharCE(msg_cstr.as_ptr(), ffi::CE_UTF8);
        let msg_sexp = SEXP::scalar_string(msg_charsxp);
        ffi::Rf_protect(msg_sexp);
        prot += 1;
        list.set_vector_elt(0, msg_sexp);

        // Element 1: kind string
        let kind_cstr = std::ffi::CString::new(kind)
            .unwrap_or_else(|_| std::ffi::CString::new("other_rust_error").unwrap());
        let kind_charsxp = ffi::Rf_mkCharCE(kind_cstr.as_ptr(), ffi::CE_UTF8);
        let kind_sexp = SEXP::scalar_string(kind_charsxp);
        ffi::Rf_protect(kind_sexp);
        prot += 1;
        list.set_vector_elt(1, kind_sexp);

        // Element 2: optional custom class (NULL when not provided).
        // Only the Some-branch allocates; nil is constant.
        let class_sexp = if let Some(class_name) = class {
            let class_cstr = std::ffi::CString::new(class_name)
                .unwrap_or_else(|_| std::ffi::CString::new("rust_condition").unwrap());
            let class_charsxp = ffi::Rf_mkCharCE(class_cstr.as_ptr(), ffi::CE_UTF8);
            let s = SEXP::scalar_string(class_charsxp);
            ffi::Rf_protect(s);
            prot += 1;
            s
        } else {
            SEXP::nil()
        };
        list.set_vector_elt(2, class_sexp);

        // Element 3: caller-owned SEXP — already protected (or R_NilValue)
        list.set_vector_elt(3, call.unwrap_or(SEXP::nil()));

        list.set_names(condition_names_sexp());
        list.set_class(rust_condition_class_sexp());
        let true_marker = SEXP::scalar_logical(true);
        ffi::Rf_protect(true_marker);
        prot += 1;
        list.set_attr(rust_condition_attr_symbol(), true_marker);

        ffi::Rf_unprotect(prot);
        list
    }
}
