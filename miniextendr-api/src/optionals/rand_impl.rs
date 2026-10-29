//! Integration with the `rand` crate for R's RNG.
//!
//! This module provides [`RRng`], a wrapper around R's random number generator
//! that implements the `rand` crate's `Rng` trait. This allows using R's
//! RNG with any `rand`-compatible code.
//!
//! # Features
//!
//! Enable this module with the `rand` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["rand"] }
//! ```
//!
//! # Usage
//!
//! An [`RRng`] borrows an [`RngGuard`][crate::RngGuard], the proof that R's
//! RNG state is loaded on R's main thread:
//! [`RngGuard::rng`][crate::RngGuard::rng] makes one. Under
//! `#[miniextendr(rng)]`, take the guard as a `&RngGuard` parameter (it is no R
//! argument):
//!
//! ```ignore
//! use miniextendr_api::RngGuard;
//! use rand::RngExt;
//!
//! #[miniextendr(rng)]
//! fn random_uniform(n: i32, guard: &RngGuard) -> Vec<f64> {
//!     let mut rng = guard.rng();
//!     (0..n).map(|_| rng.random::<f64>()).collect()
//! }
//! ```
//!
//! Elsewhere on R's main thread, [`with_rng`][crate::with_rng] hands a closure
//! the guard: `with_rng(|guard| guard.rng().random::<f64>())`.
//!
//! # R's Native Distributions
//!
//! For maximum efficiency, this module also provides direct access to R's
//! native distribution functions via the [`RDistributions`] trait:
//!
//! ```ignore
//! use miniextendr_api::RngGuard;
//! use miniextendr_api::rand_impl::RDistributions;
//!
//! #[miniextendr(rng)]
//! fn efficient_normal(n: i32, guard: &RngGuard) -> Vec<f64> {
//!     let mut rng = guard.rng();
//!     // Uses R's norm_rand() directly - no Box-Muller overhead
//!     (0..n).map(|_| rng.standard_normal()).collect()
//! }
//! ```
//!
//! These are faster than going through `rand`'s generic distribution machinery
//! because they use R's optimized C implementations directly, and they draw
//! what R's own `rnorm()` / `rexp()` / `sample()` draw for the same seed.
//!
//! # Threads
//!
//! [`RRng`] is `!Send` and `!Sync`, like the guard it borrows: R's RNG state
//! belongs to R's main thread. It can't move into a worker closure or a rayon
//! task. For parallel random numbers, seed a Rust RNG per chunk instead (the
//! `rayon` feature's docs show how), or draw what you need from R first and
//! hand the values over.

use std::convert::Infallible;

use crate::miniextendr;
use crate::rng::RngGuard;
use rand::TryRng;

/// A wrapper around R's random number generator that implements `rand::Rng`.
///
/// This allows using R's RNG with any `rand`-compatible code, ensuring
/// reproducibility when seeds are set via `set.seed()` in R.
///
/// An `RRng` borrows the [`RngGuard`] it was made from ([`RngGuard::rng`]),
/// so it draws only while R's RNG state is loaded, and only on R's main thread
/// (`RRng` is `!Send` and `!Sync`).
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::RngGuard;
/// use rand::RngExt;
///
/// #[miniextendr(rng)]
/// fn random_sample(n: i32, guard: &RngGuard) -> Vec<f64> {
///     let mut rng = guard.rng();
///     // Generate n random f64 values in [0, 1)
///     (0..n).map(|_| rng.random()).collect()
/// }
/// ```
#[derive(Debug)]
pub struct RRng<'g> {
    guard: &'g RngGuard,
}

impl RngGuard {
    /// An [`RRng`] drawing from R's RNG through this guard, for `rand`'s
    /// traits and [`RDistributions`].
    #[inline]
    pub fn rng(&self) -> RRng<'_> {
        RRng { guard: self }
    }
}

