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
//! # Safety
//!
//! [`RRng`] requires that R's RNG state has been initialized via `GetRNGstate()`.
//! Use `#[miniextendr(rng)]` or [`RngGuard`][crate::RngGuard] to ensure this.

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
