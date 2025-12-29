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

#[cfg(test)]
mod tests {
    use super::*;

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
}
