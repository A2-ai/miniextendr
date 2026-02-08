//! Tests for `#[miniextendr(internal)]` and `#[miniextendr(noexport)]` attributes.

use miniextendr_api::{miniextendr, miniextendr_module};

/// A normal exported function for comparison.
#[miniextendr]
pub fn export_control_normal() -> &'static str {
    "normal"
}

/// An internal function — callable but not exported, with @keywords internal.
#[miniextendr(internal)]
pub fn export_control_internal() -> &'static str {
    "internal"
}

/// A noexport function — callable but not exported, no @keywords internal.
#[miniextendr(noexport)]
pub fn export_control_noexport() -> &'static str {
    "noexport"
}

miniextendr_module! {
    mod export_control_tests;

    fn export_control_normal;
    fn export_control_internal;
    fn export_control_noexport;
}
