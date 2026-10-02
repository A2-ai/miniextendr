//! Links in leading doc-comment prose under the default
//! `roxygen_prose_links = "strip"`.
//!
//! rpkg's doc comments are written for rustdoc (`` [`Type`] `` links abound),
//! so it keeps the default: leading prose loses its link brackets, and an
//! explicit tag keeps them. `man/roxygen_prose_links_tests.Rd` pins both, and
//! `tests/testthat/test-roxygen-carry.R` reads it back. The `"keep"` side is
//! `producer_prose_links_probe` in `tests/cross-package/producer.pkg`.

use miniextendr_api::miniextendr;

/// Prose links demo.
///
/// [roxygen_prose_links_demo()] in the leading prose is plain text on the R
/// page (rustdoc still resolves it to this fn).
///
/// @title Prose links demo
/// @details An explicit tag keeps its link: see [roxygen_carry_demo()].
/// @return The number 4.
#[miniextendr]
pub fn roxygen_prose_links_demo() -> f64 {
    4.0
}
