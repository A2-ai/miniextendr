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
//! # Chunk-Based Parallel Fill
//!
//! The `with_r_*` functions handle parallelism internally. Your closure receives
//! a **chunk** (mutable slice) and its **offset** (starting index). The framework
//! splits the data and dispatches chunks to Rayon threads automatically.
//!
//! ```ignore
//! // Fill a vector: closure receives (chunk, offset)
//! with_r_vec(1000, |chunk: &mut [f64], offset: usize| {
//!     for (i, slot) in chunk.iter_mut().enumerate() {
//!         *slot = ((offset + i) as f64).sqrt();
//!     }
//! });
//!
//! // Even simpler: element-wise map
//! with_r_vec_map(1000, |i: usize| (i as f64).sqrt());
//!
//! // Matrix: closure receives (column, col_idx)
//! with_r_matrix(100, 50, |col: &mut [f64], col_idx: usize| {
//!     for (row, slot) in col.iter_mut().enumerate() {
//!         *slot = (row + col_idx * 1000) as f64;
//!     }
//! });
//! ```
//!
//! **Chunk boundaries are deterministic** for a given vector length and thread count.
//! This makes parallel RNG reproducible — seed per-chunk from `offset`:
//!
//! ```ignore
//! with_r_vec(len, |chunk, offset| {
//!     let mut rng = ChaChaRng::seed_from_u64(base_seed + offset as u64);
//!     for slot in chunk { *slot = rng.gen(); }
//! });
//! ```
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
//! // FAST: Chunk-based parallel fill (framework handles par_chunks_mut)
//! with_r_vec(data.len(), |chunk: &mut [f64], offset| {
//!     for (i, slot) in chunk.iter_mut().enumerate() {
//!         *slot = data[offset + i].sqrt();
//!     }
//! })
//!
//! // FAST: Element-wise parallel map
//! with_r_vec_map(data.len(), |i| data[i].sqrt())
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
//! Rayon threads. For parallel RNG, use chunk-based seeding for reproducibility:
//!
//! ```ignore
//! // REPRODUCIBLE: Each chunk gets a deterministic seed from its offset
//! with_r_vec(len, |chunk, offset| {
//!     let mut rng = ChaChaRng::seed_from_u64(base_seed + offset as u64);
//!     for slot in chunk {
//!         *slot = rng.gen();
//!     }
//! });
//!
//! // PANIC: RRng cannot be used inside closures (calls R APIs)
//! with_r_vec(len, |chunk, _| {
//!     let mut rng = RRng::new();  // PANICS! R API not available
//!     for slot in chunk { *slot = rng.uniform_f64(); }
//! });
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
use crate::ffi::{RNativeType, SEXP, SexpExt};
use crate::worker::with_r_thread;

#[cfg(feature = "rayon")]
pub use rayon;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

// region: Chunk-based parallel fill

