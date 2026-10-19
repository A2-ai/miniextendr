//! Fixtures for `reexport!` (#1910): R re-exports declared from Rust.
//!
//! `vec_ptype_abbr_percent` (`vctrs_class_example.rs`) is an S3 method for
//! vctrs's `vec_ptype_abbr()`. Re-exporting the generic lets a session that
//! attached only this package call it bare. vctrs is in `Imports`, which the
//! `importFrom()` needs. Two invocations name it: the wrappers get one block.
//! `%0%` is a name written as a string literal.

use miniextendr_api::reexport;

reexport!(vctrs::vec_ptype_abbr);

reexport!(vctrs::vec_ptype_abbr, vctrs::"%0%");
