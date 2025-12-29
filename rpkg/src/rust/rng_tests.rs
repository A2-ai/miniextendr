//! RNG (Random Number Generation) tests.
//!
//! Tests for:
//! - `#[miniextendr(rng)]` attribute on standalone functions
//! - `#[miniextendr(rng)]` attribute on impl methods
//! - `#[miniextendr(rng)]` attribute on trait methods
//! - Manual RNG management with `RngGuard` and `with_rng`

use miniextendr_api::ffi::{unif_rand, norm_rand, exp_rand, R_unif_index, SEXP};
use miniextendr_api::rng::{with_rng, RngGuard};
use miniextendr_api::{miniextendr, miniextendr_module};

// =============================================================================
// Standalone function tests
// =============================================================================

/// Generate n uniform random numbers using the `#[miniextendr(rng)]` attribute.
/// This tests automatic GetRNGstate/PutRNGstate wrapping.
/// @title RNG Tests
/// @name rpkg_rng
/// @description RNG state management tests
/// @return A numeric vector.
/// @examples
/// set.seed(42)
/// rng_uniform(5L)
/// @param n Number of values to generate.
#[miniextendr(rng)]
fn rng_uniform(n: i32) -> Vec<f64> {
    (0..n).map(|_| unsafe { unif_rand() }).collect()
}

/// Generate n normal random numbers.
/// @rdname rpkg_rng
#[miniextendr(rng)]
fn rng_normal(n: i32) -> Vec<f64> {
    (0..n).map(|_| unsafe { norm_rand() }).collect()
}

/// Generate n exponential random numbers.
/// @rdname rpkg_rng
#[miniextendr(rng)]
fn rng_exponential(n: i32) -> Vec<f64> {
    (0..n).map(|_| unsafe { exp_rand() }).collect()
}

/// Generate n random integers in [0, max).
/// @rdname rpkg_rng
/// @param max Upper bound (exclusive).
#[miniextendr(rng)]
fn rng_int(n: i32, max: f64) -> Vec<i32> {
    (0..n)
        .map(|_| unsafe { R_unif_index(max) } as i32)
        .collect()
}

/// Test combining rng with worker thread strategy.
/// This verifies RNG works correctly when run on worker thread.
/// @rdname rpkg_rng
#[miniextendr(rng, worker)]
fn rng_worker_uniform(n: i32) -> Vec<f64> {
    (0..n).map(|_| unsafe { unif_rand() }).collect()
}

// =============================================================================
// Manual RNG management tests
// =============================================================================

/// Test RngGuard for manual RNG state management.
/// @rdname rpkg_rng
#[miniextendr]
fn rng_guard_test(n: i32) -> Vec<f64> {
    let _guard = RngGuard::new();
    (0..n).map(|_| unsafe { unif_rand() }).collect()
}

/// Test with_rng helper for scoped RNG access.
/// @rdname rpkg_rng
#[miniextendr]
fn rng_with_rng_test(n: i32) -> Vec<f64> {
    with_rng(|| (0..n).map(|_| unsafe { unif_rand() }).collect())
}

// =============================================================================
// Impl method tests
// =============================================================================

/// A struct to test RNG in impl methods.
#[derive(Clone)]
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

// =============================================================================
// Trait method tests (env class system)
// =============================================================================

/// Trait for RNG sampling.
pub trait Sampler {
    fn sample(&self, n: i32) -> Vec<f64>;
}

/// Implementation using env class system.
pub struct EnvSampler {
    multiplier: f64,
}

#[miniextendr(env)]
impl Sampler for EnvSampler {
    /// Sample n values, multiplied by the multiplier.
    #[miniextendr(rng)]
    fn sample(&self, n: i32) -> Vec<f64> {
        (0..n)
            .map(|_| unsafe { unif_rand() } * self.multiplier)
            .collect()
    }
}

#[miniextendr(env)]
impl EnvSampler {
    fn new(multiplier: f64) -> Self {
        Self { multiplier }
    }

    fn multiplier(&self) -> f64 {
        self.multiplier
    }
}

// =============================================================================
// Module registration
// =============================================================================

miniextendr_module! {
    mod rng_tests;

    // Standalone functions
    fn rng_uniform;
    fn rng_normal;
    fn rng_exponential;
    fn rng_int;
    fn rng_worker_uniform;

    // Manual RNG tests
    fn rng_guard_test;
    fn rng_with_rng_test;

    // Impl methods
    impl RngSampler;

    // Trait methods (env)
    impl Sampler for EnvSampler;
    impl EnvSampler;
}