/// Pre-allocate an R vector, fill chunks in parallel, return the SEXP.
///
/// The closure `f(chunk, offset)` is called in parallel for each chunk:
/// - `chunk`: mutable slice to fill
/// - `offset`: starting index of this chunk in the overall vector
///
/// The framework handles splitting and parallel dispatch via Rayon's
/// `par_chunks_mut`. **Chunk boundaries are deterministic** for a given
/// vector length and thread count, making parallel RNG reproducible.
///
/// # Type Mapping
///
/// `T` must implement [`RNativeType`]:
/// - `f64` → `REALSXP`
/// - `i32` → `INTSXP`
/// - `RLogical` → `LGLSXP`
/// - `u8` → `RAWSXP`
/// - `Rcomplex` → `CPLXSXP`
///
/// # Example
///
/// ```ignore
/// // Fill with sqrt(index) — framework handles parallelism
/// with_r_vec(1000, |chunk: &mut [f64], offset: usize| {
///     for (i, slot) in chunk.iter_mut().enumerate() {
///         *slot = ((offset + i) as f64).sqrt();
///     }
/// });
///
/// // Reproducible parallel RNG (seed from offset)
/// with_r_vec(1000, |chunk: &mut [f64], offset| {
///     let mut rng = ChaChaRng::seed_from_u64(42 + offset as u64);
///     for slot in chunk { *slot = rng.gen(); }
/// });
/// ```
///
/// # Protection
///
/// The vector is PROTECTED during parallel writes. After return, UNPROTECT
/// is called and the SEXP becomes the caller's responsibility.
#[cfg(feature = "rayon")]
pub fn with_r_vec<T, F>(len: usize, f: F) -> SEXP
where
    T: RNativeType + Send + Sync,
    F: Fn(&mut [T], usize) + Send + Sync,
{
    // Validate length fits in R_xlen_t
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

    // Allocate, protect, and get data pointer on the R main thread
    use crate::worker::Sendable;
    let (sexp, Sendable(ptr)) = with_r_thread(move || unsafe {
        let sexp = crate::ffi::Rf_allocVector_unchecked(T::SEXP_TYPE, len as crate::ffi::R_xlen_t);
        crate::ffi::Rf_protect_unchecked(sexp);
        let ptr = T::dataptr_mut(sexp);
        (sexp, Sendable(ptr))
    });
    let _guard = crate::gc_protect::WorkerUnprotectGuard::new(1);

    // Create slice and dispatch chunks in parallel
    let slice = unsafe { crate::from_r::r_slice_mut(ptr, len) };

    if len > 0 {
        let chunk_size = std::cmp::max(1, len / (rayon::current_num_threads() * 4));
        slice
            .par_chunks_mut(chunk_size)
            .enumerate()
            .for_each(|(chunk_idx, chunk)| {
                f(chunk, chunk_idx * chunk_size);
            });
    }

    sexp
}

/// Pre-allocate an R vector, fill element-wise in parallel, return the SEXP.
///
/// This is syntactic sugar over [`with_r_vec`] for the common case where each
/// element depends only on its index.
///
/// # Example
///
/// ```ignore
/// // Fill with sqrt(index)
/// with_r_vec_map(1000, |i: usize| (i as f64).sqrt());
/// ```
#[cfg(feature = "rayon")]
pub fn with_r_vec_map<T, F>(len: usize, f: F) -> SEXP
where
    T: RNativeType + Send + Sync,
    F: Fn(usize) -> T + Send + Sync,
{
    with_r_vec(len, |chunk, offset| {
        for (i, slot) in chunk.iter_mut().enumerate() {
            *slot = f(offset + i);
        }
    })
}

/// Transform an input slice into a new R vector, element-wise in parallel.
///
/// Allocates an output R vector of the same length as `input`, then fills it
/// in parallel using chunk-based dispatch. Each element `output[i] = f(&input[i])`.
///
/// This is the parallel equivalent of `input.iter().map(f).collect::<Vec<U>>()`,
/// but writes directly into R memory (zero intermediate allocation).
///
/// # Example
///
/// ```ignore
/// // Parallel sqrt of an R numeric vector
/// fn parallel_sqrt(x: &[f64]) -> SEXP {
///     par_map(x, |&v| v.sqrt())
/// }
///
/// // Type conversion: i32 → f64
/// fn int_to_double(x: &[i32]) -> SEXP {
///     par_map(x, |&v| v as f64)
/// }
/// ```
#[cfg(feature = "rayon")]
pub fn par_map<T, U, F>(input: &[T], f: F) -> SEXP
where
    T: Send + Sync,
    U: RNativeType + Send + Sync,
    F: Fn(&T) -> U + Send + Sync,
{
    let len = input.len();
    with_r_vec(len, |chunk, offset| {
        for (i, slot) in chunk.iter_mut().enumerate() {
            *slot = f(&input[offset + i]);
        }
    })
}

