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
//! │  (handles routed R API calls from worker thread)          │
//! └─────────────────────────┬─────────────────────────────────┘
//!                           ↓
//! ┌─────────────────────────┴─────────────────────────────────┐
//! │               Worker Thread (run_on_worker)               │
//! │  1. Setup: with_r_vec() allocates R vectors               │
//! │  2. Parallel: spawn Rayon work                            │
//! │  3. Cleanup: convert results to R                         │
//! │  (can route R API calls to main thread via with_r_thread) │
//! └─────────────────────────┬─────────────────────────────────┘
//!                           ↓
//! ┌───────────────────────────────────────────────────────────┐
//! │              Rayon Thread Pool (2MB stacks)               │
//! │   Thread 1    Thread 2    Thread 3    Thread N            │
//! │      ↓           ↓           ↓           ↓                │
//! │   Compute    Compute    Compute    Compute                │
//! │   ⚠️  PURE RUST ONLY - NO R API CALLS ⚠️                  │
//! └───────────────────────────────────────────────────────────┘
//! ```
//!
//! # Thread Context for `with_r_vec`
//!
//! [`with_r_vec`] can be called from the **worker thread or main thread**. It works by:
//!
//! 1. **Allocation on R thread**: Uses [`with_r_thread`][crate::worker::with_r_thread]
//!    to allocate and PROTECT the R vector on the R main thread.
//! 2. **Pointer acquisition**: The raw pointer is obtained on the main thread (inside
//!    the `with_r_thread` call) while the object is protected.
//! 3. **Parallel fill**: The closure receives a `&mut [T]` slice that Rayon threads
//!    can safely write to. The PROTECTED vector cannot be collected by GC.
//! 4. **Cleanup**: UNPROTECT is called via a guard, and the SEXP is returned.
//!
//! **Safety**: The R vector is PROTECTED for the entire duration of parallel writes.
//! GC cannot collect the protected vector, so the slice remains valid.
//!
//! **Critical**: The closure **must contain only pure Rust code**. Do not call any
//! R APIs from Rayon threads - they will panic (no routing available).
//!
//! # CRITICAL: No R API Calls from Rayon Threads
//!
//! **R API calls from Rayon threads will panic.** The `with_r_thread` routing
//! mechanism only works from the worker thread (inside `run_on_worker`), not from
//! Rayon pool threads. Rayon threads do not have the thread-local channels needed
//! for routing.
//!
//! ## Broken Patterns (will panic)
//!
//! ```ignore
//! // PANIC: Rayon threads cannot call R APIs
//! data.par_iter().map(|x| {
//!     unsafe { ffi::Rf_ScalarReal(*x) }  // PANICS! with_r_thread not available
//! }).collect()
//!
//! // PANIC: into_sexp() tries to route but fails on Rayon threads
//! data.par_iter().map(|x| {
//!     x.into_sexp()  // PANICS! with_r_thread not available
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
//! `RRng` (with `rand` feature) calls R's RNG APIs, which cannot be called from
//! Rayon threads. For parallel RNG, use Rust's `thread_rng`:
//!
//! ```ignore
//! // PANIC: RRng cannot be used from Rayon threads
//! data.par_iter().map(|x| {
//!     let mut rng = RRng::new();
//!     x + rng.uniform_f64()  // PANICS! R API not available
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
//! | R FFI (any) in `par_iter` | **Panic** | N/A |
//! | R FFI `_unchecked` in `par_iter` | **UB** | N/A |
//!
//! # APIs That CANNOT Be Called from Rayon Threads
//!
//! The following will **panic** or cause **undefined behavior** if called from
//! inside `par_iter`, `par_chunks`, or any Rayon parallel context:
//!
//! - **Any `ffi::*` function** (both checked and unchecked variants)
//! - **`with_r_thread()`** - routing only works from the worker thread
//! - **`with_r_vec()`**, **`with_r_matrix()`**, etc. - these use `with_r_thread` internally
//! - **`.into_sexp()`** on any type - routes to main thread, fails on Rayon threads
//! - **`RRng`** and R's RNG functions - call R APIs internally
//! - **`TryFromSexp::try_from_sexp()`** - may call R for error messages
//!
//! **The golden rule**: Inside Rayon parallel code, use only pure Rust operations
//! on primitive types (`i32`, `f64`, `bool`, etc.) and standard Rust collections.

use crate::IntoR;
use crate::ffi::{RNativeType, SEXP};
use crate::worker::with_r_thread;

#[cfg(feature = "rayon")]
pub use rayon;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

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
/// any R APIs inside the closure - they will panic on Rayon threads.
///
/// # Example
///
/// ```ignore
/// // Type is inferred from the closure parameter
/// let vec = with_r_vec(1000, |output: &mut [f64]| {
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
    // Validate length fits in R_xlen_t (i64 on 64-bit, i32 on 32-bit)
    #[cfg(target_pointer_width = "64")]
    assert!(
        len <= i64::MAX as usize,
        "with_r_vec: length {} exceeds R_xlen_t maximum",
        len
    );
    #[cfg(target_pointer_width = "32")]
    assert!(
        len <= i32::MAX as usize,
        "with_r_vec: length {} exceeds R_xlen_t maximum",
        len
    );

    struct UnprotectGuard;

    impl Drop for UnprotectGuard {
        fn drop(&mut self) {
            with_r_thread(move || unsafe {
                crate::ffi::Rf_unprotect_unchecked(1);
            });
        }
    }

    // Allocate, protect, and get data pointer on the R main thread
    use crate::worker::Sendable;
    let (sexp, Sendable(ptr)) = with_r_thread(move || unsafe {
        let sexp = crate::ffi::Rf_allocVector_unchecked(T::SEXP_TYPE, len as crate::ffi::R_xlen_t);
        crate::ffi::Rf_protect_unchecked(sexp);
        let ptr = T::dataptr_mut(sexp);
        (sexp, Sendable(ptr))
    });
    let _guard = UnprotectGuard;

    // Create slice (safe: vector is protected, pointer acquired on main thread)
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };

    // Run user's parallel work
    f(slice);

    sexp
}

