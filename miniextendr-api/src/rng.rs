//! RNG (Random Number Generation) utilities for R interop.
//!
//! This module provides safe access to R's random number generators.
//! R's RNG must have its state loaded before generating random numbers, and
//! the state must be saved back afterwards (even on error).
//!
//! # Background
//!
//! R's random number generators maintain internal state that must be synchronized
//! with R's `.Random.seed` variable. Before drawing, `GetRNGstate()` loads the
//! state; after drawing, `PutRNGstate()` saves it back, even if an error occurs.
//!
//! An [`RngGuard`] is the proof that the state is loaded on R's main thread, so
//! drawing is a safe method on it, named after R's C function:
//!
//! - [`RngGuard::unif_rand`] - Uniform random on `(0, 1)`
//! - [`RngGuard::norm_rand`] - Standard normal random
//! - [`RngGuard::exp_rand`] - Standard exponential random
//! - [`RngGuard::unif_index`] - Uniform integer on `[0, n)` (`R_unif_index`)
//!
//! With the `rand` feature, `guard.rng()` gives an `RRng` that drives `rand`'s
//! traits from R's generator for as long as it borrows the guard.
//!
//! # Usage: The `#[miniextendr(rng)]` Attribute (Recommended)
//!
//! The `rng` option loads and saves the state around the call. A parameter of
//! type `&RngGuard` receives the guard the wrapper holds; it is no R argument
//! (not in the R function's formals, usage line or `@param` list):
//!
//! ```ignore
//! use miniextendr_api::RngGuard;
//!
//! #[miniextendr(rng)]
//! fn random_sample(n: i32, rng: &RngGuard) -> Vec<f64> {
//!     (0..n).map(|_| rng.unif_rand()).collect()
//! }
//! ```
//!
//! From R the function is `random_sample(n)`. A `&RngGuard` parameter on a
//! function without `rng` is a compile error: nothing has loaded the state.
//!
//! This also works on impl methods, in every class system:
//!
//! ```ignore
//! #[miniextendr]
//! impl MyStruct {
//!     #[miniextendr(rng)]
//!     fn sample(&self, n: i32, rng: &RngGuard) -> Vec<f64> {
//!         (0..n).map(|_| rng.norm_rand()).collect()
//!     }
//! }
//! ```
//!
//! A function or method that takes the guard runs on R's main thread, as every
//! `#[miniextendr]` function does: the guard cannot leave it. Trait methods
//! take no `&RngGuard` parameter; their arguments cross the trait ABI as R
//! values.
//!
//! ## Generated Code Pattern
//!
//! The `#[miniextendr(rng)]` attribute generates code that:
//!
//! 1. Enters an outer `with_r_unwind_protect` guard and calls `GetRNGstate()`
//! 2. Runs the body under an inner guard, returning tagged conditions on Rust panics
//! 3. Protects that result while `PutRNGstate()` saves the state
//! 4. Returns the value or tagged condition for the R wrapper to handle
//!
//! Both RNG calls can raise R errors, so both run inside the outer guard.
//! Saving runs on successful returns and caught Rust panics. R-origin nonlocal
//! exits propagate through the guards; they do not run the remaining body or
//! save step. The guard handed to a `&RngGuard` parameter does not save the
//! state when it drops: step 3 does.
//!
//! Inside an `rng` function, draw from the `&RngGuard` parameter rather than
//! calling [`with_rng`]. A nested `GetRNGstate()` reloads `.Random.seed`, which
//! still holds the state from before the call, so the draws would repeat.
//!
//! # Usage: Manual Control with [`with_rng`] and [`RngGuard`]
//!
//! For code that isn't directly exposed to R, or when you need finer control,
//! use [`with_rng`]. It loads the state when it is called, so a `set.seed()`
//! that ran in R before the call decides the draws:
//!
//! ```ignore
//! use miniextendr_api::rng::with_rng;
//!
//! let value = with_rng(|rng| rng.unif_rand());
//! ```
//!
//! Or create an [`RngGuard`] yourself:
//!
//! ```ignore
//! use miniextendr_api::rng::RngGuard;
//!
//! fn generate_random() -> f64 {
//!     // SAFETY: called on R's main thread.
//!     let rng = unsafe { RngGuard::new() };
//!     rng.unif_rand()
//!     // PutRNGstate() called automatically when `rng` drops
//! }
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
//! Use [`with_rng`] / [`RngGuard`] for:
//! - Internal helper functions not directly exposed to R
//! - Code already wrapped in `with_r_unwind_protect`
//! - Scoped RNG access within a larger function

use crate::sys::{GetRNGstate, PutRNGstate};
use std::marker::PhantomData;
use std::rc::Rc;

/// Enforces `!Send + !Sync` (R's RNG state is only valid on the R main thread).
type NoSendSync = PhantomData<Rc<()>>;