/// Two-input element-wise parallel map into a new R vector.
///
/// Like [`par_map`] but zips two input slices: `output[i] = f(&a[i], &b[i])`.
///
/// # Panics
///
/// Panics if `a.len() != b.len()`.
///
/// # Example
///
/// ```ignore
/// // Parallel element-wise addition
/// fn vec_add(a: &[f64], b: &[f64]) -> SEXP {
///     par_map2(a, b, |&x, &y| x + y)
/// }
///
/// // Weighted transform
/// fn weighted_sqrt(values: &[f64], weights: &[f64]) -> SEXP {
///     par_map2(values, weights, |&v, &w| v.sqrt() * w)
/// }
/// ```
#[cfg(feature = "rayon")]
pub fn par_map2<T, U, V, F>(a: &[T], b: &[U], f: F) -> SEXP
where
    T: Send + Sync,
    U: Send + Sync,
    V: RNativeType + Send + Sync,
    F: Fn(&T, &U) -> V + Send + Sync,
{
    assert_eq!(
        a.len(),
        b.len(),
        "par_map2: input slices must have equal length ({} vs {})",
        a.len(),
        b.len()
    );
    let len = a.len();
    with_r_vec(len, |chunk, offset| {
        for (i, slot) in chunk.iter_mut().enumerate() {
            *slot = f(&a[offset + i], &b[offset + i]);
        }
    })
}

/// Three-input element-wise parallel map into a new R vector.
///
/// Like [`par_map2`] but zips three input slices: `output[i] = f(&a[i], &b[i], &c[i])`.
///
/// # Panics
///
/// Panics if the three slices have different lengths.
///
/// # Example
///
/// ```ignore
/// // Fused multiply-add: a * b + c
/// fn fma(a: &[f64], b: &[f64], c: &[f64]) -> SEXP {
///     par_map3(a, b, c, |&x, &y, &z| x * y + z)
/// }
/// ```
#[cfg(feature = "rayon")]
pub fn par_map3<A, B, C, V, F>(a: &[A], b: &[B], c: &[C], f: F) -> SEXP
where
    A: Send + Sync,
    B: Send + Sync,
    C: Send + Sync,
    V: RNativeType + Send + Sync,
    F: Fn(&A, &B, &C) -> V + Send + Sync,
{
    assert!(
        a.len() == b.len() && b.len() == c.len(),
        "par_map3: input slices must have equal length ({}, {}, {})",
        a.len(),
        b.len(),
        c.len()
    );
    let len = a.len();
    with_r_vec(len, |chunk, offset| {
        for (i, slot) in chunk.iter_mut().enumerate() {
            *slot = f(&a[offset + i], &b[offset + i], &c[offset + i]);
        }
    })
}

/// Pre-allocate an R matrix, fill columns in parallel, return the SEXP.
///
/// The closure `f(column, col_idx)` is called in parallel for each column:
/// - `column`: mutable slice of length `nrow` (one column)
/// - `col_idx`: column index (0-based)
///
/// R matrices are column-major, so each column is a contiguous slice.
///
/// # Example
///
/// ```ignore
/// // Create a 100x50 matrix, fill by column
/// with_r_matrix(100, 50, |col: &mut [f64], col_idx: usize| {
///     for (row, slot) in col.iter_mut().enumerate() {
///         *slot = (row + col_idx * 1000) as f64;
///     }
/// });
/// ```
///
/// # Protection
///
/// The matrix is PROTECTED during parallel writes. After return, UNPROTECT
/// is called and the SEXP becomes the caller's responsibility.
#[cfg(feature = "rayon")]
pub fn with_r_matrix<T, F>(nrow: usize, ncol: usize, f: F) -> SEXP
where
    T: RNativeType + Send + Sync,
    F: Fn(&mut [T], usize) + Send + Sync,
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

    // Allocate, protect, and get data pointer on the R main thread
    use crate::worker::Sendable;
    let (sexp, Sendable(ptr)) = with_r_thread(move || unsafe {
        let sexp = crate::ffi::Rf_allocMatrix_unchecked(T::SEXP_TYPE, nrow as i32, ncol as i32);
        crate::ffi::Rf_protect_unchecked(sexp);
        let ptr = T::dataptr_mut(sexp);
        (sexp, Sendable(ptr))
    });
    let _guard = crate::gc_protect::WorkerUnprotectGuard::new(1);

    // Split into columns and dispatch in parallel
    let slice = unsafe { crate::from_r::r_slice_mut(ptr, len) };

    if nrow > 0 && ncol > 0 {
        slice
            .par_chunks_mut(nrow)
            .enumerate()
            .for_each(|(col_idx, col)| {
                f(col, col_idx);
            });
    }

    sexp
}