/// Implement `TryRng` with `Error = Infallible` so the blanket impl provides `Rng`.
impl TryRng for RRng<'_> {
    type Error = Infallible;

    /// Generate a random u32 using R's RNG.
    ///
    /// Uses `unif_rand()` to generate a value in (0, 1) and scales to u32 range.
    #[inline]
    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        // R's unif_rand() returns a value in (0, 1)
        // Scale to full u32 range
        let u = self.guard.unif_rand();
        // u * 2^32, but we need to be careful with floating point
        // Using the standard conversion: floor(u * (MAX + 1))
        // SAFETY (lint): `u ∈ (0, 1)` so the product is in `(0, 2^32)`; the cast
        // is the intended floor-to-u32 of a non-negative, in-range value.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let scaled = (u * (f64::from(u32::MAX) + 1.0)) as u32;
        Ok(scaled)
    }

    /// Generate a random u64 using R's RNG.
    ///
    /// Combines two u32 values to create a full u64.
    #[inline]
    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        // Combine two u32 values — call try_next_u32 (infallible) directly
        let high = u64::from(self.try_next_u32()?);
        let low = u64::from(self.try_next_u32()?);
        Ok((high << 32) | low)
    }

    /// Fill a byte slice with random data from R's RNG.
    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Infallible> {
        // Fill using u64 values for efficiency
        let (chunks, remainder) = dest.as_chunks_mut::<8>();
        for chunk in chunks {
            *chunk = self.try_next_u64()?.to_le_bytes();
        }
        // Handle remainder
        if !remainder.is_empty() {
            let val = self.try_next_u64()?;
            let bytes = val.to_le_bytes();
            remainder.copy_from_slice(&bytes[..remainder.len()]);
        }
        Ok(())
    }
}

// region: R's Native Distribution Functions

/// Direct access to R's native distribution functions.
///
/// These methods bypass `rand`'s generic distribution machinery and call
/// R's optimized C implementations directly. Use these when you need:
///
/// - **Standard normal**: `standard_normal()` uses `norm_rand()`
/// - **Exponential(1)**: `standard_exp()` uses `exp_rand()`
/// - **Uniform integer**: `uniform_index(n)` uses `R_unif_index()`
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::RngGuard;
/// use miniextendr_api::rand_impl::RDistributions;
///
/// #[miniextendr(rng)]
/// fn sample_distributions(guard: &RngGuard) -> Vec<f64> {
///     let mut rng = guard.rng();
///     vec![
///         rng.standard_normal(),  // N(0, 1)
///         rng.standard_exp(),     // Exp(1)
///         rng.uniform_index(100) as f64,  // Uniform integer in [0, 100)
///     ]
/// }
/// ```
pub trait RDistributions {
    /// Generate a standard normal random value (mean 0, sd 1).
    ///
    /// Uses R's `norm_rand()` directly, which is typically faster than
    /// `rand`'s Box-Muller or Ziggurat implementations because R's RNG
    /// is already optimized for this.
    fn standard_normal(&mut self) -> f64;

    /// Generate a standard exponential random value (rate 1).
    ///
    /// Uses R's `exp_rand()` directly, which is typically faster than
    /// using the inverse transform method through `rand`.
    fn standard_exp(&mut self) -> f64;

    /// Generate a uniform random integer in [0, n).
    ///
    /// Uses R's `R_unif_index()` directly, which handles edge cases
    /// and provides good uniformity without rejection sampling overhead.
    fn uniform_index(&mut self, n: usize) -> usize;

    /// Generate a uniform random f64 in (0, 1).
    ///
    /// Uses R's `unif_rand()` directly: the value `runif(1)` draws.
    fn uniform_f64(&mut self) -> f64;
}

impl RDistributions for RRng<'_> {
    #[inline]
    fn standard_normal(&mut self) -> f64 {
        self.guard.norm_rand()
    }

    #[inline]
    fn standard_exp(&mut self) -> f64 {
        self.guard.exp_rand()
    }

    #[inline]
    fn uniform_index(&mut self, n: usize) -> usize {
        // SAFETY (lint): `R_unif_index` returns a non-negative f64 in `[0, n)`;
        // the `as usize` cast floors it back into range. `n as f64` may lose
        // precision only for n > 2^53, which exceeds any realistic R length.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let idx = self.guard.unif_index(n as f64) as usize;
        idx
    }

    #[inline]
    fn uniform_f64(&mut self) -> f64 {
        self.guard.unif_rand()
    }
}
// endregion

