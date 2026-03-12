#![allow(dead_code)]

//! Helper macro and `r_ffi_checked` coverage.
//!
//! Exercises `r_ffi_checked`, `list!`, and `typed_list!` proc-macro entrypoints.

use crate::ffi::{R_xlen_t, SEXP, SEXPTYPE};
use crate::{miniextendr, r_ffi_checked};

// =============================================================================
// r_ffi_checked: value-returning and pointer-returning wrappers
// =============================================================================

#[r_ffi_checked]
unsafe extern "C-unwind" {
    pub fn Rf_allocVector(t: SEXPTYPE, n: R_xlen_t) -> SEXP;
    pub fn INTEGER(x: SEXP) -> *mut i32;
}

// =============================================================================
// list! macro usage
// =============================================================================

#[miniextendr]
pub(crate) fn cov_list_macro() -> crate::ffi::SEXP {
    use crate::IntoR;
    crate::list!(a = 1i32, b = "x").into_sexp()
}