/// Pre-allocate an R matrix, fill in parallel, return the SEXP.
///
/// This is the most efficient pattern for parallel matrix output - it writes directly
/// to R memory without intermediate copies.
///
/// # Thread Safety
///
/// This function can be called from either the worker thread or directly from
/// the R main thread. It uses [`with_r_thread`][crate::worker::with_r_thread]
/// internally to ensure R allocation happens on the correct thread.
///
/// **Critical**: The closure `f` must contain only pure Rust code. Do not call
/// any R APIs inside the closure - they will panic on Rayon threads.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rayon_bridge::{with_r_matrix, rayon::prelude::*};
///
/// // Create a 3x4 matrix, fill in parallel
/// let mat = with_r_matrix::<f64, _>(3, 4, |slice, nrow, ncol| {
///     // slice is in column-major order
///     slice.par_iter_mut()
///         .enumerate()
///         .for_each(|(i, slot)| {
///             let row = i % nrow;
///             let col = i / nrow;
///             *slot = (row * col) as f64;
///         });
/// });
/// ```
///
/// # Protection
///
/// The matrix is protected during the closure execution using `Rf_protect`.
/// After the function returns, the SEXP is unprotected and becomes the caller's
/// responsibility to protect.
#[cfg(feature = "rayon")]
pub fn with_r_matrix<T, F>(nrow: usize, ncol: usize, f: F) -> SEXP
where
    T: RNativeType + Send + Sync,
    F: FnOnce(&mut [T], usize, usize),
{
    // Validate dimensions fit in i32 (R uses int for matrix dims)
    assert!(
        nrow <= i32::MAX as usize,
        "with_r_matrix: nrow {} exceeds i32 maximum",
        nrow
    );
    assert!(
        ncol <= i32::MAX as usize,
        "with_r_matrix: ncol {} exceeds i32 maximum",
        ncol
    );

    // Checked multiply for total length
    let len = nrow
        .checked_mul(ncol)
        .expect("with_r_matrix: nrow * ncol overflow");

    // Validate total length fits in R_xlen_t
    #[cfg(target_pointer_width = "64")]
    assert!(
        len <= i64::MAX as usize,
        "with_r_matrix: total length {} exceeds R_xlen_t maximum",
        len
    );
    #[cfg(target_pointer_width = "32")]
    assert!(
        len <= i32::MAX as usize,
        "with_r_matrix: total length {} exceeds R_xlen_t maximum",
        len
    );

    struct UnprotectGuard;

    impl Drop for UnprotectGuard {
        fn drop(&mut self) {
            with_r_thread(move || unsafe {
                crate::ffi::Rf_unprotect_unchecked(1);
            });
        }
    }

    // Allocate, protect, and get data pointer on the R main thread
    use crate::worker::Sendable;
    let (sexp, Sendable(ptr)) = with_r_thread(move || unsafe {
        let sexp = crate::ffi::Rf_allocMatrix_unchecked(T::SEXP_TYPE, nrow as i32, ncol as i32);
        crate::ffi::Rf_protect_unchecked(sexp);
        let ptr = T::dataptr_mut(sexp);
        (sexp, Sendable(ptr))
    });
    let _guard = UnprotectGuard;

    // Create slice (safe: matrix is protected, pointer acquired on main thread)
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };

    // Run user's parallel work
    f(slice, nrow, ncol);

    sexp
}

