//! Lint rule implementations.
//!
//! Each submodule contains one or more related lint checks. All rules operate
//! on the shared [`CrateIndex`](crate::crate_index::CrateIndex) and produce
//! [`Diagnostic`](crate::diagnostic::Diagnostic) values.

pub mod cfg_parity;
pub mod export_attrs;
pub mod ffi_unchecked;
pub mod fn_visibility;
pub mod impl_validation;
pub mod module_graph;
pub mod module_items;
pub mod rf_error;
pub mod submodule_wiring;
pub mod tag_collision;
pub mod trait_registration;
pub mod typed_external;

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;

/// Run all lint rules against the crate index, collecting diagnostics.
pub fn run_all_rules(index: &CrateIndex) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Per-file rules (existing checks + MXL101)
    module_items::check(index, &mut diagnostics);

    // Per-file impl validation (existing + MXL201)
    impl_validation::check(index, &mut diagnostics);

    // Cross-file: use child wiring (existing MXL006 + MXL202)
    submodule_wiring::check(index, &mut diagnostics);

    // Cross-file: ExternalPtr/TypedExternal (existing MXL007 + MXL102)
    typed_external::check(index, &mut diagnostics);

    // Cross-file: module graph (MXL100, MXL105, MXL204)
    module_graph::check(index, &mut diagnostics);

    // Per-file: cfg parity (MXL104)
    cfg_parity::check(index, &mut diagnostics);

    // Per-file: function visibility (MXL106)
    fn_visibility::check(index, &mut diagnostics);

    // Cross-file: trait registration (MXL103, MXL107, MXL108)
    trait_registration::check(index, &mut diagnostics);

    // Per-file: tag collision preflight (MXL200)
    tag_collision::check(index, &mut diagnostics);

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