/// Pre-allocate an N-dimensional R array, fill slabs in parallel, return the SEXP.
///
/// The closure `f(slab, slab_idx)` is called in parallel for each slab:
/// - `slab`: mutable slice of one slab (product of all dims except the last)
/// - `slab_idx`: slab index along the last dimension (0-based)
///
/// For dims `[d0, d1, ..., dN]`, each slab has `d0 * d1 * ... * d(N-1)` elements
/// and there are `dN` slabs total.
///
/// # Example
///
/// ```ignore
/// // Create a 2x3x4 array, 4 slabs of 6 elements each
/// with_r_array([2, 3, 4], |slab: &mut [f64], slab_idx: usize| {
///     for (i, val) in slab.iter_mut().enumerate() {
///         *val = (slab_idx * 100 + i) as f64;
///     }
/// });
/// ```
///
/// # Protection
///
/// The array is PROTECTED during parallel writes. After return, UNPROTECT
/// is called and the SEXP becomes the caller's responsibility.
#[cfg(feature = "rayon")]
pub fn with_r_array<T, const NDIM: usize, F>(dims: [usize; NDIM], f: F) -> SEXP
where
    T: RNativeType + Send + Sync,
    F: Fn(&mut [T], usize) + Send + Sync,
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

    // Allocate, set dims, protect, and get data pointer on the R main thread
    use crate::worker::Sendable;
    let (sexp, Sendable(ptr)) = with_r_thread(move || unsafe {
        let sexp =
            crate::ffi::Rf_allocVector_unchecked(T::SEXP_TYPE, total_len as crate::ffi::R_xlen_t);
        crate::ffi::Rf_protect_unchecked(sexp);

        let (dim_sexp, dim_s) = crate::into_r::alloc_r_vector_unchecked::<i32>(NDIM);
        crate::ffi::Rf_protect_unchecked(dim_sexp);

        for (slot, &d) in dim_s.iter_mut().zip(dims.iter()) {
            *slot = d as i32;
        }

        sexp.set_attr_unchecked(crate::ffi::R_DimSymbol, dim_sexp);
        crate::ffi::Rf_unprotect_unchecked(1); // unprotect dim_sexp, sexp stays protected

        let ptr = T::dataptr_mut(sexp);
        (sexp, Sendable(ptr))
    });
    let _guard = crate::gc_protect::WorkerUnprotectGuard::new(1);

    // Calculate slab size (product of all dims except the last)
    let slab_size: usize = if NDIM <= 1 {
        total_len
    } else {
        dims[..NDIM - 1].iter().product()
    };

    // Split into slabs and dispatch in parallel
    let slice = unsafe { crate::from_r::r_slice_mut(ptr, total_len) };

    if total_len > 0 && slab_size > 0 {
        slice
            .par_chunks_mut(slab_size)
            .enumerate()
            .for_each(|(slab_idx, slab)| {
                f(slab, slab_idx);
            });
    }

    sexp
}

/// Pre-allocate an R matrix and return it as [`RMatrix<T>`][crate::rarray::RMatrix].
///
/// This is like [`with_r_matrix`] but returns a typed wrapper instead of raw SEXP.
///
/// # Example
///
/// ```ignore
/// let matrix: RMatrix<f64> = new_r_matrix(3, 4, |col, col_idx| {
///     for (row, slot) in col.iter_mut().enumerate() {
///         *slot = (row + col_idx * 10) as f64;
///     }
/// });
/// ```
#[cfg(feature = "rayon")]
pub fn new_r_matrix<T, F>(nrow: usize, ncol: usize, f: F) -> crate::rarray::RMatrix<T>
where
    T: RNativeType + Send + Sync,
    F: Fn(&mut [T], usize) + Send + Sync,
{
    let sexp = with_r_matrix(nrow, ncol, f);
    // Safety: we just allocated this with correct type and dims
    unsafe { crate::rarray::RMatrix::from_sexp_unchecked(sexp) }
}