// region: Adapter Traits for Exposing RNGs to R

/// Adapter trait for exposing any [`rand::RngExt`] to R.
///
/// This trait provides R-friendly methods for random number generation.
/// Implement it on a concrete class that owns mutable RNG state.
///
/// # Methods
///
/// - `random_f64()` - Random float in [0, 1)
/// - `random_i32()` - Random i32 (full range)
/// - `random_bool()` - Random boolean (50/50)
/// - `gen_range_f64(low, high)` - Random float in [low, high)
/// - `gen_range_i32(low, high)` - Random integer in [low, high)
/// - `gen_bool(p)` - Bernoulli trial with probability p
/// - `random_f64_vec(n)` / `gen_range_f64_vec(...)` / `gen_range_i32_vec(...)`
///   / `gen_bool_vec(...)` - Vectorized variants (default methods)
///
/// # Example
///
/// ```ignore
/// use std::cell::RefCell;
/// use miniextendr_api::rand::rngs::StdRng;
/// use miniextendr_api::rand::{RngExt, SeedableRng};
///
/// #[derive(ExternalPtr)]
/// struct MyRng(RefCell<StdRng>);
///
/// #[miniextendr]
/// impl MyRng {
///     fn new(seed: i32) -> Self {
///         Self(RefCell::new(StdRng::seed_from_u64(u64::from(seed.unsigned_abs()))))
///     }
/// }
///
/// /// RRngOps trait implementation for MyRng.
/// #[miniextendr]
/// impl miniextendr_api::rand_impl::RRngOps for MyRng {
///     fn random_f64(&self) -> f64 {
///         self.0.borrow_mut().random()
///     }
///     // ... implement ALL trait methods (including defaults) — the trait
///     // vtable requires every method. Use #[miniextendr(skip)] on a method
///     // to keep it out of R while still filling the vtable slot.
/// }
/// ```
///
/// In R (trait methods live under the trait's namespace):
/// ```r
/// rng <- MyRng$new(42L)
/// rng$RRngOps$random_f64()           # Random float in [0, 1)
/// rng$RRngOps$gen_range_f64(0, 10)   # Random float in [0, 10)
/// rng$RRngOps$gen_bool(0.3)          # TRUE with 30% probability
/// ```
///
/// # Design Note
///
/// This trait does not have a blanket impl because
/// `rand::RngExt` methods require `&mut self`, but R's ExternalPtr pattern
/// provides `&self`. Users must implement manually using interior mutability.
#[miniextendr]
pub trait RRngOps {
    /// Generate a random f64 in [0, 1).
    fn random_f64(&self) -> f64;

    /// Generate a random i32 covering the full i32 range.
    fn random_i32(&self) -> i32;

    /// Generate a random boolean (50% chance each).
    fn random_bool(&self) -> bool;

    /// Generate a random f64 in [low, high).
    ///
    /// # Panics
    ///
    /// Panics if `low >= high`.
    fn gen_range_f64(&self, low: f64, high: f64) -> f64;

    /// Generate a random i32 in [low, high).
    ///
    /// # Panics
    ///
    /// Panics if `low >= high`.
    fn gen_range_i32(&self, low: i32, high: i32) -> i32;

    /// Generate a boolean with probability `p` of being true.
    ///
    /// # Arguments
    ///
    /// * `p` - Probability of returning true, in [0, 1]
    ///
    /// # Panics
    ///
    /// Panics if `p < 0` or `p > 1`.
    fn gen_bool(&self, p: f64) -> bool;

    /// Fill a vector with random f64 values in [0, 1).
    ///
    /// Returns a new vector of the given length.
    fn random_f64_vec(&self, n: i32) -> Vec<f64> {
        (0..n).map(|_| self.random_f64()).collect()
    }

    /// Fill a vector with random f64 values in [low, high).
    fn gen_range_f64_vec(&self, n: i32, low: f64, high: f64) -> Vec<f64> {
        (0..n).map(|_| self.gen_range_f64(low, high)).collect()
    }

    /// Fill a vector with random i32 values in [low, high).
    fn gen_range_i32_vec(&self, n: i32, low: i32, high: i32) -> Vec<i32> {
        (0..n).map(|_| self.gen_range_i32(low, high)).collect()
    }