/// Pre-allocate an N-dimensional R array, fill in parallel, return the SEXP.
///
/// This is the most efficient pattern for parallel array output - it writes directly
/// to R memory without intermediate copies.
///
/// # Thread Safety
///
/// This function can be called from either the worker thread or directly from
/// the R main thread. It uses [`with_r_thread`][crate::worker::with_r_thread]
/// internally to ensure R allocation happens on the correct thread.
///
/// **Critical**: The closure `f` must contain only pure Rust code. Do not call
/// any R APIs inside the closure - they will panic on Rayon threads.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rayon_bridge::{with_r_array, rayon::prelude::*};
///
/// // Create a 2x3x4 array, fill in parallel
/// let arr = with_r_array::<f64, 3, _>([2, 3, 4], |slice, dims| {
///     slice.par_iter_mut()
///         .enumerate()
///         .for_each(|(i, slot)| {
///             *slot = i as f64;
///         });
/// });
/// ```
///
/// # Protection
///
/// The array is protected during the closure execution using `Rf_protect`.
/// After the function returns, the SEXP is unprotected and becomes the caller's
/// responsibility to protect.
#[cfg(feature = "rayon")]
pub fn with_r_array<T, const NDIM: usize, F>(dims: [usize; NDIM], f: F) -> SEXP
where
    T: RNativeType + Send + Sync,
    F: FnOnce(&mut [T], [usize; NDIM]),
{
    // Validate each dimension fits in i32 (R uses int for dim attribute)
    for (i, &d) in dims.iter().enumerate() {
        assert!(
            d <= i32::MAX as usize,
            "with_r_array: dims[{}] = {} exceeds i32 maximum",
            i,
            d
        );
    }

    // Checked multiply for total length
    let total_len: usize = dims
        .iter()
        .try_fold(1usize, |acc, &d| acc.checked_mul(d))
        .expect("with_r_array: dimension product overflow");

    // Validate total length fits in R_xlen_t
    #[cfg(target_pointer_width = "64")]
    assert!(
        total_len <= i64::MAX as usize,
        "with_r_array: total length {} exceeds R_xlen_t maximum",
        total_len
    );
    #[cfg(target_pointer_width = "32")]
    assert!(
        total_len <= i32::MAX as usize,
        "with_r_array: total length {} exceeds R_xlen_t maximum",
        total_len
    );

    use crate::ffi::SEXPTYPE;

    struct UnprotectGuard(i32);

    impl Drop for UnprotectGuard {
        fn drop(&mut self) {
            let n = self.0;
            with_r_thread(move || unsafe {
                crate::ffi::Rf_unprotect_unchecked(n);
            });
        }
    }

    // Allocate, set dims, protect, and get data pointer on the R main thread
    use crate::worker::Sendable;
    let (sexp, Sendable(ptr)) = with_r_thread(move || unsafe {
        // Allocate the vector
        let sexp =
            crate::ffi::Rf_allocVector_unchecked(T::SEXP_TYPE, total_len as crate::ffi::R_xlen_t);
        crate::ffi::Rf_protect_unchecked(sexp);

        // Create and set dim attribute
        let dim_sexp =
            crate::ffi::Rf_allocVector_unchecked(SEXPTYPE::INTSXP, NDIM as crate::ffi::R_xlen_t);
        crate::ffi::Rf_protect_unchecked(dim_sexp);

        let dim_ptr = crate::ffi::INTEGER_unchecked(dim_sexp);
        for (i, &d) in dims.iter().enumerate() {
            *dim_ptr.add(i) = d as i32;
        }

        crate::ffi::Rf_setAttrib_unchecked(sexp, crate::ffi::R_DimSymbol, dim_sexp);
        crate::ffi::Rf_unprotect_unchecked(1); // unprotect dim_sexp, sexp stays protected

        let ptr = T::dataptr_mut(sexp);
        (sexp, Sendable(ptr))
    });
    let _guard = UnprotectGuard(1);

    // Create slice (safe: array is protected, pointer acquired on main thread)
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, total_len) };

    // Run user's parallel work
    f(slice, dims);

    sexp
}

