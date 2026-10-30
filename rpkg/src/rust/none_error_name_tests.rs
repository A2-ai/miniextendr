//! None diagnostics use the R entry point's name (#2063).
//!
//! The fixtures that raise return `Option<()>`, which raises on `None` in
//! every path. A `None` of `Option<scalar>` is a typed `NA` instead (#1415),
//! so it would not reach the message these tests check.

use miniextendr_api::{ExternalPtr, miniextendr};

/// @noRd
#[miniextendr(noexport, r_name = "none_named_main")]
pub fn none_rust_main() -> Option<()> {
    None
}

/// @noRd
#[miniextendr(noexport, postfix = "_entry")]
pub fn none_postfix_main() -> Option<()> {
    None
}

/// @noRd
#[miniextendr(noexport, r_name = "none_named_scalar")]
pub fn none_rust_scalar() -> Option<i32> {
    None
}

#[derive(ExternalPtr)]
pub struct NoneNameLookup;

/// @noRd
#[miniextendr(env, noexport)]
impl NoneNameLookup {
    pub fn new() -> Self {
        Self
    }

    #[miniextendr(r_name = "find_value")]
    pub fn rust_find(&self) -> Option<()> {
        None
    }

    #[miniextendr(postfix = "_entry")]
    pub fn find(&self) -> Option<()> {
        None
    }
}
