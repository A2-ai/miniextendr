//! RNG (Random Number Generation) tests.
//!
//! Tests for:
//! - `#[miniextendr(rng)]` on standalone functions, drawing from the
//!   `&RngGuard` parameter the wrapper fills (no R argument, #2002)
//! - the same on impl methods, one class per class system
//! - Manual RNG management with `RngGuard` and `with_rng`

use miniextendr_api::miniextendr;
use miniextendr_api::rng::{RngGuard, with_rng};

// region: Standalone function tests

/// Test generating n uniform random numbers using R's RNG.
/// @param n Integer number of values to generate.
#[miniextendr(rng)]
pub fn rng_uniform(n: i32, rng: &RngGuard) -> Vec<f64> {
    (0..n).map(|_| rng.unif_rand()).collect()
}

/// Test generating n standard normal random numbers using R's RNG.
/// @param n Integer number of values to generate.
#[miniextendr(rng)]
pub fn rng_normal(rng: &RngGuard, n: i32) -> Vec<f64> {
    (0..n).map(|_| rng.norm_rand()).collect()
}

/// Test generating n exponential random numbers using R's RNG.
/// @param n Integer number of values to generate.
#[miniextendr(rng)]
pub fn rng_exponential(n: i32, rng: &RngGuard) -> Vec<f64> {
    (0..n).map(|_| rng.exp_rand()).collect()
}

/// Test generating n random integers in [0, max) using R's RNG.
/// @param n Integer number of values to generate.
/// @param max Numeric upper bound (exclusive) for the random integers.
#[miniextendr(rng)]
pub fn rng_int(n: i32, rng: &RngGuard, max: f64) -> Vec<i32> {
    (0..n).map(|_| rng.unif_index(max) as i32).collect()
}

/// Test RNG with interrupt checking enabled.
/// @param n Integer number of uniform values to generate.
#[miniextendr(rng, check_interrupt)]
pub fn rng_with_interrupt(n: i32, rng: &RngGuard) -> Vec<f64> {
    (0..n).map(|_| rng.unif_rand()).collect()
}

// endregion

// region: Manual RNG management tests

/// Test manual RNG management using RngGuard RAII pattern.
/// @param n Integer number of uniform values to generate.
#[miniextendr]
pub fn rng_guard_test(n: i32) -> Vec<f64> {
    // SAFETY: the wrapper runs this body on R's main thread.
    let rng = unsafe { RngGuard::new() };
    (0..n).map(|_| rng.unif_rand()).collect()
}

/// Test manual RNG management using the with_rng closure API.
/// @param n Integer number of uniform values to generate.
#[miniextendr]
pub fn rng_with_rng_test(n: i32) -> Vec<f64> {
    with_rng(|rng| (0..n).map(|_| rng.unif_rand()).collect())
}

/// One draw of each kind through `with_rng`, in order:
/// c(unif_rand, norm_rand, exp_rand, unif_index(10)).
#[miniextendr]
pub fn rng_with_rng_each() -> Vec<f64> {
    with_rng(|rng| {
        vec![
            rng.unif_rand(),
            rng.norm_rand(),
            rng.exp_rand(),
            rng.unif_index(10.0),
        ]
    })
}
// endregion

// region: Impl method tests, one class per class system

/// A struct to test RNG in impl methods.
/// @rdname rpkg_rng
/// @export
#[derive(Clone, miniextendr_api::ExternalPtr)]
pub struct RngSampler {
    seed_hint: i32,
}

#[miniextendr(env)]
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
    fn sample_uniform(&self, n: i32, rng: &RngGuard) -> Vec<f64> {
        (0..n).map(|_| rng.unif_rand()).collect()
    }

    /// Sample n normal values.
    #[miniextendr(rng)]
    fn sample_normal(&self, rng: &RngGuard, n: i32) -> Vec<f64> {
        (0..n).map(|_| rng.norm_rand()).collect()
    }

    /// Static method with rng.
    #[miniextendr(rng)]
    fn static_sample(n: i32, rng: &RngGuard) -> Vec<f64> {
        (0..n).map(|_| rng.unif_rand()).collect()
    }
}

/// R6 class drawing from R's RNG.
#[derive(miniextendr_api::ExternalPtr)]
pub struct RngSamplerR6 {
    scale: f64,
}