/// Pre-allocate an R matrix with column-wise mutable access.
///
/// Unlike [`with_r_matrix`] which gives a flat slice, this function provides
/// an iterator over columns as disjoint mutable slices. This is optimal for
/// parallel column-wise operations since R matrices are stored column-major.
///
/// # Thread Safety
///
/// This function can be called from either the worker thread or directly from
/// the R main thread. It uses [`with_r_thread`][crate::worker::with_r_thread]
/// internally to ensure R allocation happens on the correct thread.
///
/// **Critical**: The closure `f` must contain only pure Rust code. Do not call
/// any R APIs inside the closure - they will panic on Rayon threads.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rayon_bridge::{with_r_matrix_cols, rayon::prelude::*};
///
/// // Create a 3x4 matrix, process columns in parallel
/// let mat = with_r_matrix_cols::<f64, _>(3, 4, |cols| {
///     cols.par_iter_mut()
///         .enumerate()
///         .for_each(|(col_idx, column)| {
///             // Each column is a &mut [f64] of length nrow
///             for (row_idx, val) in column.iter_mut().enumerate() {
///                 *val = (row_idx + col_idx * 10) as f64;
///             }
///         });
/// });
/// ```
///
/// # Protection
///
/// The matrix is protected during the closure execution using `Rf_protect`.
/// After the function returns, the SEXP is unprotected and becomes the caller's
/// responsibility to protect.
#[cfg(feature = "rayon")]
pub fn with_r_matrix_cols<T, F>(nrow: usize, ncol: usize, f: F) -> SEXP
where
    T: RNativeType + Send + Sync,
    F: FnOnce(std::slice::ChunksMut<'_, T>),
{
    // Validate dimensions fit in i32 (R uses int for matrix dims)
    assert!(
        nrow <= i32::MAX as usize,
        "with_r_matrix_cols: nrow {} exceeds i32 maximum",
        nrow
    );
    assert!(
        ncol <= i32::MAX as usize,
        "with_r_matrix_cols: ncol {} exceeds i32 maximum",
        ncol
    );

    // Checked multiply for total length
    let len = nrow
        .checked_mul(ncol)
        .expect("with_r_matrix_cols: nrow * ncol overflow");

    // Validate total length fits in R_xlen_t
    #[cfg(target_pointer_width = "64")]
    assert!(
        len <= i64::MAX as usize,
        "with_r_matrix_cols: total length {} exceeds R_xlen_t maximum",
        len
    );
    #[cfg(target_pointer_width = "32")]
    assert!(
        len <= i32::MAX as usize,
        "with_r_matrix_cols: total length {} exceeds R_xlen_t maximum",
        len
    );

    struct UnprotectGuard;

    impl Drop for UnprotectGuard {
        fn drop(&mut self) {
            with_r_thread(move || unsafe {
                crate::ffi::Rf_unprotect_unchecked(1);
            });
        }
    }

    // Allocate, protect, and get data pointer on the R main thread
    use crate::worker::Sendable;
    let (sexp, Sendable(ptr)) = with_r_thread(move || unsafe {
        let sexp = crate::ffi::Rf_allocMatrix_unchecked(T::SEXP_TYPE, nrow as i32, ncol as i32);
        crate::ffi::Rf_protect_unchecked(sexp);
        let ptr = T::dataptr_mut(sexp);
        (sexp, Sendable(ptr))
    });
    let _guard = UnprotectGuard;

    // Create slice and split into column chunks
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
    let cols = if nrow == 0 {
        // Edge case: empty matrix - create an empty chunks iterator
        slice.chunks_mut(1) // Won't produce any chunks since len=0
    } else {
        slice.chunks_mut(nrow)
    };

    // Run user's parallel work with column iterator
    f(cols);

    sexp
}

/// Pre-allocate an R matrix and return it as [`RMatrix<T>`][crate::rarray::RMatrix].
///
/// This is like [`with_r_matrix`] but returns a typed wrapper instead of raw SEXP.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rayon_bridge::{new_r_matrix, rayon::prelude::*};
/// use miniextendr_api::rarray::RMatrix;
///
/// let matrix: RMatrix<f64> = new_r_matrix(3, 4, |slice, nrow, ncol| {
///     slice.par_iter_mut()
///         .enumerate()
///         .for_each(|(i, slot)| *slot = i as f64);
/// });
/// ```
#[cfg(feature = "rayon")]
pub fn new_r_matrix<T, F>(nrow: usize, ncol: usize, f: F) -> crate::rarray::RMatrix<T>
where
    T: RNativeType + Send + Sync,
    F: FnOnce(&mut [T], usize, usize),
{
    let sexp = with_r_matrix::<T, F>(nrow, ncol, f);
    // Safety: we just allocated this with correct type and dims
    unsafe { crate::rarray::RMatrix::from_sexp_unchecked(sexp) }
}

/// Pre-allocate an R array with slab-wise mutable access along the last dimension.
///
/// Unlike [`with_r_array`] which gives a flat slice, this function provides
/// an iterator over "slabs" - contiguous slices along the last dimension.
/// This is optimal for parallel operations that process each slab independently.
///
/// For an array with dims `[d0, d1, ..., dN]`, each slab has `d0 * d1 * ... * d(N-1)`
/// elements and there are `dN` slabs total.
///
/// # Thread Safety
///
/// This function can be called from either the worker thread or directly from
/// the R main thread. It uses [`with_r_thread`][crate::worker::with_r_thread]
/// internally to ensure R allocation happens on the correct thread.
///
/// **Critical**: The closure `f` must contain only pure Rust code. Do not call
/// any R APIs inside the closure - they will panic on Rayon threads.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rayon_bridge::{with_r_array_slabs, rayon::prelude::*};
///
/// // Create a 2x3x4 array, process each of the 4 slabs (2x3 matrices) in parallel
/// let arr = with_r_array_slabs::<f64, 3, _>([2, 3, 4], |slabs, dims| {
///     slabs.par_iter_mut()
///         .enumerate()
///         .for_each(|(slab_idx, slab)| {
///             // Each slab is a &mut [f64] of length dims[0] * dims[1] = 6
///             for (i, val) in slab.iter_mut().enumerate() {
///                 *val = (slab_idx * 100 + i) as f64;
///             }
///         });
/// });
/// ```
///
/// # Protection
///
/// The array is protected during the closure execution using `Rf_protect`.
/// After the function returns, the SEXP is unprotected and becomes the caller's
/// responsibility to protect.
#[cfg(feature = "rayon")]
pub fn with_r_array_slabs<T, const NDIM: usize, F>(dims: [usize; NDIM], f: F) -> SEXP
where
    T: RNativeType + Send + Sync,
    F: FnOnce(std::slice::ChunksMut<'_, T>, [usize; NDIM]),
{
    // Validate each dimension fits in i32 (R uses int for dim attribute)
    for (i, &d) in dims.iter().enumerate() {
        assert!(
            d <= i32::MAX as usize,
            "with_r_array_slabs: dims[{}] = {} exceeds i32 maximum",
            i,
            d
        );
    }

    // Checked multiply for total length
    let total_len: usize = dims
        .iter()
        .try_fold(1usize, |acc, &d| acc.checked_mul(d))
        .expect("with_r_array_slabs: dimension product overflow");

    // Validate total length fits in R_xlen_t
    #[cfg(target_pointer_width = "64")]
    assert!(
        total_len <= i64::MAX as usize,
        "with_r_array_slabs: total length {} exceeds R_xlen_t maximum",
        total_len
    );
    #[cfg(target_pointer_width = "32")]
    assert!(
        total_len <= i32::MAX as usize,
        "with_r_array_slabs: total length {} exceeds R_xlen_t maximum",
        total_len
    );

    use crate::ffi::SEXPTYPE;

    struct UnprotectGuard(i32);

    impl Drop for UnprotectGuard {
        fn drop(&mut self) {
            let n = self.0;
            with_r_thread(move || unsafe {
                crate::ffi::Rf_unprotect_unchecked(n);
            });
        }
    }

    // Allocate, set dims, protect, and get data pointer on the R main thread
    use crate::worker::Sendable;
    let (sexp, Sendable(ptr)) = with_r_thread(move || unsafe {
        // Allocate the vector
        let sexp =
            crate::ffi::Rf_allocVector_unchecked(T::SEXP_TYPE, total_len as crate::ffi::R_xlen_t);
        crate::ffi::Rf_protect_unchecked(sexp);

        // Create and set dim attribute
        let dim_sexp =
            crate::ffi::Rf_allocVector_unchecked(SEXPTYPE::INTSXP, NDIM as crate::ffi::R_xlen_t);
        crate::ffi::Rf_protect_unchecked(dim_sexp);

        let dim_ptr = crate::ffi::INTEGER_unchecked(dim_sexp);
        for (i, &d) in dims.iter().enumerate() {
            *dim_ptr.add(i) = d as i32;
        }

        crate::ffi::Rf_setAttrib_unchecked(sexp, crate::ffi::R_DimSymbol, dim_sexp);
        crate::ffi::Rf_unprotect_unchecked(1); // unprotect dim_sexp, sexp stays protected

        let ptr = T::dataptr_mut(sexp);
        (sexp, Sendable(ptr))
    });
    let _guard = UnprotectGuard(1);

    // Calculate slab size (product of all dims except the last)
    let slab_size: usize = if NDIM <= 1 {
        total_len
    } else {
        dims[..NDIM - 1].iter().product()
    };

    // Create slice and split into slab chunks
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, total_len) };
    let slabs = if slab_size == 0 {
        slice.chunks_mut(1) // Edge case: won't produce chunks if total_len=0
    } else {
        slice.chunks_mut(slab_size)
    };

    // Run user's parallel work with slab iterator
    f(slabs, dims);

    sexp
}

