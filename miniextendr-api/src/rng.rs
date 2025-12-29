//! RNG (Random Number Generation) utilities for R interop.
//!
//! This module provides safe wrappers around R's RNG state management functions.
//! R's RNG must have its state loaded before generating random numbers, and
//! the state must be saved back afterwards (even on error).
//!
//! # Usage
//!
//! The simplest way is to use the `#[miniextendr(rng)]` attribute on functions
//! that need to generate random numbers:
//!
//! ```ignore
//! #[miniextendr(rng)]
//! fn random_sample(n: i32) -> Vec<f64> {
//!     (0..n).map(|_| unsafe { unif_rand() }).collect()
//! }
//! ```
//!
//! For manual control, use [`RngGuard`]:
//!
//! ```ignore
//! use miniextendr_api::rng::RngGuard;
//!
//! fn generate_random() -> f64 {
//!     let _guard = RngGuard::new();
//!     unsafe { unif_rand() }
//!     // PutRNGstate() called automatically when _guard drops
//! }
//! ```
//!
//! # Important: R Longjumps
//!
//! Note that [`RngGuard`] relies on Rust's drop semantics. If R triggers a
//! longjmp (via `Rf_error` etc.), the guard's destructor will NOT run unless
//! the code is wrapped in `with_r_unwind_protect`. The `#[miniextendr(rng)]`
//! attribute handles this correctly by using explicit placement.

use crate::ffi::{GetRNGstate, PutRNGstate};

/// RAII guard for R's RNG state.
///
/// Calls `GetRNGstate()` on creation and `PutRNGstate()` on drop.
/// This ensures RNG state is properly saved even if the function panics
/// or returns early.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rng::RngGuard;
/// use miniextendr_api::ffi::unif_rand;
///
/// fn generate_uniform() -> f64 {
///     let _guard = RngGuard::new();
///     unsafe { unif_rand() }
/// }
/// ```
///
/// # Warning: R Longjumps
///
/// This guard relies on Rust's drop semantics. If R triggers a longjmp
/// (via `Rf_error` etc.), the destructor will NOT run unless the code
/// is wrapped in `with_r_unwind_protect`. For functions exposed to R,
/// prefer using `#[miniextendr(rng)]` which handles this correctly.
///
/// # Safety
///
/// Must be used on R's main thread. The guard assumes it has exclusive
/// access to R's RNG state while alive.
pub struct RngGuard {
    _private: (), // Prevent construction outside this module
}

impl RngGuard {
    /// Create a new RNG guard, loading the current RNG state.
    ///
    /// Calls `GetRNGstate()` to load R's `.Random.seed` into the RNG.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread.
    #[inline]
    pub fn new() -> Self {
        unsafe { GetRNGstate() };
        Self { _private: () }
    }
}

impl Default for RngGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for RngGuard {
    #[inline]
    fn drop(&mut self) {
        // Always save RNG state, even on panic
        unsafe { PutRNGstate() };
    }
}

/// Scope guard for RNG operations.
///
/// Executes a closure with RNG state properly managed.
/// This is a convenience wrapper around [`RngGuard`].
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rng::with_rng;
/// use miniextendr_api::ffi::unif_rand;
///
/// let values = with_rng(|| {
///     (0..10).map(|_| unsafe { unif_rand() }).collect::<Vec<_>>()
/// });
/// ```
///
/// # Warning
///
/// Like [`RngGuard`], this relies on Rust drop semantics and won't
/// properly clean up if R longjumps. For R-exposed functions, use
/// `#[miniextendr(rng)]` instead.
#[inline]
pub fn with_rng<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = RngGuard::new();
    f()
}
