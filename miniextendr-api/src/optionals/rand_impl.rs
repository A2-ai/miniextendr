//! Integration with the `rand` crate for R's RNG.
//!
//! This module provides [`RRng`], a wrapper around R's random number generator
//! that implements the `rand` crate's [`RngCore`] trait. This allows using R's
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
//! ```ignore
//! use miniextendr_api::rand_impl::RRng;
//! use rand::Rng;
//!
//! #[miniextendr(rng)]
//! fn random_gaussian(n: i32) -> Vec<f64> {
//!     let mut rng = RRng::new();
//!     (0..n).map(|_| rng.random::<f64>()).collect()
//! }
//! ```
//!
//! # R's Native Distributions
//!
//! For maximum efficiency, this module also provides direct access to R's
//! native distribution functions via the [`RDistributions`] trait:
//!
//! ```ignore
//! use miniextendr_api::rand_impl::{RRng, RDistributions};
//!
//! #[miniextendr(rng)]
//! fn efficient_normal(n: i32) -> Vec<f64> {
//!     let mut rng = RRng::new();
//!     // Uses R's norm_rand() directly - no Box-Muller overhead
//!     (0..n).map(|_| rng.standard_normal()).collect()
//! }
//! ```
//!
//! These are faster than going through `rand`'s generic distribution machinery
//! because they use R's optimized C implementations directly.
//!
//! # Safety
//!
//! [`RRng`] requires that R's RNG state has been initialized via `GetRNGstate()`.
//! Use `#[miniextendr(rng)]` or [`RngGuard`][crate::RngGuard] to ensure this.
//!
//! # Performance Considerations
//!
//! **Thread-routing cost**: When `RRng` is used inside a `#[miniextendr]`-exported
//! function running on the worker thread, each RNG call is routed back to the
//! R main thread. This has implications:
//!
//! - **Small draws (tens to hundreds)**: Overhead is negligible
//! - **Large draws (thousands+)**: Cross-thread roundtrips can dominate runtime
//!
//! For high-volume random number generation, consider:
//!
//! 1. **Batch generation**: Generate all needed random numbers in one call, then
//!    process them on the worker thread
//! 2. **Use Rust RNG**: For non-reproducibility-critical work, use `rand::thread_rng()`
//!    which doesn't require R thread access
//!
//! ```ignore
//! #[miniextendr(rng)]
//! fn efficient_simulation(n: i32) -> f64 {
//!     let mut rng = RRng::new();
//!     // Generate all random numbers first
//!     let samples: Vec<f64> = (0..n).map(|_| rng.uniform_f64()).collect();
//!     // Then do expensive computation without RNG calls
//!     samples.iter().map(|x| expensive_function(*x)).sum()
//! }
//! ```

use rand::RngCore;

/// A wrapper around R's random number generator that implements [`RngCore`].
///
/// This allows using R's RNG with any `rand`-compatible code, ensuring
/// reproducibility when seeds are set via `set.seed()` in R.
///
/// # Requirements
///
/// R's RNG state must be initialized before using this type. Either:
/// - Use `#[miniextendr(rng)]` attribute on the function
/// - Create an [`RngGuard`][crate::RngGuard] before using `RRng`
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rand_impl::RRng;
/// use rand::Rng;
///
/// #[miniextendr(rng)]
/// fn random_sample(n: i32) -> Vec<f64> {
///     let mut rng = RRng::new();
///     // Generate n random f64 values in [0, 1)
///     (0..n).map(|_| rng.random()).collect()
/// }
/// ```
#[derive(Debug, Default)]
pub struct RRng {
    _private: (),
}

impl RRng {
    /// Create a new R RNG wrapper.
    ///
    /// # Safety Requirements
    ///
    /// R's RNG state must have been initialized via `GetRNGstate()` before
    /// calling any methods on this type. Use `#[miniextendr(rng)]` or
    /// [`RngGuard`][crate::RngGuard] to ensure this.
    #[inline]
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl RngCore for RRng {
    /// Generate a random u32 using R's RNG.
    ///
    /// Uses `unif_rand()` to generate a value in [0, 1) and scales to u32 range.
    #[inline]
    fn next_u32(&mut self) -> u32 {
        // R's unif_rand() returns a value in [0, 1)
        // Scale to full u32 range
        let u = unsafe { crate::ffi::unif_rand() };
        // u * 2^32, but we need to be careful with floating point
        // Using the standard conversion: floor(u * (MAX + 1))
        (u * (u32::MAX as f64 + 1.0)) as u32
    }

    /// Generate a random u64 using R's RNG.
    ///
    /// Combines two u32 values to create a full u64.
    #[inline]
    fn next_u64(&mut self) -> u64 {
        // Combine two u32 values
        let high = self.next_u32() as u64;
        let low = self.next_u32() as u64;
        (high << 32) | low
    }

    /// Fill a byte slice with random data from R's RNG.
    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        // Fill using u64 values for efficiency
        let mut chunks = dest.chunks_exact_mut(8);
        for chunk in chunks.by_ref() {
            let val = self.next_u64();
            chunk.copy_from_slice(&val.to_le_bytes());
        }
        // Handle remainder
        let remainder = chunks.into_remainder();
        if !remainder.is_empty() {
            let val = self.next_u64();
            let bytes = val.to_le_bytes();
            remainder.copy_from_slice(&bytes[..remainder.len()]);
        }
    }
}

