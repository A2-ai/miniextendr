//! Regression fixture proving `#[altrep(no_lowlevel)]` requires a manual
//! `impl_altinteger_from_data!` call.
//!
//! With `#[altrep(no_lowlevel)]`, the derive suppresses the automatic bridge
//! emission. The user must call `impl_altinteger_from_data!` explicitly.
//! If the call is removed from this file, the code will fail to link.
//!
//! See also `altrep_manual_fixture.rs` for the `manual` auto-bridge path.

use miniextendr_api::altrep_data::{AltIntegerData, AltrepLen};
use miniextendr_api::impl_altinteger_from_data;
use miniextendr_api::prelude::*;

// region: NoLowlevelAltrep — manual high-level impls + manual bridge

/// ALTREP integer vector where element `i` (0-indexed) returns `i + 100`.
///
/// Uses `#[altrep(no_lowlevel, manual)]` to suppress both auto-generated trait
/// impls and the automatic bridge. The `impl_altinteger_from_data!` call below
/// provides the bridge manually.
#[derive(miniextendr_api::AltrepInteger)]
#[altrep(no_lowlevel, manual, class = "NoLowlevelAltrep")]
pub struct NoLowlevelAltrep {
    /// Length of the vector.
    pub length: i32,
}

impl AltrepLen for NoLowlevelAltrep {
    fn len(&self) -> usize {
        self.length.max(0) as usize
    }
}

impl AltIntegerData for NoLowlevelAltrep {
    fn elt(&self, i: usize) -> i32 {
        (i as i32) + 100
    }
}

// Explicit bridge — required because `no_lowlevel` suppresses auto-emission.
impl_altinteger_from_data!(NoLowlevelAltrep);

// endregion

// region: Exported R functions

/// Create a +100 ALTREP integer vector of given length.
///
/// Element at R index `k` (1-based) returns `(k - 1) + 100`.
/// Uses `#[altrep(no_lowlevel, manual)]` — the user must call
/// `impl_altinteger_from_data!` manually (see this file).
///
/// @param length Length of the vector (non-negative integer).
/// @return An ALTREP-backed integer vector.
/// @export
#[miniextendr]
pub fn make_no_lowlevel_altrep(length: i32) -> NoLowlevelAltrep {
    NoLowlevelAltrep { length }
}

// endregion
