//! Tagged error value transport for `#[miniextendr(error_in_r)]` mode.
//!
//! When `error_in_r` is enabled, Rust-origin failures (panics, `Result::Err`,
//! `Option::None`) are converted to a tagged SEXP value instead of raising an
//! R error immediately. The generated R wrapper inspects this tagged value and
//! escalates it to a proper R error condition past the Rust boundary.
//!
//! This ensures Rust destructors run cleanly before R sees the error.
//!
//! # Condition value structure (`make_rust_condition_value`)
//!
//! The tagged SEXP is a 4-element named list:
//! - `error`: error message (character scalar)
//! - `kind`: condition kind string (`"panic"`, `"result_err"`, `"none_err"`,
//!   `"conversion"`, `"error"`, `"warning"`, `"message"`, `"condition"`)
//! - `class`: optional user-supplied custom class (character scalar or `NULL`)
//! - `call`: the R call SEXP (or `NULL` if not available)
//! - class attribute: `"rust_condition_value"`
//! - `__rust_condition__` attribute: `TRUE`

use crate::cached_class::{
    condition_names_sexp, rust_condition_attr_symbol, rust_condition_class_sexp,
};
use crate::ffi::{self, SEXP, SexpExt};

/// Convert a `&str` to a `CString`, falling back to `fallback` on interior NUL bytes.
///
/// Used internally by [`make_rust_condition_value`] to avoid duplicating the
/// `CString::new(s).unwrap_or_else(…)` pattern across every slot.
fn to_cstring_lossy(s: &str, fallback: &str) -> std::ffi::CString {
    std::ffi::CString::new(s).unwrap_or_else(|_| std::ffi::CString::new(fallback).unwrap())
}

/// Build a tagged condition-value SEXP for transport across the Rust→R boundary.
///
/// Used for all Rust-origin failures and user-facing conditions. The R-side
/// switch in `error_in_r_check_lines` reads `.val$kind` to select the condition
/// type and `.val$class` to prepend optional user classes before the standard
/// `rust_*` layering.
///
/// # Safety
///
/// Must be called from R's main thread (standard R API constraint).
/// The returned SEXP is unprotected — caller must protect if needed.
///
/// # PROTECT discipline
///
/// Every fresh allocation (msg, kind, optional class, true-marker) is protected
/// before the next allocation that might trigger a GC barrier. The `prot` counter
/// is incremented on each `Rf_protect` and balanced by `Rf_unprotect(prot)` at
/// exit on all branches. This pattern was established by PR #344 commit `af6b4875`
/// to fix a `recursive gc invocation` segfault on R-devel.
///
/// # Arguments
///
/// * `message` - Human-readable condition message
/// * `kind` - Condition kind: `"panic"`, `"result_err"`, `"none_err"`,
///   `"conversion"`, `"error"`, `"warning"`, `"message"`, or `"condition"`
/// * `class` - Optional user-supplied class name to prepend to the layered vector
/// * `call` - Optional R call SEXP for error context. When `None`, uses `R_NilValue`.
pub fn make_rust_condition_value(
    message: &str,
    kind: &str,
    class: Option<&str>,
    call: Option<SEXP>,
) -> SEXP {
    unsafe {
        // PROTECT discipline: every fresh allocation that's live across another
        // allocation must be protected. SET_VECTOR_ELT and SETATTRIB can both
        // trigger old-to-new GC barriers; R-devel's GC fires more aggressively
        // here than R-release/oldrel, so unprotected intermediates corrupt the
        // heap on R-devel even when R 4.5/4.4 happen to survive (PR #344 fix).
        let list = ffi::Rf_allocVector(ffi::SEXPTYPE::VECSXP, 4);
        ffi::Rf_protect(list);
        let mut prot = 1;

        // Element 0: error message
        let msg_cstr = to_cstring_lossy(message, "<invalid error message>");
        let msg_charsxp = ffi::Rf_mkCharCE(msg_cstr.as_ptr(), ffi::CE_UTF8);
        let msg_sexp = SEXP::scalar_string(msg_charsxp);
        ffi::Rf_protect(msg_sexp);
        prot += 1;
        list.set_vector_elt(0, msg_sexp);

        // Element 1: kind string
        let kind_cstr = to_cstring_lossy(kind, "other_rust_error");
        let kind_charsxp = ffi::Rf_mkCharCE(kind_cstr.as_ptr(), ffi::CE_UTF8);
        let kind_sexp = SEXP::scalar_string(kind_charsxp);
        ffi::Rf_protect(kind_sexp);
        prot += 1;
        list.set_vector_elt(1, kind_sexp);

        // Element 2: optional custom class (NULL when not provided).
        // Only the Some-branch allocates; nil is constant.
        let class_sexp = if let Some(class_name) = class {
            let class_cstr = to_cstring_lossy(class_name, "rust_condition");
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

        // Names / class symbols are cached. The TRUE marker on set_attr is a
        // fresh LGLSXP — protect across the SETATTRIB call.
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
