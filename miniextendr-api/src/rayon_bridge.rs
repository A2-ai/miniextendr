//! Rayon integration for parallel computation with R interop.
//!
//! This module provides seamless Rayon integration by leveraging miniextendr's
//! existing type system (`IntoR`, `RNativeType`, `TryCoerce`).
//!
//! # Design Philosophy
//!
//! **Rust computation: Parallel on Rayon threads (normal 2MB stacks)**
//! **R API calls: Serial on worker/main thread (before/after parallel work)**
//!
//! Uses existing infrastructure:
//! - `IntoR` trait for R conversion
//! - `RNativeType` trait for type → SEXPTYPE mapping
//! - `with_r_thread` for main thread dispatch
//!
//! # Quick Start
//!
//! ```ignore
//! use miniextendr_api::miniextendr;
//! use miniextendr_api::rayon_bridge::rayon::prelude::*;
//!
//! #[miniextendr]
//! fn parallel_sqrt(x: &[f64]) -> Vec<f64> {
//!     // Pure Rust parallel computation - no R calls inside!
//!     x.par_iter().map(|&v| v.sqrt()).collect()
//! }
//! ```
//!
//! # Architecture
//!
//! ```text
//! ┌───────────────────────────────────────────────────────────┐
//! │                    R Main Thread                          │
//! │  .Call("my_func") → miniextendr entry point               │
//! │  (also handles routed R API calls from other threads)     │
//! └─────────────────────────┬─────────────────────────────────┘
//!                           ↓
//! ┌─────────────────────────┴─────────────────────────────────┐
//! │               Worker Thread (run_on_worker)               │
//! │  1. Setup: with_r_vec() allocates R vectors               │
//! │  2. Parallel: spawn Rayon work                            │
//! │  3. Cleanup: convert results to R                         │
//! └─────────────────────────┬─────────────────────────────────┘
//!                           ↓
//! ┌───────────────────────────────────────────────────────────┐
//! │              Rayon Thread Pool (2MB stacks)               │
//! │   Thread 1    Thread 2    Thread 3    Thread N            │
//! │      ↓           ↓           ↓           ↓                │
//! │   Compute    Compute    Compute    Compute                │
//! │   (R API calls route to main thread - safe but slow)      │
//! └───────────────────────────────────────────────────────────┘
//! ```
//!
//! # Thread Context for `with_r_vec`
//!
//! [`with_r_vec`] can be called from **any thread**. It works by:
//!
//! 1. **Allocation on R thread**: Uses [`with_r_thread`][crate::worker::with_r_thread]
//!    to allocate and PROTECT the R vector on the R main thread.
//! 2. **Pointer acquisition**: The raw pointer is obtained while the object is protected.
//! 3. **Parallel fill**: The closure receives a `&mut [T]` slice that Rayon threads
//!    can safely write to. The PROTECTED vector cannot be collected by GC.
//! 4. **Cleanup**: UNPROTECT is called via a guard, and the SEXP is returned.
//!
//! **Safety**: The R vector is PROTECTED for the entire duration of parallel writes.
//! Even if R APIs are called from within the closure (which route to main thread),
//! GC cannot collect the protected vector, so the slice remains valid.
//!
//! **Efficiency tip**: For best performance, keep the closure body as pure Rust.
//! R API calls inside will work but serialize through the main thread.
//!
//! # Performance: Avoid R Calls in Hot Loops
//!
//! **Non-`_unchecked` FFI calls are safe from any thread** - they automatically
//! route to the R main thread via [`with_r_thread`][crate::worker::with_r_thread].
//! However, this routing has overhead, so calling R APIs inside tight parallel
//! loops defeats the purpose of parallelism.
//!
//! ## Inefficient Patterns (safe but slow)
//!
//! ```ignore
//! // SLOW: Each Rf_ScalarReal routes to main thread
//! data.par_iter().map(|x| {
//!     unsafe { ffi::Rf_ScalarReal(*x) }  // Works, but serializes on main thread
//! }).collect()
//!
//! // SLOW: into_sexp() routes to main thread per element
//! data.par_iter().map(|x| {
//!     x.into_sexp()  // Works, but no parallelism benefit
//! }).collect()
//! ```
//!
//! ## Efficient Patterns
//!
//! ```ignore
//! // FAST: Pure Rust parallel computation, single R conversion after
//! let results: Vec<f64> = data.par_iter().map(|x| x.sqrt()).collect();
//! results.into_sexp()  // One R call at the end
//!
//! // FAST: Pre-allocate R vector, parallel fill with pure Rust
//! with_r_vec(data.len(), |output: &mut [f64]| {
//!     output.par_iter_mut()
//!         .zip(data.par_iter())
//!         .for_each(|(out, x)| *out = x.sqrt());
//! })
//!
//! // FAST: Use reduce::* for parallel reductions
//! rayon_bridge::reduce::sum(&data)
//! ```
//!
//! ## Special Case: `_unchecked` FFI Functions
//!
//! The `*_unchecked` variants bypass thread routing and **must** be called
//! from the R main thread. Calling them from Rayon threads is undefined behavior.
//!
//! ```ignore
//! // UNSAFE: _unchecked variants don't route to main thread
//! data.par_iter().map(|x| {
//!     unsafe { ffi::Rf_ScalarReal_unchecked(*x) }  // UB! No routing
//! }).collect()
//! ```
//!
//! # RNG in Parallel Code
//!
//! `RRng` (with `rand` feature) routes each call to R's RNG on the main thread.
//! This is safe but serializes. For parallel RNG, use Rust's `thread_rng`:
//!
//! ```ignore
//! // SLOW: RRng serializes on main thread
//! data.par_iter().map(|x| {
//!     let mut rng = RRng::new();
//!     x + rng.uniform_f64()  // Each call routes to main thread
//! }).collect::<Vec<_>>()
//!
//! // FAST: thread_rng is thread-local, no routing (not R-reproducible)
//! use rand::Rng;
//! data.par_iter().map(|x| {
//!     let mut rng = rand::thread_rng();
//!     x + rng.random::<f64>()
//! }).collect::<Vec<_>>()
//! ```
//!
//! # Summary
//!
//! | Pattern | Safety | Performance |
//! |---------|--------|-------------|
//! | Pure Rust in `par_iter`, R at end | Safe | Fast |
//! | `with_r_vec` + parallel fill | Safe | Fast |
//! | R FFI (non-`_unchecked`) in `par_iter` | Safe | Slow (serialized) |
//! | R FFI `_unchecked` in `par_iter` | **UB** | N/A |

