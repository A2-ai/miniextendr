//! Rayon integration for parallel computation with R interop.
//!
//! This module provides seamless Rayon integration by leveraging miniextendr's
//! existing type system (`IntoR`, `RNativeType`, `TryCoerce`).
//!
//! # Design Philosophy
//!
//! **🚀 Rust computation: Parallel on Rayon threads (normal 2MB stacks)**
//! **🔒 R API calls: Serial on main thread (via `run_r`)**
//!
//! Uses existing infrastructure:
//! - `IntoR` trait for R conversion
//! - `RNativeType` trait for type → SEXPTYPE mapping
//! - `with_r_thread` for main thread dispatch
//!
//! # Quick Start
//!
//! ```ignore
//! use miniextendr_api::prelude::*;
//! use rayon::prelude::*;
//!
//! #[miniextendr]
//! fn parallel_sqrt(x: &[f64]) -> SEXP {
//!     // Leverage existing IntoR trait!
//!     x.par_iter()
//!         .map(|&v| v.sqrt())
//!         .collect::<Vec<f64>>()
//!         .into_sexp()  // Uses existing IntoR
//! }
//! ```
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────┐
//! │   Rayon Thread Pool (2MB stacks) │
//! │   Thread 1  Thread 2  Thread 3   │
//! │      ↓         ↓         ↓       │ Parallel Rust
//! │   run_r()  run_r()  run_r()     │ Need R? → Main thread
//! └──────┬─────────┬─────────┬───────┘
//!        │         │         │
//!        └─────────┴─────────┘
//!                  ↓
//!        ┌────────────────┐
//!        │ Main R Thread  │
//!        │ Rf_allocVector │ Sequential R ops
//!        │ IntoR traits   │
//!        └────────┬───────┘
//!                 ↓ Results
//!        Back to Rayon threads
//! ```

use crate::IntoR;
use crate::externalptr::SendableSexp;
use crate::ffi::SEXP;
use crate::worker::with_r_thread;

#[cfg(feature = "rayon")]
pub use rayon;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

// region: Core R execution guard

/// Execute R code on the main thread from a Rayon thread.
///
/// Routes R API calls through the existing `with_r_thread` infrastructure.
///
/// # Example
///
/// ```ignore
/// let sexp = run_r(|| unsafe { ffi::Rf_ScalarInteger(42) });
/// ```
///
/// # Panics
///
/// Panics if called outside a `run_on_worker` context.
#[cfg(feature = "rayon")]
#[inline]
pub fn run_r<F>(f: F) -> SEXP
where
    F: FnOnce() -> SEXP + Send + 'static,
{
    let sendable = with_r_thread(move || SendableSexp::new(f()));
    sendable.into_inner()
}

// endregion

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
    pub fn from_vec(data: Vec<T>) -> Self {
        Self { data }
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
    pub fn into_inner(self) -> Vec<T> {
        self.data
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

/// Pre-allocate R real vector, fill in parallel.
///
/// Most efficient pattern - writes directly to R memory.
///
/// # Example
///
/// ```ignore
/// let r_vec = with_r_real_vec(1000, |output| {
///     output.par_iter_mut()
///         .enumerate()
///         .for_each(|(i, slot)| *slot = (i as f64).sqrt());
/// });
/// ```
#[cfg(feature = "rayon")]
pub fn with_r_real_vec<F>(len: usize, f: F) -> SEXP
where
    F: FnOnce(&mut [f64]),
{
    let sexp = run_r(move || unsafe {
        crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::REALSXP, len as crate::ffi::R_xlen_t)
    });

    let ptr = unsafe { crate::ffi::REAL(sexp) };
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
    f(slice);
    sexp
}

/// Pre-allocate R integer vector.
#[cfg(feature = "rayon")]
pub fn with_r_int_vec<F>(len: usize, f: F) -> SEXP
where
    F: FnOnce(&mut [i32]),
{
    let sexp = run_r(move || unsafe {
        crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::INTSXP, len as crate::ffi::R_xlen_t)
    });

    let ptr = unsafe { crate::ffi::INTEGER(sexp) };
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
    f(slice);
    sexp
}

/// Pre-allocate R logical vector.
#[cfg(feature = "rayon")]
pub fn with_r_logical_vec<F>(len: usize, f: F) -> SEXP
where
    F: FnOnce(&mut [i32]),
{
    let sexp = run_r(move || unsafe {
        crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::LGLSXP, len as crate::ffi::R_xlen_t)
    });

    let ptr = unsafe { crate::ffi::LOGICAL(sexp) };
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
    f(slice);
    sexp
}

// endregion

// region: Parallel reduction

/// Parallel reduction operations using existing IntoR.
#[cfg(feature = "rayon")]
pub mod reduce {
    use super::*;

    /// Parallel sum → R scalar (uses IntoR for f64).
    pub fn sum(slice: &[f64]) -> SEXP {
        let total: f64 = slice.par_iter().sum();
        run_r(move || total.into_sexp())
    }

    /// Parallel sum for integers (uses IntoR for i32).
    pub fn sum_int(slice: &[i32]) -> SEXP {
        let total: i32 = slice.par_iter().sum();
        run_r(move || total.into_sexp())
    }

    /// Parallel minimum.
    pub fn min(slice: &[f64]) -> SEXP {
        let min_val = slice
            .par_iter()
            .copied()
            .reduce(|| f64::INFINITY, |a, b| a.min(b));
        run_r(move || min_val.into_sexp())
    }

    /// Parallel maximum.
    pub fn max(slice: &[f64]) -> SEXP {
        let max_val = slice
            .par_iter()
            .copied()
            .reduce(|| f64::NEG_INFINITY, |a, b| a.max(b));
        run_r(move || max_val.into_sexp())
    }

    /// Parallel mean.
    pub fn mean(slice: &[f64]) -> SEXP {
        if slice.is_empty() {
            return run_r(|| unsafe { crate::ffi::R_NaString });
        }

        let (sum, count) = slice
            .par_iter()
            .fold(|| (0.0_f64, 0_usize), |(s, c), &x| (s + x, c + 1))
            .reduce(|| (0.0, 0), |(s1, c1), (s2, c2)| (s1 + s2, c1 + c2));

        let mean_val = sum / count as f64;
        run_r(move || mean_val.into_sexp())
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
