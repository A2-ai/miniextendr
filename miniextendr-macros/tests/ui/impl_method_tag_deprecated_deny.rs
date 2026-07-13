//! Compile-fail test for #1206: activating the impl-level method-tag nudge.
//!
//! `strip_method_tags` strips method-only roxygen tags (`@param`, `@return`,
//! `@returns`, `@examples`, `@export`) from impl-block docs and used to emit
//! an unused `#[deprecated]` const as a nudge — but an unused deprecated
//! const warns nowhere, so the nudge never fired (dead code implying a
//! feature). A sibling "use" const now reads the warn const's value, so the
//! `deprecated` lint actually fires at the impl-block span. Under
//! `#![deny(deprecated)]` that turns into a hard compile error.

#![deny(deprecated)]

use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
struct MyType;

/// @param x nope
#[miniextendr]
impl MyType {
    fn new() -> Self {
        MyType
    }
}

fn main() {}
