//! Miscellaneous test functions.

use miniextendr_api::ffi::SEXP;
use miniextendr_api::{miniextendr, miniextendr_module};

// Test that wildcard `_` parameters work (transformed to synthetic names internally)
#[miniextendr]
/// @title Miscellaneous Tests
/// @name rpkg_misc
/// @keywords internal
/// @description Miscellaneous test helpers
/// @examples
/// underscore_it_all(1L, 2)
/// r6_standalone_add(1L, 2L)
/// @aliases underscore_it_all r6_standalone_add
pub fn underscore_it_all(_: i32, _: f64) {}

/// @title ALTREP Helpers
/// @name rpkg_altrep_helpers
/// @keywords internal
/// @description ALTREP convenience wrappers (internal)
/// @examples
/// x <- altrep_compact_int(5L, 1L, 2L)
/// y <- altrep_from_doubles(c(1, 2, 3))
/// z <- altrep_from_strings(c("a", "b"))
/// altrep_lazy_int_seq_is_materialized(lazy_int_seq(1L, 5L, 1L))
/// @aliases altrep_compact_int altrep_from_doubles altrep_from_strings
/// @aliases altrep_from_logicals altrep_from_raw altrep_from_list
/// @aliases altrep_constant_int altrep_lazy_int_seq_is_materialized
#[miniextendr]
fn rpkg_doc_altrep_helpers() {}

// Simple SEXP return
#[miniextendr]
pub fn do_nothing() -> SEXP {
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(42) }
}

miniextendr_module! {
    mod misc_tests;

    // Wildcard parameter test
    fn underscore_it_all;
    fn rpkg_doc_altrep_helpers;
    fn do_nothing;
}