/// Pre-allocate an N-dimensional R array and return it as [`RArray<T, NDIM>`][crate::rarray::RArray].
///
/// This is like [`with_r_array`] but returns a typed wrapper instead of raw SEXP.
///
/// # Example
///
/// ```ignore
/// let array: RArray<f64, 3> = new_r_array([2, 3, 4], |slab, slab_idx| {
///     for (i, slot) in slab.iter_mut().enumerate() {
///         *slot = (slab_idx * 100 + i) as f64;
///     }
/// });
/// ```
#[cfg(feature = "rayon")]
pub fn new_r_array<T, const NDIM: usize, F>(
    dims: [usize; NDIM],
    f: F,
) -> crate::rarray::RArray<T, NDIM>
where
    T: RNativeType + Send + Sync,
    F: Fn(&mut [T], usize) + Send + Sync,
{
    let sexp = with_r_array(dims, f);
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
/// to R via `#[miniextendr]`. Unlike the standard `ParallelIterator`
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

// region: ColumnWriter for parallel DataFrame fill

/// Send+Sync wrapper for scatter-writing into pre-allocated Vec columns.
///
/// Used by the `DataFrameRow` derive macro. Not intended for direct use.
///
/// # Safety contract
///
/// - Backing Vec must have `set_len(n)` called (elements uninitialized).
/// - Every index in `0..n` must be written exactly once before the Vec is read.
/// - No two threads may write to the same index.
#[doc(hidden)]
pub struct ColumnWriter<T> {
    ptr: *mut T,
    len: usize,
}

unsafe impl<T: Send> Send for ColumnWriter<T> {}
unsafe impl<T: Send> Sync for ColumnWriter<T> {}

impl<T> ColumnWriter<T> {
    /// Create a new writer for a Vec that has had `set_len(n)` called.
    ///
    /// # Safety
    /// Vec must have `len` already set. Caller must write all indices before reading.
    #[inline]
    pub unsafe fn new(vec: &mut Vec<T>) -> Self {
        Self {
            ptr: vec.as_mut_ptr(),
            len: vec.len(),
        }
    }

    /// Write a value at index. Each index must be written by exactly one thread.
    ///
    /// # Safety
    /// - `index` must be in bounds (`< len`).
    /// - No two threads may call this with the same `index`.
    #[inline]
    pub unsafe fn write(&self, index: usize, value: T) {
        debug_assert!(
            index < self.len,
            "ColumnWriter: index {index} out of bounds"
        );
        unsafe {
            self.ptr.add(index).write(value);
        }
    }
}

// endregion

// region: Standard rayon trait extensions

/// Extension trait for collecting indexed parallel iterators directly into R memory.
///
/// This is the primary bridge between Rayon's parallel computation and R's data
/// structures. It extends every `IndexedParallelIterator` with a `.collect_r()`
/// method that writes directly into pre-allocated R memory — zero intermediate
/// allocation.
///
/// Most parallel iterator chains are indexed (known length):
/// - `slice.par_iter().map(...)` — indexed
/// - `(0..n).into_par_iter().map(...)` — indexed
/// - `vec.into_par_iter().map(...)` — indexed
/// - `.enumerate()`, `.zip()`, `.take()`, `.skip()` — indexed
///
/// For non-indexed iterators (`.filter()`, `.flat_map()`), use
/// [`par_collect_sexp()`] which collects via an intermediate `Vec<T>`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rayon_bridge::{rayon::prelude::*, ParCollectR};
///
/// #[miniextendr]
/// fn parallel_sqrt(x: &[f64]) -> SEXP {
///     // Zero-copy: writes sqrt values directly into R memory
///     x.par_iter().map(|&v| v.sqrt()).collect_r()
/// }
/// ```
///
/// # Thread Safety
///
/// `.collect_r()` must be called from the worker thread or main thread, NOT
/// from inside a Rayon parallel context. The R vector allocation uses
/// `with_r_thread()`, which only works from those threads.
#[cfg(feature = "rayon")]
pub trait ParCollectR: rayon::iter::IndexedParallelIterator + Sized {
    /// Collect this indexed parallel iterator directly into an R SEXP vector.
    ///
    /// Allocates the R vector on the main thread, then fills it in parallel
    /// using Rayon's `zip` (each output slot is paired with an iterator item).
    /// Zero intermediate allocation.
    fn collect_r(self) -> SEXP
    where
        Self::Item: RNativeType + Send + Sync,
    {
        let len = self.len();

        // Allocate, protect, and get data pointer on the R main thread
        use crate::worker::Sendable;
        let (sexp, Sendable(ptr)) = with_r_thread(move || unsafe {
            let sexp = crate::ffi::Rf_allocVector_unchecked(
                Self::Item::SEXP_TYPE,
                len as crate::ffi::R_xlen_t,
            );
            crate::ffi::Rf_protect_unchecked(sexp);
            let ptr = Self::Item::dataptr_mut(sexp);
            (sexp, Sendable(ptr))
        });
        let _guard = crate::gc_protect::WorkerUnprotectGuard::new(1);

        // Parallel zip: iterator items → output slots (zero intermediate allocation)
        if len > 0 {
            let slice = unsafe { crate::from_r::r_slice_mut(ptr, len) };
            slice.par_iter_mut().zip(self).for_each(|(slot, item)| {
                *slot = item;
            });
        }

        sexp
    }
}

// Blanket implementation: every IndexedParallelIterator gets .collect_r()
#[cfg(feature = "rayon")]
impl<T: rayon::iter::IndexedParallelIterator> ParCollectR for T {}

/// `FromParallelIterator` implementation for `Sendable<SEXP>`.
///
/// Enables standard `.collect()` syntax when the return type is `Sendable<SEXP>`:
///
/// ```ignore
/// use miniextendr_api::worker::Sendable;
///
/// #[miniextendr]
/// fn parallel_sqrt(x: &[f64]) -> Sendable<SEXP> {
///     x.par_iter().map(|&v| v.sqrt()).collect()
/// }
/// ```
///
/// The return type `Sendable<SEXP>` drives type inference so `.collect()` works
/// without a turbofish. `Sendable<SEXP>` implements `IntoR`, so it works as a
/// `#[miniextendr]` return type.
///
/// # Performance
///
/// This collects to an intermediate `Vec<T>` then converts to R. For zero-copy
/// performance on indexed iterators, use [`.collect_r()`][ParCollectR::collect_r]
/// instead.
#[cfg(feature = "rayon")]
impl<T> rayon::iter::FromParallelIterator<T> for crate::worker::Sendable<SEXP>
where
    T: Send + crate::IntoR + 'static,
    Vec<T>: crate::IntoR,
{
    fn from_par_iter<I>(pi: I) -> Self
    where
        I: rayon::iter::IntoParallelIterator<Item = T>,
    {
        let vec: Vec<T> = pi.into_par_iter().collect();
        let sexp = with_r_thread(|| vec.into_sexp());
        crate::worker::Sendable(sexp)
    }
}

/// Collect a non-indexed parallel iterator into an R vector (SEXP).
///
/// For iterators that lose their index (`.filter()`, `.flat_map()`, `.par_bridge()`),
/// this function collects to an intermediate `Vec<T>` then converts to R.
///
/// For indexed iterators (the common case), prefer [`ParCollectR::collect_r()`]
/// which writes directly into R memory with zero intermediate allocation.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rayon_bridge;
///
/// // filter() loses index — must use par_collect_sexp
/// let sexp = rayon_bridge::par_collect_sexp(
///     data.par_iter().filter(|&&x| x > 0.0).copied()
/// );
/// ```
#[cfg(feature = "rayon")]
pub fn par_collect_sexp<T, I>(iter: I) -> SEXP
where
    T: Send + IntoR + 'static,
    Vec<T>: IntoR,
    I: rayon::iter::IntoParallelIterator<Item = T>,
{
    let vec: Vec<T> = iter.into_par_iter().collect();
    with_r_thread(|| vec.into_sexp())
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

    #[test]
    #[allow(clippy::uninit_vec)]
    fn test_column_writer_parallel() {
        let n = 10_000;
        let mut col: Vec<i32> = Vec::with_capacity(n);
        unsafe {
            col.set_len(n);
        }
        {
            let w = unsafe { ColumnWriter::new(&mut col) };
            (0..n)
                .into_par_iter()
                .for_each(|i| unsafe { w.write(i, i as i32) });
        }
        for (i, &v) in col.iter().enumerate() {
            assert_eq!(v, i as i32);
        }
    }

    #[test]
    #[allow(clippy::uninit_vec)]
    fn test_column_writer_string() {
        let n = 100;
        let mut col: Vec<String> = Vec::with_capacity(n);
        unsafe {
            col.set_len(n);
        }
        {
            let w = unsafe { ColumnWriter::new(&mut col) };
            (0..n)
                .into_par_iter()
                .for_each(|i| unsafe { w.write(i, format!("s_{i}")) });
        }
        assert_eq!(col[0], "s_0");
        assert_eq!(col[99], "s_99");
    }

    #[test]
    #[allow(clippy::uninit_vec)]
    fn test_column_writer_option() {
        let n = 100;
        let mut col: Vec<Option<f64>> = Vec::with_capacity(n);
        unsafe {
            col.set_len(n);
        }
        {
            let w = unsafe { ColumnWriter::new(&mut col) };
            (0..n).into_par_iter().for_each(|i| unsafe {
                w.write(i, if i % 2 == 0 { Some(i as f64) } else { None });
            });
        }
        assert_eq!(col[0], Some(0.0));
        assert_eq!(col[1], None);
    }

    // =========================================================================
    // ParCollectR / ParCollectRIndexed tests
    // =========================================================================

    #[test]
    fn test_par_collect_r_trait_exists() {
        // Verify that ParCollectR is available on indexed parallel iterators
        // (compile-time test: if this compiles, the blanket impl works)
        fn _assert_has_collect_r<T: ParCollectR>() {}
        fn _assert_par_iter_has_it() {
            _assert_has_collect_r::<rayon::iter::Map<rayon::slice::Iter<'_, f64>, fn(&f64) -> f64>>(
            );
        }
    }

    #[test]
    fn test_par_collect_vec_roundtrip() {
        // Verify standard rayon traits work for Vec<T> (already provided by rayon)
        let data = vec![1.0_f64, 2.0, 3.0, 4.0];
        let doubled: Vec<f64> = data.par_iter().map(|&x| x * 2.0).collect();
        assert_eq!(doubled, vec![2.0, 4.0, 6.0, 8.0]);

        // IntoParallelIterator for Vec<T>
        let data = vec![1, 2, 3, 4, 5];
        let result: Vec<i32> = data.into_par_iter().map(|x| x * x).collect();
        assert_eq!(result, vec![1, 4, 9, 16, 25]);
    }

    #[test]
    fn test_par_iter_ref_and_mut() {
        // IntoParallelRefIterator for &[T]
        let data = vec![1.0, 4.0, 9.0, 16.0];
        let sqrts: Vec<f64> = data.par_iter().map(|&x: &f64| x.sqrt()).collect();
        assert_eq!(sqrts, vec![1.0, 2.0, 3.0, 4.0]);

        // IntoParallelRefMutIterator for &mut [T]
        let mut data = vec![1.0, 2.0, 3.0, 4.0];
        data.par_iter_mut().for_each(|x| *x *= 2.0);
        assert_eq!(data, vec![2.0, 4.0, 6.0, 8.0]);
    }

    #[test]
    fn test_from_par_iter_vec() {
        // FromParallelIterator<T> for Vec<T> (rayon-provided)
        let result: Vec<f64> = (0..100)
            .into_par_iter()
            .map(|i| (i as f64).sqrt())
            .collect();
        assert_eq!(result.len(), 100);
        assert!((result[0] - 0.0).abs() < 1e-10);
        assert!((result[1] - 1.0).abs() < 1e-10);
        assert!((result[4] - 2.0).abs() < 1e-10);
    }
}