#[miniextendr(r6, noexport)]
impl RngSamplerR6 {
    /// Create the fixture.
    /// @param scale Multiplies every draw.
    pub fn new(scale: f64) -> Self {
        Self { scale }
    }

    /// `n` scaled uniform draws.
    /// @param n Number of draws.
    #[miniextendr(rng)]
    pub fn unif(&self, n: i32, rng: &RngGuard) -> Vec<f64> {
        (0..n).map(|_| self.scale * rng.unif_rand()).collect()
    }
}

/// S3 class drawing from R's RNG.
#[derive(miniextendr_api::ExternalPtr)]
pub struct RngSamplerS3 {
    scale: f64,
}

/// S3 class drawing from R's RNG. Exported: roxygen2 wants every S3 method
/// exported or registered.
#[miniextendr(s3)]
impl RngSamplerS3 {
    /// Create the fixture.
    /// @param scale Multiplies every draw.
    pub fn new(scale: f64) -> Self {
        Self { scale }
    }

    /// `n` scaled uniform draws.
    /// @param n Number of draws.
    #[miniextendr(rng)]
    pub fn rng_s3_unif(&self, rng: &RngGuard, n: i32) -> Vec<f64> {
        (0..n).map(|_| self.scale * rng.unif_rand()).collect()
    }
}

/// S4 class drawing from R's RNG.
#[derive(miniextendr_api::ExternalPtr)]
pub struct RngSamplerS4 {
    scale: f64,
}

#[miniextendr(s4, noexport)]
impl RngSamplerS4 {
    /// Create the fixture.
    /// @param scale Multiplies every draw.
    pub fn new(scale: f64) -> Self {
        Self { scale }
    }

    /// `n` scaled uniform draws.
    /// @param n Number of draws.
    #[miniextendr(rng)]
    pub fn rng_unif(&self, n: i32, rng: &RngGuard) -> Vec<f64> {
        (0..n).map(|_| self.scale * rng.unif_rand()).collect()
    }
}

/// S7 class drawing from R's RNG.
#[derive(miniextendr_api::ExternalPtr)]
pub struct RngSamplerS7 {
    scale: f64,
}

#[miniextendr(s7, noexport)]
impl RngSamplerS7 {
    /// Create the fixture.
    /// @param scale Multiplies every draw.
    pub fn new(scale: f64) -> Self {
        Self { scale }
    }

    /// `n` scaled uniform draws.
    /// @param n Number of draws.
    #[miniextendr(rng)]
    pub fn rng_s7_unif(&self, n: i32, rng: &RngGuard) -> Vec<f64> {
        (0..n).map(|_| self.scale * rng.unif_rand()).collect()
    }
}

/// vctrs class: vctrs impls take no instance methods (MXL120), so the
/// constructor draws.
pub struct RngNoise;

#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "rngn"), noexport)]
impl RngNoise {
    /// `n` standard normal draws.
    /// @param n Number of draws.
    #[allow(clippy::new_ret_no_self)]
    #[miniextendr(rng)]
    pub fn new(n: i32, rng: &RngGuard) -> Vec<f64> {
        (0..n).map(|_| rng.norm_rand()).collect()
    }
}
// endregion

// region: Upstream example-derived fixtures

/// Generate a random boolean using R's RNG.
#[miniextendr(rng)]
pub fn rng_bool(rng: &RngGuard) -> bool {
    rng.unif_rand() < 0.5
}

/// Generate a random double in [min, max) using R's RNG.
/// @param min Lower bound.
/// @param max Upper bound.
#[miniextendr(rng)]
pub fn rng_range(min: f64, max: f64, rng: &RngGuard) -> f64 {
    min + rng.unif_rand() * (max - min)
}

/// Generate n chi-squared-like values using the sum-of-normals method.
/// @param n Number of values.
/// @param df Degrees of freedom (integer, used as count of squared normals).
#[miniextendr(rng)]
pub fn rng_chi_sq_approx(n: i32, df: i32, rng: &RngGuard) -> Vec<f64> {
    (0..n)
        .map(|_| {
            (0..df)
                .map(|_| {
                    let z = rng.norm_rand();
                    z * z
                })
                .sum()
        })
        .collect()
}

// endregion
