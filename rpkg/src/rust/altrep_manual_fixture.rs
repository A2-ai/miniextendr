//! Regression fixture proving `#[altrep(manual)]` auto-emits `impl_altinteger_from_data!`.
//!
//! With `#[altrep(manual)]`, the user supplies `AltrepLen` and `AltIntegerData`
//! impls by hand, but the derive still emits the `impl_altinteger_from_data!`
//! bridge automatically. If the derive stops emitting the bridge, this fixture
//! will fail to link (missing low-level `Altrep`/`AltVec`/`AltInteger` impls).
//!
//! See also `altrep_no_lowlevel_fixture.rs` for the `no_lowlevel` escape hatch.

use miniextendr_api::altrep_data::{AltIntegerData, AltrepLen};
use miniextendr_api::prelude::*;

// region: DoublingAltrep — manual high-level impls, auto bridge

/// ALTREP integer vector where element `i` (0-indexed) returns `i * 2`.
///
/// Uses `#[altrep(manual)]` to write `AltrepLen` and `AltIntegerData` by hand.
/// The derive emits the low-level bridge automatically — no
/// `impl_altinteger_from_data!` call needed here.
#[derive(miniextendr_api::AltrepInteger)]
#[altrep(manual, class = "DoublingAltrep")]
pub struct DoublingAltrep {
    /// Length of the vector.
    pub length: i32,
}

impl AltrepLen for DoublingAltrep {
    fn len(&self) -> usize {
        self.length.max(0) as usize
    }
}

impl AltIntegerData for DoublingAltrep {
    fn elt(&self, i: usize) -> i32 {
        (i as i32) * 2
    }
}

// endregion

// region: Exported R functions

/// Create a doubling ALTREP integer vector of given length.
///
/// Element at R index `k` (1-based) returns `(k - 1) * 2`.
/// Uses `#[altrep(manual)]` — the `impl_altinteger_from_data!` bridge is
/// emitted automatically by the derive.
///
/// @param length Length of the vector (non-negative integer).
/// @return An ALTREP-backed integer vector.
/// @export
#[miniextendr]
pub fn make_doubling_altrep(length: i32) -> DoublingAltrep {
    DoublingAltrep { length }
}

// endregion
