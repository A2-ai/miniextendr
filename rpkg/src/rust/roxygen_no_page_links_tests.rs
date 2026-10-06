//! Links in a block that renders no help page (#1818).
//!
//! A `noexport` fn's doc comment reaches its wrapper under `@noRd`, so no
//! link in it can render. Every `[...]` link there loses its brackets, in the
//! leading prose and in the text of an explicit tag alike, whatever the
//! crate's `roxygen_prose_links` setting. Left in, the plain rustdoc link
//! `` [`no_page_takes_scope`] `` below would make roxygen2 warn "Could not
//! resolve link to topic" on every `just force-document`.
//! `tests/testthat/test-roxygen-carry.R` reads the wrapper block back.

use miniextendr_api::miniextendr;

/// The verbs that take a scope (see [`no_page_takes_scope`]).
///
/// @return The verbs [`no_page_takes_scope`] accepts.
/// @details Each verb is checked by [`no_page_takes_scope`], a Rust helper R
///   never sees.
#[miniextendr(noexport)]
fn no_page_links_scoped_verbs() -> Vec<String> {
    ["filter", "mutate", "select"]
        .into_iter()
        .filter(|verb| no_page_takes_scope(verb))
        .map(String::from)
        .collect()
}

/// Whether `verb` takes a scope.
fn no_page_takes_scope(verb: &str) -> bool {
    verb != "select"
}
