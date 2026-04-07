//! Tests for R_compute_identical and SEXP equality semantics.

use miniextendr_api::ffi::{IDENT_USE_CLOENV, R_compute_identical, SEXP};
use miniextendr_api::miniextendr;

/// Test SEXP pointer equality vs R_compute_identical semantic equality.
/// @param x First SEXP to compare.
/// @param y Second SEXP to compare.
#[miniextendr]
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn C_test_sexp_equality(x: SEXP, y: SEXP) -> SEXP {
    use miniextendr_api::ffi::{Rboolean, SexpExt};
    use miniextendr_api::gc_protect::ProtectScope;

    let pointer_eq = x == y;
    let semantic_eq = unsafe { R_compute_identical(x, y, IDENT_USE_CLOENV) } != Rboolean::FALSE;

    unsafe {
        let scope = ProtectScope::new();
        let result = scope.alloc_vecsxp(2);
        result
            .get()
            .set_vector_elt(0, SEXP::scalar_logical(pointer_eq));
        result
            .get()
            .set_vector_elt(1, SEXP::scalar_logical(semantic_eq));

        let names = scope.alloc_strsxp(2);
        names.get().set_string_elt(0, SEXP::charsxp("pointer_eq"));
        names.get().set_string_elt(1, SEXP::charsxp("semantic_eq"));
        result.get().set_names(names.get());

        result.get()
    }
}