// =============================================================================
// R's Native Distribution Functions
// =============================================================================

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
/// use miniextendr_api::rand_impl::{RRng, RDistributions};
///
/// #[miniextendr(rng)]
/// fn sample_distributions() -> Vec<f64> {
///     let mut rng = RRng::new();
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

    /// Generate a uniform random f64 in [0, 1).
    ///
    /// Uses R's `unif_rand()` directly with full f64 precision.
    fn uniform_f64(&mut self) -> f64;
}

impl RDistributions for RRng {
    #[inline]
    fn standard_normal(&mut self) -> f64 {
        unsafe { crate::ffi::norm_rand() }
    }

    #[inline]
    fn standard_exp(&mut self) -> f64 {
        unsafe { crate::ffi::exp_rand() }
    }

    #[inline]
    fn uniform_index(&mut self, n: usize) -> usize {
        unsafe { crate::ffi::R_unif_index(n as f64) as usize }
    }

    #[inline]
    fn uniform_f64(&mut self) -> f64 {
        unsafe { crate::ffi::unif_rand() }
    }
}

// =============================================================================
// Adapter Traits for Exposing RNGs to R
// =============================================================================

/// Adapter trait for exposing any [`rand::Rng`] to R.
///
/// This trait provides R-friendly methods for random number generation.
/// It has a blanket implementation for all types implementing `Rng`,
/// so any Rust RNG can be exposed to R with no additional code.
///
/// # Methods
///
/// - `r_random_f64()` - Random float in [0, 1)
/// - `r_random_i32()` - Random i32 (full range)
/// - `r_random_bool()` - Random boolean (50/50)
/// - `r_gen_range_f64(low, high)` - Random float in [low, high)
/// - `r_gen_range_i32(low, high)` - Random integer in [low, high)
/// - `r_gen_bool(p)` - Bernoulli trial with probability p
/// - `r_shuffle(items)` - Shuffle a vector in place
/// - `r_sample(items, n)` - Sample n items without replacement
///
/// # Example
///
/// ```ignore
/// use std::cell::RefCell;
/// use rand::rngs::StdRng;
/// use rand::SeedableRng;
///
/// #[derive(ExternalPtr)]
/// struct MyRng(RefCell<StdRng>);
///
/// impl MyRng {
///     fn new(seed: u64) -> Self {
///         Self(RefCell::new(StdRng::seed_from_u64(seed)))
///     }
/// }
///
/// impl RRngOps for MyRng {
///     fn random_f64(&self) -> f64 {
///         use rand::Rng;
///         self.0.borrow_mut().random()
///     }
///     // ... implement other methods using self.0.borrow_mut()
/// }
///
/// #[miniextendr]
/// impl RRngOps for MyRng {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RRngOps for MyRng;
/// }
/// ```
///
/// In R:
/// ```r
/// rng <- MyRng$new(42L)
/// rng$r_random_f64()           # Random float in [0, 1)
/// rng$r_gen_range_f64(0, 10)   # Random float in [0, 10)
/// rng$r_gen_bool(0.3)          # TRUE with 30% probability
/// ```
///
/// # Design Note
///
/// Like `RIterator`, this trait does NOT have a blanket impl because
/// `rand::Rng` methods require `&mut self`, but R's ExternalPtr pattern
/// provides `&self`. Users must implement manually using interior mutability.
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

// Note: No blanket impl because Rng methods require &mut self,
// but ExternalPtr methods receive &self. Users must use interior mutability.

