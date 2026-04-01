//! Test fixtures for encoding module.
//!
//! Note: REncodingInfo fields require `nonapi` feature. We only test
//! the basic availability check here.

use miniextendr_api::encoding;
use miniextendr_api::prelude::*;

/// Check if encoding info is available.
/// On most R package builds this returns false (encoding_init is disabled).
#[miniextendr]
pub fn encoding_info_available() -> bool {
    encoding::encoding_info().is_some()
}