    /// Fill a vector with random booleans with probability `p` of true.
    fn gen_bool_vec(&self, n: i32, p: f64) -> Vec<bool> {
        (0..n).map(|_| self.gen_bool(p)).collect()
    }
}

// Note: No blanket impl because RngExt methods require &mut self,
// but ExternalPtr methods receive &self. Users must use interior mutability.

/// Adapter trait for exposing probability distributions to R.
///
/// This trait provides methods for sampling from any probability distribution.
/// Implementations typically wrap both a distribution and an RNG together,
/// using interior mutability for the RNG state.
///
/// # Methods
///
/// - `sample()` - Draw a single sample from the distribution
/// - `sample_n(n)` - Draw n samples from the distribution
/// - `sample_vec(n)` - Alias for sample_n
/// - `mean()` / `variance()` / `std_dev()` - Known statistics (default `None`)
///
/// # Example
///
/// ```ignore
/// use std::cell::RefCell;
/// use miniextendr_api::rand::rngs::StdRng;
/// use miniextendr_api::rand::SeedableRng;
/// use miniextendr_api::rand_distr::{Distribution, Normal};
///
/// #[derive(ExternalPtr)]
/// struct NormalDist {
///     dist: Normal<f64>,
///     rng: RefCell<StdRng>,
/// }
///
/// #[miniextendr]
/// impl NormalDist {
///     fn new(mean: f64, std_dev: f64, seed: i32) -> Self {
///         Self {
///             dist: Normal::new(mean, std_dev).unwrap(),
///             rng: RefCell::new(StdRng::seed_from_u64(u64::from(seed.unsigned_abs()))),
///         }
///     }
/// }
///
/// /// RDistributionOps trait implementation for NormalDist.
/// #[miniextendr]
/// impl miniextendr_api::rand_impl::RDistributionOps<f64> for NormalDist {
///     fn sample(&self) -> f64 {
///         self.dist.sample(&mut *self.rng.borrow_mut())
///     }
///     // ... implement ALL trait methods (including defaults) — the trait
///     // vtable requires every method. Use #[miniextendr(skip)] on a method
///     // to keep it out of R while still filling the vtable slot.
/// }
/// ```
///
/// In R (trait methods live under the trait's namespace):
/// ```r
/// dist <- NormalDist$new(mean = 0, std_dev = 1, seed = 42L)
/// dist$RDistributionOps$sample()        # Single sample
/// dist$RDistributionOps$sample_n(100L)  # 100 samples
/// ```
///
/// # Design Note
///
/// Like `RRngOps`, this trait does not have a blanket impl
/// because sampling requires mutable RNG state, but R's ExternalPtr pattern
/// provides `&self`. Users must use interior mutability (RefCell, Mutex, etc.).
#[miniextendr]
pub trait RDistributionOps<T> {
    /// Draw a single sample from the distribution.
    fn sample(&self) -> T;

    /// Draw n samples from the distribution.
    ///
    /// Default implementation calls `r_sample()` n times.
    fn sample_n(&self, n: i32) -> Vec<T> {
        (0..n).map(|_| self.sample()).collect()
    }

    /// Draw n samples from the distribution (alias for sample_n).
    fn sample_vec(&self, n: i32) -> Vec<T> {
        self.sample_n(n)
    }

    /// Get the mean/expected value of the distribution, if known.
    ///
    /// Returns None by default. Override for distributions with known mean.
    fn mean(&self) -> Option<f64> {
        None
    }

    /// Get the variance of the distribution, if known.
    ///
    /// Returns None by default. Override for distributions with known variance.
    fn variance(&self) -> Option<f64> {
        None
    }

    /// Get the standard deviation of the distribution, if known.
    ///
    /// Default implementation returns sqrt(variance) if variance is known.
    fn std_dev(&self) -> Option<f64> {
        self.variance().map(|v| v.sqrt())
    }
}

// Note: No blanket impl because Distribution::sample() requires &mut Rng,
// but ExternalPtr methods receive &self. Users must use interior mutability.

#[cfg(test)]
mod tests;
// endregion