/// Adapter trait for exposing probability distributions to R.
///
/// This trait provides methods for sampling from any probability distribution.
/// Implementations typically wrap both a distribution and an RNG together,
/// using interior mutability for the RNG state.
///
/// # Methods
///
/// - `r_sample()` - Draw a single sample from the distribution
/// - `r_sample_n(n)` - Draw n samples from the distribution
/// - `r_sample_vec(n)` - Alias for sample_n
///
/// # Example
///
/// ```ignore
/// use std::cell::RefCell;
/// use rand::rngs::StdRng;
/// use rand::SeedableRng;
/// use rand_distr::{Normal, Distribution};
///
/// #[derive(ExternalPtr)]
/// struct NormalDist {
///     dist: Normal<f64>,
///     rng: RefCell<StdRng>,
/// }
///
/// impl NormalDist {
///     fn new(mean: f64, std_dev: f64, seed: u64) -> Self {
///         Self {
///             dist: Normal::new(mean, std_dev).unwrap(),
///             rng: RefCell::new(StdRng::seed_from_u64(seed)),
///         }
///     }
/// }
///
/// impl RDistributionOps<f64> for NormalDist {
///     fn sample(&self) -> f64 {
///         self.dist.sample(&mut *self.rng.borrow_mut())
///     }
/// }
///
/// #[miniextendr]
/// impl RDistributionOps<f64> for NormalDist {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RDistributionOps<f64> for NormalDist;
/// }
/// ```
///
/// In R:
/// ```r
/// dist <- NormalDist$new(mean = 0, std_dev = 1, seed = 42L)
/// dist$r_sample()        # Single sample
/// dist$r_sample_n(100L)  # 100 samples
/// ```
///
/// # Design Note
///
/// Like `RIterator` and `RRngOps`, this trait does NOT have a blanket impl
/// because sampling requires mutable RNG state, but R's ExternalPtr pattern
/// provides `&self`. Users must use interior mutability (RefCell, Mutex, etc.).
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
mod tests {
    use super::*;
    use std::cell::RefCell;

    // Note: These tests require R to be initialized and RNG state loaded.
    // They're primarily for documentation/coverage purposes.

    #[test]
    fn rrng_can_be_created() {
        let _rng = RRng::new();
    }

    #[test]
    fn rrng_is_default() {
        let _rng: RRng = Default::default();
    }

    // Test RRngOps with a mock implementation
    struct MockRng(RefCell<u64>);

    impl MockRng {
        fn new(seed: u64) -> Self {
            Self(RefCell::new(seed))
        }

        // Simple LCG for testing
        fn next(&self) -> u64 {
            let mut state = self.0.borrow_mut();
            *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            *state
        }
    }

    impl RRngOps for MockRng {
        fn random_f64(&self) -> f64 {
            (self.next() as f64) / (u64::MAX as f64)
        }

        fn random_i32(&self) -> i32 {
            self.next() as i32
        }

        fn random_bool(&self) -> bool {
            self.next() % 2 == 0
        }

        fn gen_range_f64(&self, low: f64, high: f64) -> f64 {
            assert!(low < high, "low must be less than high");
            low + self.random_f64() * (high - low)
        }

        fn gen_range_i32(&self, low: i32, high: i32) -> i32 {
            assert!(low < high, "low must be less than high");
            let range = (high - low) as u64;
            low + (self.next() % range) as i32
        }

        fn gen_bool(&self, p: f64) -> bool {
            assert!((0.0..=1.0).contains(&p), "p must be in [0, 1]");
            self.random_f64() < p
        }
    }

    #[test]
    fn test_rrngops_random_f64() {
        let rng = MockRng::new(42);
        let val = rng.random_f64();
        assert!((0.0..1.0).contains(&val));
    }

    #[test]
    fn test_rrngops_random_i32() {
        let rng = MockRng::new(42);
        let _val = rng.random_i32(); // Just verify it doesn't panic
    }

    #[test]
    fn test_rrngops_random_bool() {
        let rng = MockRng::new(42);
        // Generate multiple to verify both outcomes are possible
        let bools: Vec<bool> = (0..100).map(|_| rng.random_bool()).collect();
        assert!(bools.iter().any(|&b| b));
        assert!(bools.iter().any(|&b| !b));
    }

    #[test]
    fn test_rrngops_gen_range_f64() {
        let rng = MockRng::new(42);
        for _ in 0..100 {
            let val = rng.gen_range_f64(10.0, 20.0);
            assert!((10.0..20.0).contains(&val));
        }
    }

    #[test]
    fn test_rrngops_gen_range_i32() {
        let rng = MockRng::new(42);
        for _ in 0..100 {
            let val = rng.gen_range_i32(5, 15);
            assert!((5..15).contains(&val));
        }
    }

    #[test]
    fn test_rrngops_gen_bool() {
        let rng = MockRng::new(42);
        // With p=0.5, should get roughly equal distribution
        let count_true = (0..1000).filter(|_| rng.gen_bool(0.5)).count();
        // Should be roughly 500 ± 100
        assert!(count_true > 350 && count_true < 650);
    }

