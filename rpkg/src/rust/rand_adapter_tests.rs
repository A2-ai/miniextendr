//! Fixtures for the rand / rand_distr adapters.
//!
//! Covers the four surfaces of `miniextendr_api::rand_impl`:
//! - `RRng` driven through the `rand::RngExt` trait (R's RNG behind rand's API)
//! - `RDistributions` (R's native distribution functions as a trait on `RRng`)
//! - `RRngOps` (exposing a pure-Rust RNG to R via a trait impl)
//! - `RDistributionOps` (exposing a rand_distr distribution to R)
//!
//! `RRng` does NOT manage R's RNG state itself — every fixture touching it
//! carries `#[miniextendr(rng)]` so GetRNGstate/PutRNGstate bracket the call.
//! `SeededRng` / `SeededNormal` wrap a pure-Rust `StdRng` and need no
//! bracketing (and are deliberately independent of `set.seed()`).

use std::cell::RefCell;

use miniextendr_api::rand::rngs::StdRng;
use miniextendr_api::rand::{RngExt, SeedableRng};
use miniextendr_api::rand_impl::{RDistributions, RRng};
use miniextendr_api::{ExternalPtr, miniextendr};

// region: RRng through the rand trait surface

/// Sample n uniform values in [0, 1) from R's RNG via `RngExt::random`.
/// @param n Integer number of values to generate.
#[miniextendr(rng)]
pub fn rand_rrng_uniform(n: i32) -> Vec<f64> {
    let mut rng = RRng::new();
    (0..n).map(|_| rng.random::<f64>()).collect()
}

/// Sample n values in [low, high) from R's RNG via `RngExt::random_range`.
/// @param n Integer number of values to generate.
/// @param low Lower bound (inclusive).
/// @param high Upper bound (exclusive).
#[miniextendr(rng)]
pub fn rand_rrng_range(n: i32, low: f64, high: f64) -> Vec<f64> {
    let mut rng = RRng::new();
    (0..n).map(|_| rng.random_range(low..high)).collect()
}
// endregion

// region: RDistributions (R's native distribution functions through the trait)

/// Draw one value from each `RDistributions` method, in order:
/// c(standard_normal, standard_exp, uniform_index(100), uniform_f64).
#[miniextendr(rng)]
pub fn rand_rdistributions_sample() -> Vec<f64> {
    let mut rng = RRng::new();
    vec![
        rng.standard_normal(),
        rng.standard_exp(),
        rng.uniform_index(100) as f64,
        rng.uniform_f64(),
    ]
}
// endregion

// region: RRngOps (exposing a pure-Rust RNG to R)

/// A seeded, R-independent RNG (`rand::rngs::StdRng`) exposed through `RRngOps`.
/// @rdname rpkg_rand_adapter
/// @export
#[derive(ExternalPtr)]
pub struct SeededRng(RefCell<StdRng>);

/// SeededRng inherent methods: constructor.
#[miniextendr]
impl SeededRng {
    /// @param seed Integer seed (absolute value is used).
    fn new(seed: i32) -> Self {
        SeededRng(RefCell::new(StdRng::seed_from_u64(u64::from(
            seed.unsigned_abs(),
        ))))
    }
}

/// RRngOps trait implementation for SeededRng (interior-mutable StdRng).
#[miniextendr]
impl miniextendr_api::rand_impl::RRngOps for SeededRng {
    fn random_f64(&self) -> f64 {
        self.0.borrow_mut().random()
    }

    fn random_i32(&self) -> i32 {
        self.0.borrow_mut().random()
    }

    fn random_bool(&self) -> bool {
        self.0.borrow_mut().random()
    }

    /// @param low Lower bound (inclusive).
    /// @param high Upper bound (exclusive).
    fn gen_range_f64(&self, low: f64, high: f64) -> f64 {
        self.0.borrow_mut().random_range(low..high)
    }

    /// @param low Lower bound (inclusive).
    /// @param high Upper bound (exclusive).
    fn gen_range_i32(&self, low: i32, high: i32) -> i32 {
        self.0.borrow_mut().random_range(low..high)
    }

    /// @param p Probability of TRUE, in [0, 1].
    fn gen_bool(&self, p: f64) -> bool {
        self.0.borrow_mut().random_bool(p)
    }

