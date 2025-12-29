//! RNG (Random Number Generation) utilities for R interop.
//!
//! This module provides safe wrappers around R's RNG state management functions.
//! R's RNG must have its state loaded before generating random numbers, and
//! the state must be saved back afterwards (even on error).
//!
//! # Background
//!
//! R's random number generators maintain internal state that must be synchronized
//! with R's `.Random.seed` variable. Before calling any RNG function like
//! [`unif_rand()`][crate::ffi::unif_rand], you must call `GetRNGstate()` to load
//! the state. After generating random numbers, you must call `PutRNGstate()` to
//! save it back—even if an error occurs.
//!
//! # Available RNG Functions
//!
//! After initializing RNG state, you can use these functions from [`crate::ffi`]:
//!
//! - [`unif_rand()`][crate::ffi::unif_rand] - Uniform random on `[0, 1)`
//! - [`norm_rand()`][crate::ffi::norm_rand] - Standard normal random
//! - [`exp_rand()`][crate::ffi::exp_rand] - Standard exponential random
//! - [`R_unif_index(n)`][crate::ffi::R_unif_index] - Uniform integer on `[0, n)`
//!
//! # Usage: The `#[miniextendr(rng)]` Attribute (Recommended)
//!
//! The simplest and safest way is to use the `#[miniextendr(rng)]` attribute on
//! functions that need to generate random numbers:
//!
//! ```ignore
//! use miniextendr_api::ffi::unif_rand;
//!
//! #[miniextendr(rng)]
//! fn random_sample(n: i32) -> Vec<f64> {
//!     (0..n).map(|_| unsafe { unif_rand() }).collect()
//! }
//! ```
//!
//! This also works on impl methods and trait methods:
//!
//! ```ignore
//! #[miniextendr]
//! impl MyStruct {
//!     #[miniextendr(rng)]
//!     fn sample(&self, n: i32) -> Vec<f64> {
//!         (0..n).map(|_| unsafe { unif_rand() }).collect()
//!     }
//! }
//!
//! #[miniextendr(env)]
//! impl MyTrait for MyStruct {
//!     #[miniextendr(rng)]
//!     fn random_value(&self) -> f64 {
//!         unsafe { unif_rand() }
//!     }
//! }
//! ```
//!
//! ## Generated Code Pattern
//!
//! The `#[miniextendr(rng)]` attribute generates code that:
//!
//! 1. Calls `GetRNGstate()` at the start
//! 2. Wraps the function body in `catch_unwind`
//! 3. Calls `PutRNGstate()` after `catch_unwind` (runs on both success AND panic)
//! 4. Then handles the result (returns value or re-panics)
//!
//! This explicit placement ensures `PutRNGstate()` is called before any error
//! handling, which is robust in the presence of R longjumps when combined with
//! `with_r_unwind_protect`.
//!
//! # Usage: Manual Control with [`RngGuard`]
//!
//! For code that isn't directly exposed to R, or when you need finer control,
//! use [`RngGuard`]:
//!
//! ```ignore
//! use miniextendr_api::rng::RngGuard;
//! use miniextendr_api::ffi::unif_rand;
//!
//! fn generate_random() -> f64 {
//!     let _guard = RngGuard::new();
//!     unsafe { unif_rand() }
//!     // PutRNGstate() called automatically when _guard drops
//! }
//! ```
//!
//! Or use the [`with_rng`] convenience function:
//!
//! ```ignore
//! use miniextendr_api::rng::with_rng;
//! use miniextendr_api::ffi::unif_rand;
//!
//! let value = with_rng(|| unsafe { unif_rand() });
//! ```
//!
//! # Important: R Longjumps
//!
//! [`RngGuard`] and [`with_rng`] rely on Rust's drop semantics. If R triggers a
//! longjmp (via `Rf_error` etc.), the guard's destructor will NOT run unless
//! the code is wrapped in `with_r_unwind_protect`.
//!
//! **For functions exposed to R, always prefer `#[miniextendr(rng)]`** which
//! handles this correctly by using explicit placement of `PutRNGstate()`.
//!
//! Use [`RngGuard`] for:
//! - Internal helper functions not directly exposed to R
//! - Code already wrapped in `with_r_unwind_protect`
//! - Scoped RNG access within a larger function

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
