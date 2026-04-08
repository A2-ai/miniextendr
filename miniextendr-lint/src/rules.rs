//! Lint rule implementations.
//!
//! Each submodule contains one or more related lint checks. All rules operate
//! on the shared [`CrateIndex`] and produce
//! [`Diagnostic`] values.

pub mod export_attrs;
pub mod ffi_unchecked;
pub mod fn_visibility;
pub mod impl_validation;
pub mod rf_error;

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;

/// Run all lint rules against the crate index, collecting diagnostics.
pub fn run_all_rules(index: &CrateIndex) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Per-file impl validation (MXL008, MXL009, MXL010)
    impl_validation::check(index, &mut diagnostics);

    // Per-file: function visibility (MXL106)
    fn_visibility::check(index, &mut diagnostics);

    // Per-file: export attr redundancy (MXL203)
    export_attrs::check(index, &mut diagnostics);

    // Per-file: direct Rf_error/Rf_errorcall usage (MXL300)
    rf_error::check(index, &mut diagnostics);

    // Per-file: ffi::*_unchecked() usage (MXL301)
    ffi_unchecked::check(index, &mut diagnostics);

    // Sort by path and line for deterministic output
    diagnostics.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.line.cmp(&b.line))
            .then(a.code.cmp(&b.code))
    });

    diagnostics
}