use crate::IntoR;
use crate::ffi::{RNativeType, SEXP};
use crate::worker::with_r_thread;

#[cfg(feature = "rayon")]
pub use rayon;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

// region: RVec - Parallel collection container

/// Container for parallel iterator results.
///
/// Implements `FromParallelIterator`, allowing:
/// ```ignore
/// let results: RVec<f64> = data.par_iter().map(f).collect();
/// let r_vec = results.into_sexp();  // Uses IntoR trait
/// ```
///
/// IntoR implementation is in `into_r.rs`.
#[cfg(feature = "rayon")]
#[derive(Debug, Clone)]
pub struct RVec<T> {
    data: Vec<T>,
}

#[cfg(feature = "rayon")]
impl<T> RVec<T> {
    /// Create from a Vec.
    #[inline]
    pub fn from_vec(data: Vec<T>) -> Self {
        data.into()
    }

    /// Get length.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get slice view.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Consume and get Vec.
    #[inline]
    pub fn into_inner(self) -> Vec<T> {
        self.into()
    }
}

#[cfg(feature = "rayon")]
impl<T> From<Vec<T>> for RVec<T> {
    fn from(data: Vec<T>) -> Self {
        Self { data }
    }
}

#[cfg(feature = "rayon")]
impl<T> From<RVec<T>> for Vec<T> {
    fn from(rvec: RVec<T>) -> Self {
        rvec.data
    }
}

#[cfg(feature = "rayon")]
impl<T: Send> FromParallelIterator<T> for RVec<T> {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>,
    {
        Self {
            data: par_iter.into_par_iter().collect(),
        }
    }
}

// endregion

// region: Zero-copy pre-allocation

