//! Tests for R_compute_identical, SEXP equality semantics, and the
//! `SexpExt::has_attributes` / `SexpExt::is_identical` helpers.

use miniextendr_api::miniextendr;
use miniextendr_api::prelude::{SEXP, SexpExt};
use miniextendr_api::sys::{IDENT_USE_CLOENV, R_compute_identical};

/// Whether an object has attributes, computed by `SexpExt::has_attributes()`.
///
/// Matches `!is.null(attributes(x))`.
/// @param x Any R object.
#[miniextendr]
pub fn sexp_has_attributes(x: SEXP) -> bool {
    x.has_attributes()
}

/// Whether two objects are identical, computed by `SexpExt::is_identical()`.
///
/// Matches `identical(x, y)` with its default arguments.
/// @param x Any R object.
/// @param y Any R object.
#[miniextendr]
pub fn sexp_is_identical(x: SEXP, y: SEXP) -> bool {
    x.is_identical(y)
}

/// Whether an object is identical to `FALSE`, computed in Rust.
///
/// Matches `identical(x, FALSE)`: `FALSE` with any attribute, a longer
/// vector, `NA` or `0L` all give `FALSE`.
/// @param x Any R object.
#[miniextendr]
pub fn sexp_is_identical_false(x: SEXP) -> bool {
    x.is_identical(SEXP::scalar_logical(false))
}

/// Test SEXP pointer equality vs R_compute_identical semantic equality.
/// @param x First SEXP to compare.
/// @param y Second SEXP to compare.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn C_test_sexp_equality(x: SEXP, y: SEXP) -> SEXP {
    use miniextendr_api::Rboolean;
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