/// Proof that R's RNG state is loaded on R's main thread, and the way to draw
/// from it.
///
/// A guard from [`RngGuard::new`] or [`with_rng`] calls `GetRNGstate()` when it
/// is created and `PutRNGstate()` when it drops, so the state is saved even if
/// the function panics or returns early. The draws ([`unif_rand`](Self::unif_rand),
/// [`norm_rand`](Self::norm_rand), [`exp_rand`](Self::exp_rand),
/// [`unif_index`](Self::unif_index)) borrow the guard, so none can run outside
/// its scope. The guard is `!Send` and `!Sync`, so a draw can't run on another
/// thread either.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rng::RngGuard;
///
/// fn generate_uniform() -> f64 {
///     // SAFETY: called on R's main thread.
///     let rng = unsafe { RngGuard::new() };
///     rng.unif_rand()
/// }
/// ```
///
/// # Warning: R Longjumps
///
/// This guard relies on Rust's drop semantics. If R triggers a longjmp
/// (via `Rf_error` etc.), the destructor will NOT run unless the code
/// is wrapped in `with_r_unwind_protect`. For functions exposed to R,
/// prefer using `#[miniextendr(rng)]` which handles this correctly.
#[derive(Debug)]
pub struct RngGuard {
    /// Whether dropping the guard saves the state (`PutRNGstate()`). False
    /// only for the guard a `#[miniextendr(rng)]` wrapper hands the body,
    /// whose state the wrapper saves itself.
    saves: bool,
    // `!Send + !Sync` marker that also prevents construction outside this module.
    _nosend: NoSendSync,
}

impl RngGuard {
    /// Create a new RNG guard, loading the current RNG state.
    ///
    /// Calls `GetRNGstate()` to load R's `.Random.seed` into the RNG.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread — R's RNG state and the underlying
    /// `GetRNGstate()` / `PutRNGstate()` calls are only valid there. Calling
    /// this off the main thread is undefined behaviour. There is deliberately
    /// no `Default` impl: constructing a guard is an unsafe, thread-scoped act,
    /// not a safe default (#1096).
    #[inline]
    pub unsafe fn new() -> Self {
        unsafe { GetRNGstate() };
        Self {
            saves: true,
            _nosend: PhantomData,
        }
    }

    /// The guard a `#[miniextendr(rng)]` wrapper passes to a `&RngGuard`
    /// parameter. Generated code only.
    ///
    /// # Safety
    ///
    /// Must be called on R's main thread while the caller holds the state
    /// loaded (`GetRNGstate()` has run) and saves it with `PutRNGstate()`
    /// after the guard drops: dropping this guard saves nothing.
    #[doc(hidden)]
    #[inline]
    pub unsafe fn __mx_loaded_by_wrapper() -> Self {
        debug_assert!(
            crate::is_r_main_thread(),
            "an `rng` wrapper's guard must be made on R's main thread"
        );
        Self {
            saves: false,
            _nosend: PhantomData,
        }
    }

    /// A uniform draw on `(0, 1)`: R's `unif_rand()`, what `runif(1)` draws.
    #[inline]
    pub fn unif_rand(&self) -> f64 {
        // SAFETY: a guard exists only on R's main thread with the state loaded.
        unsafe { crate::sys::unif_rand_unchecked() }
    }

    /// A standard normal draw: R's `norm_rand()`, what `rnorm(1)` draws.
    #[inline]
    pub fn norm_rand(&self) -> f64 {
        // SAFETY: a guard exists only on R's main thread with the state loaded.
        unsafe { crate::sys::norm_rand_unchecked() }
    }

    /// A standard exponential draw (rate 1): R's `exp_rand()`, what `rexp(1)`
    /// draws.
    #[inline]
    pub fn exp_rand(&self) -> f64 {
        // SAFETY: a guard exists only on R's main thread with the state loaded.
        unsafe { crate::sys::exp_rand_unchecked() }
    }

    /// A uniform integer on `[0, n)`, as a double: R's `R_unif_index(n)`, the
    /// draw behind `sample()`. It uses the sample kind `RNGkind()` selects, so
    /// `unif_index(n) + 1` draws what `sample.int(n, 1)` draws. Take `n > 0`.
    #[inline]
    pub fn unif_index(&self, n: f64) -> f64 {
        // SAFETY: a guard exists only on R's main thread with the state loaded.
        unsafe { crate::sys::R_unif_index_unchecked(n) }
    }
}

impl Drop for RngGuard {
    #[inline]
    fn drop(&mut self) {
        // Always save RNG state, even on panic, unless a wrapper saves it.
        if self.saves {
            unsafe { PutRNGstate() };
        }
    }
}

/// Run `f` with R's RNG state loaded, handing it the guard to draw from.
///
/// Loads the state (`GetRNGstate()`) when it is called, not before, so a
/// `set.seed()` that ran in R before this call decides the draws. The state is
/// saved (`PutRNGstate()`) when `f` returns or panics. This is a convenience
/// wrapper around [`RngGuard`].
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rng::with_rng;
///
/// let values = with_rng(|rng| (0..10).map(|_| rng.unif_rand()).collect::<Vec<_>>());
/// ```
///
/// # Panics
///
/// Panics when called off R's main thread.
///
/// # Warning
///
/// Like [`RngGuard`], this relies on Rust drop semantics and won't
/// properly clean up if R longjumps. For R-exposed functions, use
/// `#[miniextendr(rng)]` instead, and draw from its `&RngGuard` parameter
/// rather than calling `with_rng` inside it (see the module docs).
#[inline]
pub fn with_rng<F, R>(f: F) -> R
where
    F: FnOnce(&RngGuard) -> R,
{
    // SAFETY: `RngGuard::new()` requires the R main thread. Keep this check in
    // release builds: otherwise this safe wrapper could invoke R's RNG API from
    // a background thread and trigger undefined behaviour (#1096).
    assert!(
        crate::is_r_main_thread(),
        "with_rng must run on R's main thread"
    );
    let guard = unsafe { RngGuard::new() };
    f(&guard)
}

#[cfg(test)]
mod tests;