/// Pre-allocate an R vector, fill in parallel, return the SEXP.
///
/// This is the most efficient pattern for parallel output - it writes directly
/// to R memory without intermediate copies.
///
/// The type `T` must implement [`RNativeType`], which maps Rust types to R vector types:
/// - `f64` → `REALSXP`
/// - `i32` → `INTSXP`
/// - `RLogical` → `LGLSXP`
/// - `u8` → `RAWSXP`
/// - `Rcomplex` → `CPLXSXP`
///
/// # Thread Safety
///
/// This function can be called from either the worker thread or directly from
/// the R main thread. It uses [`with_r_thread`][crate::worker::with_r_thread]
/// internally to ensure R allocation happens on the correct thread.
///
/// **Critical**: The closure `f` must contain only pure Rust code. Do not call
/// any R APIs inside the closure - this would violate R's single-threaded model.
///
/// # Example
///
/// ```ignore
/// // Type is inferred from the closure parameter
/// let r_vec = with_r_vec(1000, |output: &mut [f64]| {
///     output.par_iter_mut()
///         .enumerate()
///         .for_each(|(i, slot)| *slot = (i as f64).sqrt());
/// });
/// ```
///
/// # Protection
///
/// The vector is protected during the closure execution using `Rf_protect`.
/// After the function returns, the SEXP is unprotected and becomes the caller's
/// responsibility to protect (e.g., by returning it to R or protecting it).
#[cfg(feature = "rayon")]
pub fn with_r_vec<T, F>(len: usize, f: F) -> SEXP
where
    T: RNativeType + Send + Sync,
    F: FnOnce(&mut [T]),
{
    struct UnprotectGuard;

    impl Drop for UnprotectGuard {
        fn drop(&mut self) {
            with_r_thread(move || unsafe {
                crate::ffi::Rf_unprotect(1);
            });
        }
    }

    // Allocate and protect on the main/worker thread
    let sexp = with_r_thread(move || unsafe {
        let sexp = crate::ffi::Rf_allocVector(T::SEXP_TYPE, len as crate::ffi::R_xlen_t);
        crate::ffi::Rf_protect(sexp);
        sexp
    });
    let _guard = UnprotectGuard;

    // Get pointer and create slice (safe: vector is protected)
    // Note: dataptr_mut handles empty vectors by returning aligned dangling pointer
    let ptr = unsafe { T::dataptr_mut(sexp) };
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };

    // Run user's parallel work
    f(slice);

    sexp
}

// endregion

// region: Parallel reduction

/// Parallel reduction operations.
///
/// These functions perform parallel computation and convert to R scalars.
#[cfg(feature = "rayon")]
pub mod reduce {
    use super::*;

    /// Parallel sum → R scalar (f64).
    pub fn sum(slice: &[f64]) -> SEXP {
        let total: f64 = slice.par_iter().sum();
        with_r_thread(move || total.into_sexp())
    }

    /// Parallel sum → R scalar (i32).
    pub fn sum_int(slice: &[i32]) -> SEXP {
        let total: i32 = slice.par_iter().sum();
        with_r_thread(move || total.into_sexp())
    }

    /// Parallel minimum.
    pub fn min(slice: &[f64]) -> SEXP {
        let min_val = slice
            .par_iter()
            .copied()
            .reduce(|| f64::INFINITY, |a, b| a.min(b));
        with_r_thread(move || min_val.into_sexp())
    }

    /// Parallel maximum.
    pub fn max(slice: &[f64]) -> SEXP {
        let max_val = slice
            .par_iter()
            .copied()
            .reduce(|| f64::NEG_INFINITY, |a, b| a.max(b));
        with_r_thread(move || max_val.into_sexp())
    }

    /// Parallel mean.
    pub fn mean(slice: &[f64]) -> SEXP {
        if slice.is_empty() {
            return with_r_thread(|| f64::NAN.into_sexp());
        }

        let (sum, count) = slice
            .par_iter()
            .fold(|| (0.0_f64, 0_usize), |(s, c), &x| (s + x, c + 1))
            .reduce(|| (0.0, 0), |(s1, c1), (s2, c2)| (s1 + s2, c1 + c2));

        let mean_val = sum / count as f64;
        with_r_thread(move || mean_val.into_sexp())
    }
}

// endregion

// region: Performance utilities

#[cfg(feature = "rayon")]
pub mod perf {
    /// Get number of threads in Rayon pool.
    pub fn num_threads() -> usize {
        rayon::current_num_threads()
    }

    /// Check if in a Rayon thread.
    pub fn in_rayon_thread() -> bool {
        rayon::current_thread_index().is_some()
    }

    /// Get thread index.
    pub fn thread_index() -> Option<usize> {
        rayon::current_thread_index()
    }
}

// endregion

#[cfg(all(test, feature = "rayon"))]
mod tests {
    use super::*;

    #[test]
    fn test_rvec_creation() {
        let vec = RVec::from_vec(vec![1, 2, 3]);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_parallel_collect() {
        use rayon::prelude::*;

        let result: RVec<i32> = (0..100).into_par_iter().collect();
        assert_eq!(result.len(), 100);
        assert_eq!(result.as_slice()[0], 0);
        assert_eq!(result.as_slice()[99], 99);
    }

    #[test]
    fn test_parallel_map() {
        use rayon::prelude::*;

        let data = vec![1.0, 2.0, 3.0, 4.0];
        let doubled: RVec<f64> = data.par_iter().map(|&x| x * 2.0).collect();

        assert_eq!(doubled.as_slice(), &[2.0, 4.0, 6.0, 8.0]);
    }
}
