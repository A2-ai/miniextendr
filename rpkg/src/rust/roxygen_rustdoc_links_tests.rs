//! Rustdoc-only links in the text of an explicit tag (#1739).
//!
//! A link whose target no R package can resolve (a `crate::` / `self::` /
//! `Self::` root, or a `pkg::` part that cannot be an R package name) loses its
//! brackets in tag text too, not only in the default `"strip"` leading prose.
//! Left in, roxygen2 would warn "refers to un-installed package" during
//! `just force-document`. `man/roxygen_rustdoc_links_tests.Rd` pins the
//! result, and `tests/testthat/test-roxygen-carry.R` reads it back.

use miniextendr_api::miniextendr;

/// Rustdoc-only links demo.
///
/// @title Rustdoc-only links demo
/// @details Rust callers: see
/// [`roxygen_rustdoc_links_demo`][crate::roxygen_rustdoc_links_tests::roxygen_rustdoc_links_demo],
/// [`roxygen_rustdoc_links_tests::roxygen_rustdoc_links_demo`] and
/// [crate::roxygen_carry_tests]. An R link stays: [roxygen_prose_links_demo()].
/// @return The number 5.
#[miniextendr]
pub fn roxygen_rustdoc_links_demo() -> f64 {
    5.0
}
