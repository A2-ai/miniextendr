//! Lint rule implementations.
//!
//! Each submodule contains one or more related lint checks. All rules operate
//! on the shared [`CrateIndex`] and produce
//! [`Diagnostic`] values.

pub mod doc_pkg_links;
pub mod export_attrs;
pub mod ffi_unchecked;
pub mod fn_visibility;
pub mod impl_validation;
pub mod no_constructor;
pub mod nonapi_attrib;
pub mod r_reserved_params;
pub mod raw_r_eval;
pub mod rf_error;
pub mod s4_method_prefix;
pub mod vctrs_self_ctor;
pub mod vec_into_sexp;

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;

/// Run all lint rules against the crate index, collecting diagnostics.
pub fn run_all_rules(index: &CrateIndex) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Per-file impl validation (MXL008)
    impl_validation::check(index, &mut diagnostics);

    // Per-file: function visibility (MXL106)
    fn_visibility::check(index, &mut diagnostics);

    // Per-file: export attr redundancy (MXL203)
    export_attrs::check(index, &mut diagnostics);

    // Per-file: doc links into undeclared R packages (MXL204)
    doc_pkg_links::check(index, &mut diagnostics);

    // Per-file: R reserved words as parameter names (MXL110)
    r_reserved_params::check(index, &mut diagnostics);

    // Per-file: direct Rf_error/Rf_errorcall usage (MXL300)
    rf_error::check(index, &mut diagnostics);

    // Per-file: sys::*_unchecked() usage (MXL301)
    ffi_unchecked::check(index, &mut diagnostics);

    // Per-file: into_sexp() inside a vec!/array literal (MXL302)
    vec_into_sexp::check(index, &mut diagnostics);

    // Per-file: non-API ATTRIB / SET_ATTRIB usage (MXL304)
    nonapi_attrib::check(index, &mut diagnostics);

    // Per-file: raw R evaluation (Rf_eval, R_CheckUserInterrupt, ...) in a #[miniextendr] body (MXL305)
    raw_r_eval::check(index, &mut diagnostics);

    // Per-file: s4_* method name on #[miniextendr(s4)] impl (MXL111)
    s4_method_prefix::check(index, &mut diagnostics);

    // Per-file: vctrs ctor returns Self / instance receiver on vctrs impl (MXL120)
    vctrs_self_ctor::check(index, &mut diagnostics);

    // Crate-wide: a class R code can never create (MXL121)
    no_constructor::check(index, &mut diagnostics);

    // Sort by path and line for deterministic output
    diagnostics.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.line.cmp(&b.line))
            .then(a.code.cmp(&b.code))
    });

    diagnostics
}