    #[test]
    fn test_rrngops_vec_methods() {
        let rng = MockRng::new(42);

        let f64_vec = rng.random_f64_vec(10);
        assert_eq!(f64_vec.len(), 10);
        assert!(f64_vec.iter().all(|&v| (0.0..1.0).contains(&v)));

        let range_vec = rng.gen_range_f64_vec(10, -5.0, 5.0);
        assert_eq!(range_vec.len(), 10);
        assert!(range_vec.iter().all(|&v| (-5.0..5.0).contains(&v)));

        let int_vec = rng.gen_range_i32_vec(10, 0, 100);
        assert_eq!(int_vec.len(), 10);
        assert!(int_vec.iter().all(|&v| (0..100).contains(&v)));

        let bool_vec = rng.gen_bool_vec(10, 0.5);
        assert_eq!(bool_vec.len(), 10);
    }

    #[test]
    #[should_panic(expected = "low must be less than high")]
    fn test_rrngops_gen_range_f64_invalid() {
        let rng = MockRng::new(42);
        rng.gen_range_f64(10.0, 5.0); // low > high
    }

    #[test]
    #[should_panic(expected = "p must be in [0, 1]")]
    fn test_rrngops_gen_bool_invalid() {
        let rng = MockRng::new(42);
        rng.gen_bool(1.5); // p > 1
    }

    // Test RDistributionOps with a mock uniform distribution
    struct MockUniformDist {
        low: f64,
        high: f64,
        rng: RefCell<u64>,
    }

    impl MockUniformDist {
        fn new(low: f64, high: f64, seed: u64) -> Self {
            Self {
                low,
                high,
                rng: RefCell::new(seed),
            }
        }

        fn next_f64(&self) -> f64 {
            let mut state = self.rng.borrow_mut();
            *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            (*state as f64) / (u64::MAX as f64)
        }
    }

    impl RDistributionOps<f64> for MockUniformDist {
        fn sample(&self) -> f64 {
            self.low + self.next_f64() * (self.high - self.low)
        }

        fn mean(&self) -> Option<f64> {
            Some((self.low + self.high) / 2.0)
        }

        fn variance(&self) -> Option<f64> {
            let range = self.high - self.low;
            Some(range * range / 12.0)
        }
    }

    #[test]
    fn test_rdistributionops_sample() {
        let dist = MockUniformDist::new(0.0, 10.0, 42);
        let sample = dist.sample();
        assert!((0.0..10.0).contains(&sample));
    }

    #[test]
    fn test_rdistributionops_sample_n() {
        let dist = MockUniformDist::new(5.0, 15.0, 42);
        let samples = dist.sample_n(100);
        assert_eq!(samples.len(), 100);
        assert!(samples.iter().all(|&s| (5.0..15.0).contains(&s)));
    }

    #[test]
    fn test_rdistributionops_sample_vec() {
        let dist = MockUniformDist::new(0.0, 1.0, 42);
        let samples = dist.sample_vec(50);
        assert_eq!(samples.len(), 50);
    }

    #[test]
    fn test_rdistributionops_mean() {
        let dist = MockUniformDist::new(0.0, 10.0, 42);
        assert_eq!(dist.mean(), Some(5.0));
    }

    #[test]
    fn test_rdistributionops_variance() {
        let dist = MockUniformDist::new(0.0, 12.0, 42);
        // Variance of uniform(0, 12) = (12-0)^2 / 12 = 144/12 = 12
        assert_eq!(dist.variance(), Some(12.0));
    }

    #[test]
    fn test_rdistributionops_std_dev() {
        let dist = MockUniformDist::new(0.0, 12.0, 42);
        let std_dev = dist.std_dev().unwrap();
        // std_dev = sqrt(12) ≈ 3.464
        assert!((std_dev - 12.0_f64.sqrt()).abs() < 1e-10);
    }

    // Test distribution with no known statistics
    struct MockUnknownDist(RefCell<u64>);

    impl RDistributionOps<i32> for MockUnknownDist {
        fn sample(&self) -> i32 {
            let mut state = self.0.borrow_mut();
            *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            (*state % 100) as i32
        }
    }

    #[test]
    fn test_rdistributionops_unknown_stats() {
        let dist = MockUnknownDist(RefCell::new(42));
        assert_eq!(dist.mean(), None);
        assert_eq!(dist.variance(), None);
        assert_eq!(dist.std_dev(), None);
        // But sampling still works
        let _sample = dist.sample();
    }
}
