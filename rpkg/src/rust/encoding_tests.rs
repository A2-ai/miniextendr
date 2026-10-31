//! Test fixtures for the encoding snapshot (`miniextendr-api/src/encoding.rs`).
//!
//! String conversions in any locale are exercised through the conversion
//! fixtures (`conv_string_arg`, `zero_copy_cow_str_is_borrowed`, ...) by
//! `tests/testthat/test-encoding.R`.

use miniextendr_api::encoding;
use miniextendr_api::prelude::*;

/// Whether `miniextendr_encoding_init()` populated the cached snapshot.
///
/// On standard R-package builds this returns `FALSE` (encoding init is
/// disabled because the underlying symbols aren't exported from libR).
/// Embedded R builds with the `nonapi` feature can flip this to `TRUE`.
#[miniextendr]
pub fn encoding_info_available() -> bool {
    encoding::encoding_info().is_some()
}
