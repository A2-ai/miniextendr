//! Tests for R_compute_identical and SEXP equality semantics.

use miniextendr_api::ffi::{R_compute_identical, SEXP, IDENT_USE_CLOENV};
use miniextendr_api::{miniextendr, miniextendr_module};

/// @title Test SEXP Equality Semantics
/// @name unsafe_C_test_sexp_equality
/// @description Demonstrates SEXP pointer equality vs semantic equality.
/// @details
/// This function shows:
/// - Pointer equality (`==` in Rust) is fast but often wrong
/// - Semantic equality (`R_compute_identical`) compares contents (correct R semantics)
/// @param x First R object
/// @param y Second R object
/// @return List with `pointer_eq` and `semantic_eq` logical values
/// @examples
/// \dontrun{
/// unsafe_C_test_sexp_equality(c(1, 2), c(1, 2))  # pointer_eq=FALSE, semantic_eq=TRUE
/// x <- c(1, 2)
/// unsafe_C_test_sexp_equality(x, x)               # pointer_eq=TRUE, semantic_eq=TRUE
/// }
/// @export
#[miniextendr]
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn C_test_sexp_equality(x: SEXP, y: SEXP) -> SEXP {
    use miniextendr_api::ffi::{Rf_allocVector, Rf_ScalarLogical, SET_VECTOR_ELT, SEXPTYPE};

    // Pointer equality (what == does on SEXP)
    let pointer_eq = x == y;

    // Semantic equality (proper R semantics)
    // Use default flags (16 = ignore closure environments)
    use miniextendr_api::ffi::Rboolean;
    let semantic_eq = unsafe { R_compute_identical(x, y, IDENT_USE_CLOENV) } == Rboolean::TRUE;

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

miniextendr_module! {
    mod identical_tests;
    extern "C-unwind" fn C_test_sexp_equality;
}
