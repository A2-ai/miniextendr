//! Sidecar accessor panic-path coverage.
//!
//! The sidecar getter/setter C wrappers emitted by `externalptr_derive.rs` are
//! hand-rolled with `numArgs = 1` (getter) or `numArgs = 2` (setter) and do
//! **not** carry the `__miniextendr_call` slot that `c_wrapper_builder.rs`
//! prepends for every other `#[miniextendr]` fn/method. So when a panic
//! travels through the sidecar accessor path, `.val$call` is NULL and the R
//! helper falls back to `sys.call()` of the enclosing R wrapper.
//!
//! This module pairs a sidecar-bearing R6 class (`PanickingSidecar`) with two
//! complementary exercise points:
//!
//! 1. An instance method (`boom()`) that panics — exercises the normal
//!    `#[miniextendr]` path on a class that **also** has sidecar fields.
//!    `match.call()` is attached as usual; `conditionCall` reflects the R6
//!    active-binding lambda or NULL.
//! 2. A standalone fn (`sidecar_consumer_panic`) that accepts the sidecar
//!    object as `ExternalPtr<PanickingSidecar>` and panics — exercises the
//!    standalone-fn path where the caller carries full `match.call()`
//!    attribution.
//!
//! Tests live in `rpkg/tests/testthat/test-conditions-comprehensive.R`.

use miniextendr_api::externalptr::{ExternalPtr, RSidecar};
use miniextendr_api::miniextendr;

// region: sidecar-bearing fixture

/// R6 fixture with a string sidecar field that an instance method reads before
/// panicking. The sidecar field accessors are auto-generated as active bindings;
/// the `boom()` method follows the standard `#[miniextendr]` path.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(r6)]
pub struct PanickingSidecar {
    /// Selector — enables sidecar accessors.
    #[r_data]
    _r: RSidecar,

    /// Message string the panicking method will surface.
    /// @field doom Character sidecar field (active binding).
    #[r_data]
    pub doom: String,
}

/// R6 class registration with `r_data_accessors` so `obj$doom` is reachable.
/// @field doom Character sidecar field surfaced via active binding.
#[miniextendr(r6(r_data_accessors))]
impl PanickingSidecar {
    /// Construct a sidecar-bearing R6 that will panic on `$boom()`.
    /// @param doom Message string surfaced when `boom()` panics.
    pub fn new(doom: String) -> Self {
        Self {
            _r: RSidecar,
            doom,
        }
    }

    /// Always panics with the stored `doom` message. Exercises the
    /// `#[miniextendr]` method path on a sidecar-bearing class.
    pub fn boom(&self) {
        miniextendr_api::error!("{}", self.doom);
    }
}

// endregion

// region: standalone consumer

/// Low-level constructor returning the raw `ExternalPtr<PanickingSidecar>`
/// (bypasses the R6 wrapper). The R6 `$new()` constructor wraps this pointer
/// inside an environment with active bindings; tests that need to pass the
/// bare externalptr into a standalone consumer can use this helper.
///
/// @param doom Message string surfaced when `boom()` panics.
#[miniextendr]
pub fn panicking_sidecar_new(doom: String) -> ExternalPtr<PanickingSidecar> {
    ExternalPtr::new(PanickingSidecar {
        _r: RSidecar,
        doom,
    })
}

/// Standalone fn that accepts the sidecar object and panics. Exercises the
/// standalone-fn path: the R wrapper carries `.call = match.call()` so
/// `conditionCall(e)` is non-NULL and references the wrapper.
///
/// @param x A `PanickingSidecar` instance (unused; just exercises argument
///   conversion through `ExternalPtr<PanickingSidecar>`).
#[miniextendr]
pub fn sidecar_consumer_panic(_x: ExternalPtr<PanickingSidecar>) {
    miniextendr_api::error!("consumer boom");
}

// endregion
