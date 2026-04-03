//! Tests for R_compute_identical and SEXP equality semantics.

use miniextendr_api::ffi::{IDENT_USE_CLOENV, R_compute_identical, SEXP};
use miniextendr_api::miniextendr;

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn C_test_sexp_equality(x: SEXP, y: SEXP) -> SEXP {
    use miniextendr_api::ffi::{Rf_ScalarLogical, Rf_allocVector, SET_VECTOR_ELT, SEXPTYPE};

    // Pointer equality (what == does on SEXP)
    let pointer_eq = x == y;

    // Semantic equality (proper R semantics)
    // Use default flags (16 = ignore closure environments)
    use miniextendr_api::ffi::Rboolean;
    let semantic_eq = unsafe { R_compute_identical(x, y, IDENT_USE_CLOENV) } != Rboolean::FALSE;

    // Return list(pointer_eq = ..., semantic_eq = ...)
    unsafe {
        let result = Rf_allocVector(SEXPTYPE::VECSXP, 2);
        SET_VECTOR_ELT(result, 0, Rf_ScalarLogical(pointer_eq as i32));
        SET_VECTOR_ELT(result, 1, Rf_ScalarLogical(semantic_eq as i32));

        // Add names
        use miniextendr_api::ffi::{Rf_mkChar, Rf_namesgets, SET_STRING_ELT};
        let names = Rf_allocVector(SEXPTYPE::STRSXP, 2);
        SET_STRING_ELT(names, 0, Rf_mkChar(c"pointer_eq".as_ptr()));
        SET_STRING_ELT(names, 1, Rf_mkChar(c"semantic_eq".as_ptr()));
        Rf_namesgets(result, names);

        result
    }
}
