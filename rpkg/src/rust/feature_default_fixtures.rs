//! Runtime fixtures for the feature-controlled `#[miniextendr]` option
//! defaults (`worker-default` / `strict-default` / `coerce-default` /
//! `r6-default` / `s7-default`) and their `no_*` opt-outs. See
//! docs/FEATURE_DEFAULTS.md.
//!
//! Compiled unconditionally: on a default build every pair behaves identically
//! (the opt-out keyword cancels a default that isn't on), which
//! tests/testthat/test-feature-defaults.R asserts. The scheduled `feature-legs`
//! CI job rebuilds with the defaults flipped on and re-runs those tests to
//! assert the pairs diverge — the only runtime coverage those features have
//! (audit A5/A10).

use miniextendr_api::miniextendr;

fn current_thread_name() -> String {
    std::thread::current()
        .name()
        .unwrap_or("<unnamed>")
        .to_string()
}

/// Name of the thread the body runs on under the build's default dispatch:
/// "miniextendr-worker" under `worker-default`, the R main thread otherwise.
#[miniextendr]
pub fn fdefault_worker_thread_name() -> String {
    current_thread_name()
}

/// Same body as `fdefault_worker_thread_name` with `no_worker`: pinned to the
/// R main thread even under `worker-default`.
#[miniextendr(no_worker)]
pub fn fdefault_no_worker_thread_name() -> String {
    current_thread_name()
}

/// i64 identity under the build's default conversion mode: rejects logical
/// and raw inputs under `strict-default`, accepts them otherwise.
/// @param x Integer-like scalar.
#[miniextendr]
pub fn fdefault_strict_i64(x: i64) -> i64 {
    x
}

/// Same as `fdefault_strict_i64` with `no_strict`: stays lenient even under
/// `strict-default`.
/// @param x Integer-like scalar.
#[miniextendr(no_strict)]
pub fn fdefault_no_strict_i64(x: i64) -> i64 {
    x
}

/// Logical identity under the build's default conversion mode: under
/// `coerce-default` the parameter converts from R's native integer type
/// (`0L`/`1L`; a logical is then rejected), otherwise it requires a logical.
/// @param x Logical scalar (integer `0L`/`1L` under `coerce-default`).
#[miniextendr]
pub fn fdefault_coerce_flag(x: bool) -> bool {
    x
}

/// Same as `fdefault_coerce_flag` with `no_coerce`: requires a logical even
/// under `coerce-default`.
/// @param x Logical scalar.
#[miniextendr(no_coerce)]
pub fn fdefault_no_coerce_flag(x: bool) -> bool {
    x
}

// region: class-system probe

/// Probe whose bare `#[miniextendr] impl` picks up the build's default class
/// system: Env by default, R6 under `r6-default`, S7 under `s7-default`.
#[derive(miniextendr_api::ExternalPtr)]
pub struct FdefaultProbe {
    value: i32,
}

/// Class-system probe generated under the build's default class system.
#[miniextendr]
impl FdefaultProbe {
    /// Creates a new probe holding `value`.
    /// @param value Integer stored in the probe.
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    /// Returns the stored value.
    pub fn probe_value(&self) -> i32 {
        self.value
    }
}

// endregion
