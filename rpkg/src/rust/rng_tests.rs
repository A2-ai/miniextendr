//! RNG (Random Number Generation) tests.
//!
//! Tests for:
//! - `#[miniextendr(rng)]` attribute on standalone functions
//! - `#[miniextendr(rng)]` attribute on impl methods
//! - Manual RNG management with `RngGuard` and `with_rng`

use miniextendr_api::ffi::{R_unif_index, exp_rand, norm_rand, unif_rand};
use miniextendr_api::miniextendr;
use miniextendr_api::rng::{RngGuard, with_rng};

// region: Standalone function tests

/// Test generating n uniform random numbers using R's RNG.
/// @param n Integer number of values to generate.
#[miniextendr(rng)]
pub fn rng_uniform(n: i32) -> Vec<f64> {
    (0..n).map(|_| unsafe { unif_rand() }).collect()
}

/// Test generating n standard normal random numbers using R's RNG.
/// @param n Integer number of values to generate.
#[miniextendr(rng)]
pub fn rng_normal(n: i32) -> Vec<f64> {
    (0..n).map(|_| unsafe { norm_rand() }).collect()
}

/// Test generating n exponential random numbers using R's RNG.
/// @param n Integer number of values to generate.
#[miniextendr(rng)]
pub fn rng_exponential(n: i32) -> Vec<f64> {
    (0..n).map(|_| unsafe { exp_rand() }).collect()
}

/// Test generating n random integers in [0, max) using R's RNG.
/// @param n Integer number of values to generate.
/// @param max Numeric upper bound (exclusive) for the random integers.
#[miniextendr(rng)]
pub fn rng_int(n: i32, max: f64) -> Vec<i32> {
    (0..n)
        .map(|_| unsafe { R_unif_index(max) } as i32)
        .collect()
}

/// Test RNG with interrupt checking enabled.
/// @param n Integer number of uniform values to generate.
#[miniextendr(rng, check_interrupt)]
pub fn rng_with_interrupt(n: i32) -> Vec<f64> {
    (0..n).map(|_| unsafe { unif_rand() }).collect()
}

/// Test RNG on worker thread (requires worker-thread feature).
/// @param n Integer number of uniform values to generate.
#[cfg(feature = "worker-thread")]
#[miniextendr(rng, worker)]
pub fn rng_worker_uniform(n: i32) -> Vec<f64> {
    (0..n).map(|_| unsafe { unif_rand() }).collect()
}
// endregion

// region: Manual RNG management tests

/// Test manual RNG management using RngGuard RAII pattern.
/// @param n Integer number of uniform values to generate.
#[miniextendr]
pub fn rng_guard_test(n: i32) -> Vec<f64> {
    let _guard = RngGuard::new();
    (0..n).map(|_| unsafe { unif_rand() }).collect()
}

/// Test manual RNG management using the with_rng closure API.
/// @param n Integer number of uniform values to generate.
#[miniextendr]
pub fn rng_with_rng_test(n: i32) -> Vec<f64> {
    with_rng(|| (0..n).map(|_| unsafe { unif_rand() }).collect())
}
// endregion

// region: Impl method tests

/// A struct to test RNG in impl methods.
/// @rdname rpkg_rng
/// @export
#[derive(Clone, miniextendr_api::ExternalPtr)]
pub struct RngSampler {
    seed_hint: i32,
}

#[miniextendr]
impl RngSampler {
    /// Create a new RngSampler.
    fn new(seed_hint: i32) -> Self {
        Self { seed_hint }
    }

    /// Get the seed hint (for testing).
    fn seed_hint(&self) -> i32 {
        self.seed_hint
    }

    /// Sample n uniform values using the rng attribute on a method.
    #[miniextendr(rng)]
    fn sample_uniform(&self, n: i32) -> Vec<f64> {
        (0..n).map(|_| unsafe { unif_rand() }).collect()
    }

    /// Sample n normal values.
    #[miniextendr(rng)]
    fn sample_normal(&self, n: i32) -> Vec<f64> {
        (0..n).map(|_| unsafe { norm_rand() }).collect()
    }

    /// Static method with rng.
    #[miniextendr(rng)]
    fn static_sample(n: i32) -> Vec<f64> {
        (0..n).map(|_| unsafe { unif_rand() }).collect()
    }
}
// endregion

// region: Module registration
// endregion
