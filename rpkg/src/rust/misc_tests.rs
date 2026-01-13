//! Miscellaneous test functions.

use miniextendr_api::ffi::SEXP;
use miniextendr_api::{miniextendr, miniextendr_module};

// Test that wildcard `_` parameters work (transformed to synthetic names internally)
#[miniextendr]
/// @title Miscellaneous Tests
/// @name rpkg_misc
/// @noRd
/// @description Miscellaneous test helpers
/// @examples
/// underscore_it_all(1L, 2)
/// do_nothing()
/// @aliases underscore_it_all do_nothing
pub fn underscore_it_all(_: i32, _: f64) {}

// Simple SEXP return
/// @noRd
#[miniextendr]
pub fn do_nothing() -> SEXP {
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(42) }
}

miniextendr_module! {
    mod misc_tests;

    // Wildcard parameter test
    fn underscore_it_all;
    fn do_nothing;
}
