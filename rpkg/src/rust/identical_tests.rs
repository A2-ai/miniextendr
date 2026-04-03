//! Tests for R_compute_identical and SEXP equality semantics.

use miniextendr_api::ffi::{IDENT_USE_CLOENV, R_compute_identical, SEXP};
use miniextendr_api::miniextendr;

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn C_test_sexp_equality(x: SEXP, y: SEXP) -> SEXP {
    use miniextendr_api::ffi::{Rf_allocVector, SEXPTYPE, SexpExt};

    // Pointer equality (what == does on SEXP)
    let pointer_eq = x == y;

    // Semantic equality (proper R semantics)
    // Use default flags (16 = ignore closure environments)
    use miniextendr_api::ffi::Rboolean;
    let semantic_eq = unsafe { R_compute_identical(x, y, IDENT_USE_CLOENV) } != Rboolean::FALSE;

    // Return list(pointer_eq = ..., semantic_eq = ...)
    unsafe {
        let result = Rf_allocVector(SEXPTYPE::VECSXP, 2);
        result.set_vector_elt(0, SEXP::scalar_logical(pointer_eq));
        result.set_vector_elt(1, SEXP::scalar_logical(semantic_eq));

        // Add names
        let names = Rf_allocVector(SEXPTYPE::STRSXP, 2);
        names.set_string_elt(0, SEXP::charsxp("pointer_eq"));
        names.set_string_elt(1, SEXP::charsxp("semantic_eq"));
        result.set_names(names);

        result
    }
}