    // Default methods — the trait vtable requires every method implemented.
    /// @param n Integer number of values to generate.
    fn random_f64_vec(&self, n: i32) -> Vec<f64> {
        (0..n).map(|_| self.random_f64()).collect()
    }

    /// @param n Integer number of values to generate.
    /// @param low Lower bound (inclusive).
    /// @param high Upper bound (exclusive).
    fn gen_range_f64_vec(&self, n: i32, low: f64, high: f64) -> Vec<f64> {
        (0..n).map(|_| self.gen_range_f64(low, high)).collect()
    }

    /// @param n Integer number of values to generate.
    /// @param low Lower bound (inclusive).
    /// @param high Upper bound (exclusive).
    fn gen_range_i32_vec(&self, n: i32, low: i32, high: i32) -> Vec<i32> {
        (0..n).map(|_| self.gen_range_i32(low, high)).collect()
    }

    /// @param n Integer number of values to generate.
    /// @param p Probability of TRUE, in [0, 1].
    fn gen_bool_vec(&self, n: i32, p: f64) -> Vec<bool> {
        (0..n).map(|_| self.gen_bool(p)).collect()
    }
}
// endregion

// region: rand_distr through RRng

/// Sample n values from Normal(mean, sd) via `rand_distr` driven by R's RNG.
/// @param n Integer number of values to generate.
/// @param mean Mean of the normal distribution.
/// @param sd Standard deviation (must be finite; a negative value mirrors the
///   distribution).
#[cfg(feature = "rand_distr")]
#[miniextendr(rng)]
pub fn rand_distr_normal(n: i32, mean: f64, sd: f64) -> Result<Vec<f64>, String> {
    use miniextendr_api::rand_distr::{Distribution, Normal};
    let normal = Normal::new(mean, sd).map_err(|e| format!("invalid normal parameters: {e}"))?;
    let mut rng = RRng::new();
    Ok((0..n).map(|_| normal.sample(&mut rng)).collect())
}
// endregion

// region: RDistributionOps (exposing a rand_distr distribution to R)

/// A Normal distribution paired with a seeded `StdRng`, exposed through
/// `RDistributionOps`.
/// @rdname rpkg_rand_adapter
/// @export
#[cfg(feature = "rand_distr")]
#[derive(ExternalPtr)]
pub struct SeededNormal {
    dist: miniextendr_api::rand_distr::Normal<f64>,
    rng: RefCell<StdRng>,
}

/// SeededNormal inherent methods: constructor.
#[cfg(feature = "rand_distr")]
#[miniextendr]
impl SeededNormal {
    /// @param mean Mean of the normal distribution.
    /// @param sd Standard deviation (must be finite).
    /// @param seed Integer seed (absolute value is used).
    fn new(mean: f64, sd: f64, seed: i32) -> Self {
        let dist = miniextendr_api::rand_distr::Normal::new(mean, sd).expect("sd must be finite");
        SeededNormal {
            dist,
            rng: RefCell::new(StdRng::seed_from_u64(u64::from(seed.unsigned_abs()))),
        }
    }
}

/// RDistributionOps trait implementation for SeededNormal.
#[cfg(feature = "rand_distr")]
#[miniextendr]
impl miniextendr_api::rand_impl::RDistributionOps<f64> for SeededNormal {
    fn sample(&self) -> f64 {
        use miniextendr_api::rand_distr::Distribution;
        self.dist.sample(&mut *self.rng.borrow_mut())
    }

    // Default methods — the trait vtable requires every method implemented.
    /// @param n Integer number of samples to draw.
    fn sample_n(&self, n: i32) -> Vec<f64> {
        (0..n).map(|_| self.sample()).collect()
    }

    /// @param n Integer number of samples to draw.
    fn sample_vec(&self, n: i32) -> Vec<f64> {
        self.sample_n(n)
    }

    fn mean(&self) -> Option<f64> {
        Some(self.dist.mean())
    }

    fn variance(&self) -> Option<f64> {
        let sd = self.dist.std_dev();
        Some(sd * sd)
    }

    fn std_dev(&self) -> Option<f64> {
        Some(self.dist.std_dev())
    }
}
// endregion