/// Pre-allocate an N-dimensional R array and return it as [`RArray<T, NDIM>`][crate::rarray::RArray].
///
/// This is like [`with_r_array`] but returns a typed wrapper instead of raw SEXP.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rayon_bridge::{new_r_array, rayon::prelude::*};
/// use miniextendr_api::rarray::RArray;
///
/// let array: RArray<f64, 3> = new_r_array([2, 3, 4], |slice, dims| {
///     slice.par_iter_mut()
///         .enumerate()
///         .for_each(|(i, slot)| *slot = i as f64);
/// });
/// ```
#[cfg(feature = "rayon")]
pub fn new_r_array<T, const NDIM: usize, F>(
    dims: [usize; NDIM],
    f: F,
) -> crate::rarray::RArray<T, NDIM>
where
    T: RNativeType + Send + Sync,
    F: FnOnce(&mut [T], [usize; NDIM]),
{
    let sexp = with_r_array::<T, NDIM, F>(dims, f);
    // Safety: we just allocated this with correct type and dims
    unsafe { crate::rarray::RArray::from_sexp_unchecked(sexp) }
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

// region: Adapter Traits

/// Adapter trait for exposing parallel iteration operations to R.
///
/// This trait provides a way to expose Rayon's parallel iteration capabilities
/// to R through the `miniextendr_module!` macro. Unlike the standard `ParallelIterator`
/// trait, this adapter is designed to work with `ExternalPtr<T>` which only provides
/// `&self` access.
///
/// # Design
///
/// The trait is designed around non-consuming parallel operations:
/// - Aggregations (sum, min, max, mean, count)
/// - Predicates (any, all, find)
/// - Transformations that return new collections (map, filter)
///
/// # Interior Mutability
///
/// Since `ExternalPtr` provides `&self` and parallel iteration typically works
/// with owned data or `&[T]` slices, implementations should either:
/// 1. Store data in a way that allows parallel access (e.g., `Vec<T>`, `[T]`)
/// 2. Use interior mutability if the iteration consumes cached state
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rayon_bridge::RParallelIterator;
/// use miniextendr_api::ExternalPtr;
///
/// #[derive(ExternalPtr)]
/// struct ParallelData {
///     values: Vec<f64>,
/// }
///
/// impl RParallelIterator for ParallelData {
///     type Item = f64;
///
///     fn par_iter(&self) -> impl rayon::iter::ParallelIterator<Item = Self::Item> + '_ {
///         self.values.par_iter().copied()
///     }
/// }
///
/// #[miniextendr]
/// impl RParallelIterator for ParallelData {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RParallelIterator for ParallelData;
/// }
/// ```
///
/// In R:
/// ```r
/// data <- ParallelData$new(as.numeric(1:1000000))
/// data$r_par_sum()      # Fast parallel sum
/// data$r_par_mean()     # Parallel mean
/// data$r_par_min()      # Parallel minimum
/// data$r_par_count()    # Count elements
/// ```
#[cfg(feature = "rayon")]
pub trait RParallelIterator {
    /// The element type produced by the parallel iterator.
    type Item: Send + Sync + Copy;

    /// Returns a parallel iterator over the elements.
    ///
    /// Implementations should return an iterator that yields `Self::Item` values.
    fn par_iter(&self) -> impl rayon::iter::ParallelIterator<Item = Self::Item> + '_;

    /// Returns the number of elements, if known.
    ///
    /// Default implementation returns -1 (unknown).
    fn par_len(&self) -> i32 {
        -1
    }

    /// Computes the parallel sum of f64 elements.
    ///
    /// Default implementation requires `Self::Item` to be convertible to f64.
    fn par_sum(&self) -> f64
    where
        Self::Item: Into<f64>,
    {
        self.par_iter().map(|x| x.into()).sum()
    }

    /// Computes the parallel sum of i32 elements.
    fn par_sum_int(&self) -> i32
    where
        Self::Item: Into<i32>,
    {
        self.par_iter().map(|x| x.into()).sum()
    }

    /// Computes the parallel sum of i64 elements (returned as f64 for R).
    fn par_sum_i64(&self) -> f64
    where
        Self::Item: Into<i64>,
    {
        self.par_iter().map(|x| x.into()).sum::<i64>() as f64
    }

    /// Computes the parallel mean of f64 elements.
    fn par_mean(&self) -> f64
    where
        Self::Item: Into<f64>,
    {
        let (sum, count) = self
            .par_iter()
            .map(|x| (x.into(), 1usize))
            .reduce(|| (0.0, 0), |(s1, c1), (s2, c2)| (s1 + s2, c1 + c2));

        if count == 0 {
            f64::NAN
        } else {
            sum / count as f64
        }
    }

    /// Finds the parallel minimum.
    fn par_min(&self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        self.par_iter().min()
    }

    /// Finds the parallel maximum.
    fn par_max(&self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        self.par_iter().max()
    }

    /// Finds the parallel minimum f64 (handles NaN).
    fn par_min_f64(&self) -> f64
    where
        Self::Item: Into<f64>,
    {
        self.par_iter()
            .map(|x| x.into())
            .reduce(|| f64::INFINITY, |a, b| a.min(b))
    }

    /// Finds the parallel maximum f64 (handles NaN).
    fn par_max_f64(&self) -> f64
    where
        Self::Item: Into<f64>,
    {
        self.par_iter()
            .map(|x| x.into())
            .reduce(|| f64::NEG_INFINITY, |a, b| a.max(b))
    }

    /// Counts the number of elements in parallel.
    fn par_count(&self) -> i32 {
        self.par_iter().count() as i32
    }

    /// Computes the parallel product of f64 elements.
    fn par_product(&self) -> f64
    where
        Self::Item: Into<f64>,
    {
        self.par_iter().map(|x| x.into()).product()
    }

    /// Returns true if any element satisfies the predicate (greater than threshold).
    fn par_any_gt(&self, threshold: f64) -> bool
    where
        Self::Item: Into<f64>,
    {
        self.par_iter().any(|x| x.into() > threshold)
    }

    /// Returns true if all elements satisfy the predicate (greater than threshold).
    fn par_all_gt(&self, threshold: f64) -> bool
    where
        Self::Item: Into<f64>,
    {
        self.par_iter().all(|x| x.into() > threshold)
    }

    /// Returns true if any element satisfies the predicate (less than threshold).
    fn par_any_lt(&self, threshold: f64) -> bool
    where
        Self::Item: Into<f64>,
    {
        self.par_iter().any(|x| x.into() < threshold)
    }

    /// Returns true if all elements satisfy the predicate (less than threshold).
    fn par_all_lt(&self, threshold: f64) -> bool
    where
        Self::Item: Into<f64>,
    {
        self.par_iter().all(|x| x.into() < threshold)
    }

    /// Counts elements greater than threshold.
    fn par_count_gt(&self, threshold: f64) -> i32
    where
        Self::Item: Into<f64>,
    {
        self.par_iter().filter(|&x| x.into() > threshold).count() as i32
    }

    /// Counts elements less than threshold.
    fn par_count_lt(&self, threshold: f64) -> i32
    where
        Self::Item: Into<f64>,
    {
        self.par_iter().filter(|&x| x.into() < threshold).count() as i32
    }

    /// Counts elements equal to value (within epsilon for floats).
    fn par_count_eq(&self, value: f64, epsilon: f64) -> i32
    where
        Self::Item: Into<f64>,
    {
        self.par_iter()
            .filter(|&x| (x.into() - value).abs() <= epsilon)
            .count() as i32
    }

    /// Computes variance in parallel.
    fn par_variance(&self) -> f64
    where
        Self::Item: Into<f64>,
    {
        let mean = self.par_mean();
        if mean.is_nan() {
            return f64::NAN;
        }

        let (sum_sq_diff, count) = self
            .par_iter()
            .map(|x| {
                let diff = x.into() - mean;
                (diff * diff, 1usize)
            })
            .reduce(|| (0.0, 0), |(s1, c1), (s2, c2)| (s1 + s2, c1 + c2));

        if count <= 1 {
            f64::NAN
        } else {
            sum_sq_diff / (count - 1) as f64 // Bessel's correction
        }
    }

    /// Computes standard deviation in parallel.
    fn par_std_dev(&self) -> f64
    where
        Self::Item: Into<f64>,
    {
        self.par_variance().sqrt()
    }

    /// Collects elements greater than threshold into a Vec<f64>.
    fn par_filter_gt(&self, threshold: f64) -> Vec<f64>
    where
        Self::Item: Into<f64>,
    {
        self.par_iter()
            .filter(|&x| x.into() > threshold)
            .map(|x| x.into())
            .collect()
    }

    /// Collects elements less than threshold into a Vec<f64>.
    fn par_filter_lt(&self, threshold: f64) -> Vec<f64>
    where
        Self::Item: Into<f64>,
    {
        self.par_iter()
            .filter(|&x| x.into() < threshold)
            .map(|x| x.into())
            .collect()
    }

    /// Applies a scalar operation and collects results (multiply by factor).
    fn par_scale(&self, factor: f64) -> Vec<f64>
    where
        Self::Item: Into<f64>,
    {
        self.par_iter().map(|x| x.into() * factor).collect()
    }

    /// Applies offset and collects results (add offset).
    fn par_offset(&self, offset: f64) -> Vec<f64>
    where
        Self::Item: Into<f64>,
    {
        self.par_iter().map(|x| x.into() + offset).collect()
    }

    /// Clamps values to range and collects results.
    fn par_clamp(&self, min: f64, max: f64) -> Vec<f64>
    where
        Self::Item: Into<f64>,
    {
        self.par_iter().map(|x| x.into().clamp(min, max)).collect()
    }

    /// Applies absolute value and collects results.
    fn par_abs(&self) -> Vec<f64>
    where
        Self::Item: Into<f64>,
    {
        self.par_iter().map(|x| x.into().abs()).collect()
    }

    /// Applies square root and collects results.
    fn par_sqrt(&self) -> Vec<f64>
    where
        Self::Item: Into<f64>,
    {
        self.par_iter().map(|x| x.into().sqrt()).collect()
    }

    /// Applies power and collects results.
    fn par_pow(&self, exp: f64) -> Vec<f64>
    where
        Self::Item: Into<f64>,
    {
        self.par_iter().map(|x| x.into().powf(exp)).collect()
    }

    /// Applies natural log and collects results.
    fn par_ln(&self) -> Vec<f64>
    where
        Self::Item: Into<f64>,
    {
        self.par_iter().map(|x| x.into().ln()).collect()
    }

    /// Applies exp and collects results.
    fn par_exp(&self) -> Vec<f64>
    where
        Self::Item: Into<f64>,
    {
        self.par_iter().map(|x| x.into().exp()).collect()
    }
}

/// Adapter trait for parallel collection extension.
///
/// This trait provides a way to extend collections in parallel, useful for
/// building up large collections from multiple sources efficiently.
///
/// # Interior Mutability
///
/// Since `ExternalPtr` only provides `&self`, implementations must use interior
/// mutability (e.g., `RefCell`, `Mutex`) to allow modification.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rayon_bridge::RParallelExtend;
/// use std::sync::Mutex;
///
/// #[derive(ExternalPtr)]
/// struct ParallelBuffer {
///     data: Mutex<Vec<f64>>,
/// }
///
/// impl RParallelExtend<f64> for ParallelBuffer {
///     fn par_extend(&self, items: Vec<f64>) {
///         // Use par_extend from rayon
///         let mut guard = self.data.lock().unwrap();
///         guard.par_extend(items);
///     }
/// }
///
/// #[miniextendr]
/// impl RParallelExtend<f64> for ParallelBuffer {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RParallelExtend<f64> for ParallelBuffer;
/// }
/// ```
#[cfg(feature = "rayon")]
pub trait RParallelExtend<T: Send> {
    /// Extends the collection with items from a Vec in parallel.
    fn par_extend(&self, items: Vec<T>);

    /// Extends the collection with items from a slice (clones items).
    fn par_extend_from_slice(&self, items: &[T])
    where
        T: Clone + Sync,
    {
        self.par_extend(items.to_vec());
    }

    /// Returns the current length of the collection.
    ///
    /// Default implementation returns -1 (unknown).
    fn par_len(&self) -> i32 {
        -1
    }

    /// Returns true if the collection is empty.
    fn par_is_empty(&self) -> bool {
        self.par_len() == 0
    }

    /// Clears the collection.
    ///
    /// Default implementation does nothing.
    fn par_clear(&self) {}

    /// Reserves capacity for at least `additional` more elements.
    ///
    /// Default implementation does nothing.
    fn par_reserve(&self, _additional: i32) {}
}

// endregion

#[cfg(all(test, feature = "rayon"))]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_collect() {
        use rayon::prelude::*;

        let result: Vec<i32> = (0..100).into_par_iter().collect();
        assert_eq!(result.len(), 100);
        assert_eq!(result[0], 0);
        assert_eq!(result[99], 99);
    }

    #[test]
    fn test_parallel_map() {
        use rayon::prelude::*;

        let data = vec![1.0, 2.0, 3.0, 4.0];
        let doubled: Vec<f64> = data.par_iter().map(|&x| x * 2.0).collect();

        assert_eq!(doubled.as_slice(), &[2.0, 4.0, 6.0, 8.0]);
    }

    // Test implementation for RParallelIterator
    struct TestParData {
        values: Vec<f64>,
    }

    impl TestParData {
        fn new(values: Vec<f64>) -> Self {
            Self { values }
        }
    }

    impl RParallelIterator for TestParData {
        type Item = f64;

        fn par_iter(&self) -> impl rayon::iter::ParallelIterator<Item = Self::Item> + '_ {
            self.values.par_iter().copied()
        }

        fn par_len(&self) -> i32 {
            self.values.len() as i32
        }
    }

    #[test]
    fn test_rparalleliterator_sum() {
        let data = TestParData::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(data.par_sum(), 15.0);
    }

    #[test]
    fn test_rparalleliterator_mean() {
        let data = TestParData::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(data.par_mean(), 3.0);
    }

    #[test]
    fn test_rparalleliterator_mean_empty() {
        let data = TestParData::new(vec![]);
        assert!(data.par_mean().is_nan());
    }

    #[test]
    fn test_rparalleliterator_min_max() {
        let data = TestParData::new(vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0]);
        assert_eq!(data.par_min_f64(), 1.0);
        assert_eq!(data.par_max_f64(), 9.0);
    }

    #[test]
    fn test_rparalleliterator_count() {
        let data = TestParData::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(data.par_count(), 5);
        assert_eq!(data.par_len(), 5);
    }

    #[test]
    fn test_rparalleliterator_product() {
        let data = TestParData::new(vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(data.par_product(), 24.0);
    }

    #[test]
    fn test_rparalleliterator_predicates() {
        let data = TestParData::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);

        assert!(data.par_any_gt(4.0)); // 5.0 > 4.0
        assert!(!data.par_all_gt(4.0)); // 1.0, 2.0, 3.0, 4.0 are not > 4.0
        assert!(data.par_any_lt(2.0)); // 1.0 < 2.0
        assert!(!data.par_all_lt(2.0)); // 2.0, 3.0, 4.0, 5.0 are not < 2.0
    }

    #[test]
    fn test_rparalleliterator_count_predicates() {
        let data = TestParData::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);

        assert_eq!(data.par_count_gt(3.0), 2); // 4.0, 5.0
        assert_eq!(data.par_count_lt(3.0), 2); // 1.0, 2.0
        assert_eq!(data.par_count_eq(3.0, 0.0), 1); // exactly 3.0
    }

    #[test]
    fn test_rparalleliterator_variance_stddev() {
        let data = TestParData::new(vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
        let var = data.par_variance();
        let std = data.par_std_dev();

        // Variance should be approximately 4.571 (sample variance)
        assert!((var - 4.571428571).abs() < 0.001);
        assert!((std - var.sqrt()).abs() < 0.001);
    }

    #[test]
    fn test_rparalleliterator_filter() {
        let data = TestParData::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);

        let gt3: Vec<f64> = data.par_filter_gt(3.0);
        assert_eq!(gt3.len(), 2);
        assert!(gt3.contains(&4.0));
        assert!(gt3.contains(&5.0));

        let lt3: Vec<f64> = data.par_filter_lt(3.0);
        assert_eq!(lt3.len(), 2);
        assert!(lt3.contains(&1.0));
        assert!(lt3.contains(&2.0));
    }

    #[test]
    fn test_rparalleliterator_transform() {
        let data = TestParData::new(vec![1.0, 2.0, 3.0, 4.0]);

        let scaled = data.par_scale(2.0);
        assert_eq!(scaled, vec![2.0, 4.0, 6.0, 8.0]);

        let offset = data.par_offset(10.0);
        assert_eq!(offset, vec![11.0, 12.0, 13.0, 14.0]);

        let clamped = data.par_clamp(2.0, 3.0);
        assert_eq!(clamped, vec![2.0, 2.0, 3.0, 3.0]);
    }

    #[test]
    fn test_rparalleliterator_math() {
        let data = TestParData::new(vec![1.0, 4.0, 9.0, 16.0]);

        let sqrt = data.par_sqrt();
        assert_eq!(sqrt, vec![1.0, 2.0, 3.0, 4.0]);

        let pow2 = data.par_pow(0.5);
        assert_eq!(pow2, vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_rparalleliterator_abs() {
        let data = TestParData::new(vec![-1.0, 2.0, -3.0, 4.0]);
        let abs = data.par_abs();
        assert_eq!(abs, vec![1.0, 2.0, 3.0, 4.0]);
    }

    // Test implementation for RParallelExtend
    use std::sync::Mutex;

    struct TestParBuffer {
        data: Mutex<Vec<f64>>,
    }

    impl TestParBuffer {
        fn new() -> Self {
            Self {
                data: Mutex::new(Vec::new()),
            }
        }

        fn get_data(&self) -> Vec<f64> {
            self.data.lock().unwrap().clone()
        }
    }

    impl RParallelExtend<f64> for TestParBuffer {
        fn par_extend(&self, items: Vec<f64>) {
            let mut guard = self.data.lock().unwrap();
            guard.extend(items);
        }

        fn par_len(&self) -> i32 {
            self.data.lock().unwrap().len() as i32
        }

        fn par_clear(&self) {
            self.data.lock().unwrap().clear();
        }

        fn par_reserve(&self, additional: i32) {
            self.data.lock().unwrap().reserve(additional as usize);
        }
    }

    #[test]
    fn test_rparallelextend_basic() {
        let buffer = TestParBuffer::new();

        buffer.par_extend(vec![1.0, 2.0, 3.0]);
        assert_eq!(buffer.par_len(), 3);
        assert_eq!(buffer.get_data(), vec![1.0, 2.0, 3.0]);

        buffer.par_extend(vec![4.0, 5.0]);
        assert_eq!(buffer.par_len(), 5);
        assert_eq!(buffer.get_data(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_rparallelextend_from_slice() {
        let buffer = TestParBuffer::new();
        let slice = [1.0, 2.0, 3.0];

        buffer.par_extend_from_slice(&slice);
        assert_eq!(buffer.get_data(), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_rparallelextend_clear() {
        let buffer = TestParBuffer::new();

        buffer.par_extend(vec![1.0, 2.0, 3.0]);
        assert!(!buffer.par_is_empty());

        buffer.par_clear();
        assert!(buffer.par_is_empty());
        assert_eq!(buffer.par_len(), 0);
    }

    #[test]
    fn test_rparallelextend_reserve() {
        let buffer = TestParBuffer::new();
        buffer.par_reserve(100);
        // Can't easily verify capacity, but at least it shouldn't panic
        buffer.par_extend(vec![1.0; 100]);
        assert_eq!(buffer.par_len(), 100);
    }
}
