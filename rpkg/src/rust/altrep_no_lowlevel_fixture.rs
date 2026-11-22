//! Regression fixture proving `#[altrep(no_lowlevel)]` requires a manual
//! `impl_altinteger_from_data!` call.
//!
//! With `#[altrep(no_lowlevel)]`, the derive suppresses the automatic bridge
//! emission. The user must call `impl_altinteger_from_data!` explicitly.
//! If the call is removed from this file, the code will fail to link.
//!
//! See also `altrep_manual_fixture.rs` for the `manual` auto-bridge path.

use miniextendr_api::altrep_data::{AltIntegerData, AltrepExtract, AltrepLen};
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
    fn altrep_len(&self) -> usize {
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

// region: ALTREP with only length and element access

/// Integer ALTREP whose default R data-pointer method refuses materialization.
#[derive(miniextendr_api::AltrepInteger)]
#[altrep(no_lowlevel, manual, class = "ElementOnlyInts")]
pub struct ElementOnlyInts(Vec<i32>);

impl miniextendr_api::altrep_traits::Altrep for ElementOnlyInts {
    fn length(x: SEXP) -> miniextendr_api::R_xlen_t {
        let data = unsafe { Self::altrep_extract_ref(x) };
        miniextendr_api::R_xlen_t::try_from(data.0.len()).unwrap()
    }
}

impl miniextendr_api::altrep_traits::AltVec for ElementOnlyInts {}

impl miniextendr_api::altrep_traits::AltInteger for ElementOnlyInts {
    const HAS_ELT: bool = true;

    fn elt(x: SEXP, i: miniextendr_api::R_xlen_t) -> i32 {
        let data = unsafe { Self::altrep_extract_ref(x) };
        data.0[usize::try_from(i).unwrap()]
    }
}

miniextendr_api::impl_inferbase_integer!(ElementOnlyInts);

/// Wrap integers in a class that supports only element reads.
#[miniextendr(noexport)]
pub fn element_only_ints(values: Vec<i32>) -> ElementOnlyInts {
    ElementOnlyInts(values)
}

type ScalarInteger = i32;

/// Exercise type-directed argument conversion when the signature uses an alias.
#[miniextendr(no_coerce, noexport)]
pub fn element_only_scalar_alias(x: ScalarInteger) -> i32 {
    x
}

// endregion
