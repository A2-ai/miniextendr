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
//!     unsafe { sys::Rf_ScalarReal(*x) }  // PANICS! with_r_thread not available
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
//!     unsafe { SEXP::scalar_real_unchecked(*x) }  // UB! No routing
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
use crate::worker::with_r_thread;
use crate::{RNativeType, SEXP, SexpExt};

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
    // Validate length fits in R_xlen_t.
    // SAFETY (lint): on a 64-bit target `usize == u64`, so `i64::MAX as usize`
    // is the exact `R_xlen_t` ceiling and cannot truncate.
    #[cfg(target_pointer_width = "64")]
    {
        #[allow(clippy::cast_possible_truncation)]
        let max = i64::MAX as usize;
        assert!(
            len <= max,
            "with_r_vec: length {len} exceeds R_xlen_t maximum"
        );
    }
    #[cfg(target_pointer_width = "32")]
    assert!(
        len <= i32::MAX as usize,
        "with_r_vec: length {} exceeds R_xlen_t maximum",
        len
    );

    // Allocate, protect, and get data pointer on the R main thread
    use crate::worker::Sendable;
    let (sexp, Sendable(ptr)) = with_r_thread(move || unsafe {
        let sexp = crate::sys::Rf_allocVector_unchecked(T::SEXP_TYPE, len as crate::R_xlen_t);
        crate::sys::Rf_protect_unchecked(sexp);
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

    // Validate total length fits in R_xlen_t.
    // SAFETY (lint): on a 64-bit target `usize == u64`, so `i64::MAX as usize`
    // is the exact `R_xlen_t` ceiling and cannot truncate.
    #[cfg(target_pointer_width = "64")]
    {
        #[allow(clippy::cast_possible_truncation)]
        let max = i64::MAX as usize;
        assert!(
            len <= max,
            "with_r_matrix: total length {len} exceeds R_xlen_t maximum"
        );
    }
    #[cfg(target_pointer_width = "32")]
    assert!(
        len <= i32::MAX as usize,
        "with_r_matrix: total length {} exceeds R_xlen_t maximum",
        len
    );

    // Allocate, protect, and get data pointer on the R main thread
    use crate::worker::Sendable;
    // SAFETY: `nrow`/`ncol <= i32::MAX` asserted above, so these narrowing
    // casts cannot truncate.
    #[allow(clippy::cast_possible_truncation)]
    let (nrow_i32, ncol_i32) = (nrow as i32, ncol as i32);
    let (sexp, Sendable(ptr)) = with_r_thread(move || unsafe {
        let sexp = crate::sys::Rf_allocMatrix_unchecked(T::SEXP_TYPE, nrow_i32, ncol_i32);
        crate::sys::Rf_protect_unchecked(sexp);
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

    // Validate total length fits in R_xlen_t.
    // SAFETY (lint): on a 64-bit target `usize == u64`, so `i64::MAX as usize`
    // is the exact `R_xlen_t` ceiling and cannot truncate.
    #[cfg(target_pointer_width = "64")]
    {
        #[allow(clippy::cast_possible_truncation)]
        let max = i64::MAX as usize;
        assert!(
            total_len <= max,
            "with_r_array: total length {total_len} exceeds R_xlen_t maximum"
        );
    }
    #[cfg(target_pointer_width = "32")]
    assert!(
        total_len <= i32::MAX as usize,
        "with_r_array: total length {} exceeds R_xlen_t maximum",
        total_len
    );

    // Allocate, set dims, protect, and get data pointer on the R main thread
    use crate::worker::Sendable;
    let (sexp, Sendable(ptr)) = with_r_thread(move || unsafe {
        let sexp = crate::sys::Rf_allocVector_unchecked(T::SEXP_TYPE, total_len as crate::R_xlen_t);
        crate::sys::Rf_protect_unchecked(sexp);

        let (dim_sexp, dim_s) = crate::into_r::alloc_r_vector_unchecked::<i32>(NDIM);
        crate::sys::Rf_protect_unchecked(dim_sexp);

        for (slot, &d) in dim_s.iter_mut().zip(dims.iter()) {
            // SAFETY: every `d <= i32::MAX` asserted above, so the narrowing
            // cast cannot truncate.
            #[allow(clippy::cast_possible_truncation)]
            let d = d as i32;
            *slot = d;
        }

        sexp.set_attr_unchecked(SEXP::dim_symbol(), dim_sexp);
        crate::sys::Rf_unprotect_unchecked(1); // unprotect dim_sexp, sexp stays protected

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

// region: with_r_dataframe — flattened parallel heterogeneous column fill

/// Type-erased "fill this row-range of this column" closure.
///
/// Called with `(offset, len)` describing a contiguous half-open row range
/// `[offset, offset + len)` of one column. The concrete closure (captured at
/// `.column::<T>()` / `.column_str()` registration time) knows the column's
/// element type and destination buffer, and writes exactly that range — no
/// other thread touches it (see the safety argument on [`RDataFrameBuilder`]).
#[cfg(feature = "rayon")]
type RangeFiller = Box<dyn Fn(usize, usize) + Send + Sync>;

/// Send+Sync wrapper for a raw column-data pointer carried across the flatten
/// boundary. The pointer addresses R-owned memory for native columns or a
/// `Vec<Option<String>>` backing buffer for character columns. Disjointness
/// (per column and per row-range within a column) is the caller's invariant —
/// see the safety argument on [`RDataFrameBuilder`].
#[cfg(feature = "rayon")]
#[derive(Clone, Copy)]
struct ColPtr(*mut ());

#[cfg(feature = "rayon")]
unsafe impl Send for ColPtr {}
#[cfg(feature = "rayon")]
unsafe impl Sync for ColPtr {}

#[cfg(feature = "rayon")]
impl ColPtr {
    /// Reinterpret the erased base pointer as `*mut T`.
    ///
    /// Taking `&self` (a method call on the whole struct) makes a capturing
    /// closure capture the `Send + Sync` `ColPtr` as a whole rather than its
    /// raw `*mut ()` field (which is neither), keeping the closure `Send + Sync`.
    #[inline]
    fn cast<T>(&self) -> *mut T {
        self.0 as *mut T
    }
}

/// How a column's R storage is materialized after the parallel fill.
#[cfg(feature = "rayon")]
enum ColumnKind {
    /// Native column: the parallel fill wrote directly into R memory; the SEXP
    /// is already complete. Holds the allocated (and currently protected) SEXP.
    Native(crate::worker::Sendable<SEXP>),
    /// Character column: the parallel fill computed `Option<String>` values into
    /// a `Vec` (no R API on rayon threads). The `CHARSXP`s are set serially on
    /// the R thread during assembly. `None` becomes `NA_character_`.
    Str(Vec<Option<String>>),
}

/// One registered column: a serial allocation step plus the parallel range
/// filler that the flattened work-list dispatches.
#[cfg(feature = "rayon")]
struct ColumnReg {
    /// Allocates the column's backing storage (R SEXP for native columns, an
    /// owned `Vec` for character columns) for `nrow` rows. Runs serially on the
    /// R/worker thread. Returns the [`ColumnKind`] (carrying the protected SEXP
    /// or the owned buffer) and the raw data pointer the range filler writes
    /// through during the parallel phase.
    #[allow(clippy::type_complexity)]
    alloc: Box<dyn FnOnce(usize) -> (ColumnKind, ColPtr) + Send>,
    /// Builds the type-erased range filler once the data pointer is known.
    make_filler: Box<dyn FnOnce(ColPtr, usize) -> RangeFiller + Send>,
}

/// Builder for assembling an R `data.frame` whose columns are filled in parallel.
///
/// This is the heterogeneous-column analogue of [`with_r_matrix`]: instead of one
/// homogeneous matrix, you declare a set of typed columns (each with its own
/// element type and fill closure) and the builder fills them all in **one flat
/// parallel pass** over `(column, row-range)` work-items.
///
/// # Two axes of parallelism, one work-stealing pass
///
/// There are two ways to parallelise a column fill:
///
/// - **Column-granular** — one task per column. Fan-out width equals the column
///   count, so a 3-column × 10M-row frame only ever uses 3 threads.
/// - **Row-slice-granular** — split *one* column into contiguous row ranges
///   (what [`with_r_vec`] does internally). Great for one long column, but on
///   its own it serialises across columns.
///
/// `RDataFrameBuilder` does **not** choose. [`build`][RDataFrameBuilder::build]
/// flattens the entire job into a single work-list of `(column_index, row-range)`
/// items — each native/character column is split into
/// `chunk_size = max(1, nrow / (current_num_threads() * 4))`-row chunks (the same
/// heuristic as [`with_r_vec`], with 4× oversubscription) — then runs **one**
/// `par_iter` over that flat list. Rayon's work-stealing balances both axes
/// automatically:
///
/// - **wide** (100 cols × short) → ~100+ items, column-dominated.
/// - **tall** (3 cols × 10M rows) → each column shatters into `~nthreads*4`
///   chunks → hundreds of items, saturated even with 3 columns.
/// - **skewed** (1 huge col + many tiny) → the huge column's chunks get stolen
///   by threads idle after finishing the tiny columns.
///
/// This also avoids the per-column barrier and repeated pool spin-up that the
/// naive "fill each column, each internally parallel" (nested `par_iter`) shape
/// would cause.
///
/// # Phases
///
/// 1. Allocate each column's backing storage **serially on the R/worker thread**
///    (native columns get a protected R vector; character columns get an owned
///    `Vec<Option<String>>`). Strict PROTECT discipline — the dangerous part.
/// 2. Fill all columns in **one flat parallel pass**. No R API calls happen
///    inside the parallel region.
/// 3. Set character `CHARSXP`s serially on the R thread (CHARSXP allocation is
///    forbidden on rayon threads), then assemble the `VECSXP`, `names`, compact
///    `row.names` (`c(NA_integer_, -nrow)`), and `class = "data.frame"`.
///
/// # Column kinds
///
/// - [`column::<T>`][RDataFrameBuilder::column] — a native-typed column
///   (`f64`/`i32`/`RLogical`/`u8`/`Rcomplex`). The fill closure receives a
///   mutable chunk and its offset, exactly like [`with_r_vec`]. The buffer is R
///   memory, filled directly with zero intermediate allocation.
/// - [`column_str`][RDataFrameBuilder::column_str] — a character (`STRSXP`)
///   column. The per-row `Option<String>` values are computed **in parallel**
///   (contributing chunks to the same flat work-list as native columns), but the
///   `CHARSXP`s are set **serially** afterward. `None` becomes `NA_character_`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rayon_bridge::RDataFrameBuilder;
///
/// let df: SEXP = RDataFrameBuilder::new(1000)
///     .column::<f64>("x", |chunk, offset| {
///         for (i, slot) in chunk.iter_mut().enumerate() {
///             *slot = ((offset + i) as f64).sqrt();
///         }
///     })
///     .column::<i32>("y", |chunk, offset| {
///         for (i, slot) in chunk.iter_mut().enumerate() {
///             *slot = (offset + i) as i32;
///         }
///     })
///     .column_str("label", |i| Some(format!("row_{i}")))
///     .build();
/// ```
///
/// # Safety argument (disjoint mutation, no aliasing)
///
/// The flat work-list never produces two items that overlap:
///
/// - Different columns address **different** backing buffers (distinct R vectors
///   / distinct `Vec`s), so cross-column items are trivially disjoint.
/// - Within a column, the row ranges are a partition of `[0, nrow)` produced by
///   chunking `nrow` into fixed-size, non-overlapping spans. Each `(offset, len)`
///   item therefore owns a unique slice of that column's buffer.
///
/// Each [`RangeFiller`] reconstitutes its slice via
/// `slice::from_raw_parts_mut(base.add(offset), len)` and writes only that span.
/// Because the spans are disjoint, no two threads ever form overlapping `&mut`
/// references — there is no aliasing UB even though the work-list shares the raw
/// base pointers ([`ColPtr`], `Send + Sync`).
///
/// # Protection
///
/// Every native column SEXP is PROTECTed from allocation through insertion into
/// the `VECSXP`; the `names` / `row.names` / class transients are likewise
/// protected across each subsequent allocation. After
/// [`build`][RDataFrameBuilder::build] returns, the resulting data.frame SEXP is
/// unprotected and becomes the caller's responsibility (return it from a
/// `#[miniextendr]` fn, or PROTECT it).
#[cfg(feature = "rayon")]
pub struct RDataFrameBuilder {
    nrow: usize,
    names: Vec<String>,
    columns: Vec<ColumnReg>,
}

#[cfg(feature = "rayon")]
impl RDataFrameBuilder {
    /// Start building a data.frame with `nrow` rows.
    pub fn new(nrow: usize) -> Self {
        // Compact row.names uses i32, so nrow must fit in i32; this also implies
        // it fits in R_xlen_t on all supported pointer widths.
        assert!(
            nrow <= i32::MAX as usize,
            "RDataFrameBuilder: nrow {} exceeds i32 maximum (compact row.names)",
            nrow
        );
        Self {
            nrow,
            names: Vec::new(),
            columns: Vec::new(),
        }
    }

    /// Add a native-typed column (`f64`/`i32`/`RLogical`/`u8`/`Rcomplex`).
    ///
    /// The fill closure `f(chunk, offset)` is dispatched in parallel over chunks
    /// of the (already-allocated) R column buffer, identical in shape to
    /// [`with_r_vec`]. Chunk boundaries are deterministic for a given `nrow` and
    /// thread count.
    pub fn column<T>(
        mut self,
        name: impl Into<String>,
        f: impl Fn(&mut [T], usize) + Send + Sync + 'static,
    ) -> Self
    where
        T: RNativeType + Send + Sync,
    {
        self.names.push(name.into());
        self.columns.push(ColumnReg {
            alloc: Box::new(|nrow| {
                // Allocate + protect the R vector serially on the R thread, then
                // hand back its data pointer for the parallel fill. The
                // protection is balanced during assembly in `build`.
                use crate::worker::Sendable;
                let (sexp, Sendable(ptr)) = with_r_thread(move || unsafe {
                    let sexp =
                        crate::sys::Rf_allocVector_unchecked(T::SEXP_TYPE, nrow as crate::R_xlen_t);
                    crate::sys::Rf_protect_unchecked(sexp);
                    let ptr = T::dataptr_mut(sexp);
                    (sexp, Sendable(ptr))
                });
                (ColumnKind::Native(Sendable(sexp)), ColPtr(ptr as *mut ()))
            }),
            make_filler: Box::new(move |base: ColPtr, nrow: usize| {
                Box::new(move |offset: usize, len: usize| {
                    debug_assert!(offset + len <= nrow);
                    // Safety: this range `[offset, offset+len)` is a disjoint
                    // partition of this column's buffer (see the safety argument
                    // on `RDataFrameBuilder`); no other thread writes it. `base`
                    // is the `Send + Sync` `ColPtr`; cast inside the closure so
                    // the closure stays `Send + Sync`.
                    let ptr = base.cast::<T>();
                    let slice = unsafe { std::slice::from_raw_parts_mut(ptr.add(offset), len) };
                    f(slice, offset);
                })
            }),
        });
        self
    }

    /// Add a character (`STRSXP`) column.
    ///
    /// The fill closure `f(i)` returns the value for row `i` as `Option<String>`,
    /// where `None` maps to `NA_character_`. Values are computed in parallel
    /// (contributing chunks to the same flat work-list as native columns), then
    /// set into the R `STRSXP` serially on the R thread (CHARSXP allocation
    /// cannot happen on Rayon threads).
    pub fn column_str(
        mut self,
        name: impl Into<String>,
        f: impl Fn(usize) -> Option<String> + Send + Sync + 'static,
    ) -> Self {
        self.names.push(name.into());
        self.columns.push(ColumnReg {
            alloc: Box::new(|nrow| {
                // No R allocation here: the parallel phase fills an owned Vec.
                let mut buf: Vec<Option<String>> = (0..nrow).map(|_| None).collect();
                let ptr = buf.as_mut_ptr();
                (ColumnKind::Str(buf), ColPtr(ptr as *mut ()))
            }),
            make_filler: Box::new(move |base: ColPtr, nrow: usize| {
                Box::new(move |offset: usize, len: usize| {
                    debug_assert!(offset + len <= nrow);
                    // Safety: disjoint partition of this column's Vec buffer.
                    // Cast `base` (Send + Sync `ColPtr`) inside the closure.
                    let ptr = base.cast::<Option<String>>();
                    let slice = unsafe { std::slice::from_raw_parts_mut(ptr.add(offset), len) };
                    for (i, slot) in slice.iter_mut().enumerate() {
                        *slot = f(offset + i);
                    }
                })
            }),
        });
        self
    }

    /// Allocate, fill, and assemble the [`DataFrame`](crate::dataframe::DataFrame).
    ///
    /// Flattens every column into a single `(column_index, row-range)` work-list
    /// and runs one parallel pass over it (see the type-level docs for the
    /// scheduling argument), then assembles the `data.frame` on the R thread.
    pub fn build(self) -> crate::dataframe::DataFrame {
        // SAFETY: `build_sexp` returns a well-formed data.frame VECSXP.
        unsafe { crate::dataframe::DataFrame::from_built_sexp(self.build_sexp()) }
    }

    /// Assemble and return the raw `VECSXP` SEXP (internal; prefer [`build`](Self::build)).
    fn build_sexp(self) -> SEXP {
        let RDataFrameBuilder {
            nrow,
            names,
            columns,
        } = self;
        let ncol = columns.len();
        assert_eq!(
            names.len(),
            ncol,
            "RDataFrameBuilder: names/columns length mismatch"
        );
        // Compact row.names `c(NA, -nrow)` are emitted as INTSXP, so `nrow` must
        // fit in `i32`. Validate up front (panic → R error) rather than letting
        // `-(nrow as i32)` below silently wrap for >2^31-row frames.
        assert!(
            nrow <= i32::MAX as usize,
            "RDataFrameBuilder: nrow {nrow} exceeds i32 maximum for compact row.names"
        );

        // Phase 1: allocate every column's backing storage serially. Native
        // columns return a freshly-protected R SEXP and its data pointer;
        // character columns return an owned `Vec<Option<String>>` and its
        // pointer. We re-protect native columns *as they are allocated* (inside
        // `alloc`), so the per-column allocation in the next iteration cannot GC
        // an earlier column. These protections are balanced during assembly.
        let mut kinds: Vec<ColumnKind> = Vec::with_capacity(ncol);
        let mut fillers: Vec<RangeFiller> = Vec::with_capacity(ncol);
        for col in columns {
            let ColumnReg { alloc, make_filler } = col;
            let (kind, ptr) = alloc(nrow);
            kinds.push(kind);
            fillers.push(make_filler(ptr, nrow));
        }

        // Phase 2: flatten to ONE (column, row-range) work-list and run a single
        // parallel pass. Each item is `(column_index, offset, len)`; the column's
        // type-erased range filler writes exactly that disjoint span. Rayon's
        // work-stealing balances the column axis and the row-slice axis together.
        if nrow > 0 && ncol > 0 {
            let chunk_size = std::cmp::max(1, nrow / (rayon::current_num_threads() * 4));
            let work: Vec<(usize, usize, usize)> = (0..ncol)
                .flat_map(|col_idx| {
                    (0..nrow).step_by(chunk_size).map(move |offset| {
                        let len = std::cmp::min(chunk_size, nrow - offset);
                        (col_idx, offset, len)
                    })
                })
                .collect();

            work.par_iter().for_each(|&(col_idx, offset, len)| {
                (fillers[col_idx])(offset, len);
            });
        }
        // Fillers are no longer needed; drop them before assembly so any captured
        // closures release before we touch R.
        drop(fillers);

        // Phase 3: assemble on the R thread with strict PROTECT discipline. We
        // are inside `with_r_thread`, a known-safe context, so `_unchecked` FFI
        // is correct here (MXL301).
        //
        // PROTECT-stack invariant on entry: phase 1 left one protection per
        // *native* column (character columns hold no SEXP yet). Track that exact
        // count and balance it precisely.
        with_r_thread(move || unsafe {
            use crate::SEXPTYPE::{INTSXP, STRSXP, VECSXP};

            // Materialize character columns into protected STRSXPs now (CHARSXP
            // allocation must be serial on the R thread). Each freshly allocated
            // STRSXP is protected immediately and stays protected until rooted in
            // the parent VECSXP, exactly like the native columns.
            //
            // `native_protected` counts the protections phase 1 left on the
            // stack; we add one per character column we protect here.
            let mut native_protected = 0i32;
            let mut col_sexps: Vec<SEXP> = Vec::with_capacity(ncol);
            for kind in kinds {
                match kind {
                    ColumnKind::Native(crate::worker::Sendable(sexp)) => {
                        native_protected += 1;
                        col_sexps.push(sexp);
                    }
                    ColumnKind::Str(values) => {
                        let sexp =
                            crate::sys::Rf_allocVector_unchecked(STRSXP, nrow as crate::R_xlen_t);
                        crate::sys::Rf_protect_unchecked(sexp);
                        native_protected += 1;
                        for (i, v) in values.iter().enumerate() {
                            match v {
                                Some(s) => {
                                    sexp.set_string_elt_unchecked(i as isize, SEXP::charsxp(s))
                                }
                                None => {
                                    sexp.set_string_elt_unchecked(i as isize, SEXP::na_string())
                                }
                            }
                        }
                        col_sexps.push(sexp);
                    }
                }
            }
            // SAFETY: `native_protected` is a non-negative running count, so the
            // sign cast to `usize` cannot lose data.
            #[allow(clippy::cast_sign_loss)]
            let native_protected_usize = native_protected as usize;
            debug_assert_eq!(native_protected_usize, ncol);

            // Allocate the parent list and protect it.
            let df = crate::sys::Rf_allocVector_unchecked(VECSXP, ncol as crate::R_xlen_t);
            crate::sys::Rf_protect_unchecked(df);

            // Root every column in the parent (SET_VECTOR_ELT does not allocate).
            for (i, col) in col_sexps.into_iter().enumerate() {
                df.set_vector_elt_unchecked(i as isize, col);
            }

            // The columns are now reachable from `df`, so their individual
            // protections are no longer needed. Drop all `ncol + 1` protections
            // (the columns and `df`) and immediately re-protect `df` — no
            // allocation happens between the two calls, so `df` cannot be
            // collected in the gap.
            crate::sys::Rf_unprotect_unchecked(native_protected + 1);
            crate::sys::Rf_protect_unchecked(df);

            // names: STRSXP of column names. Protect across CHARSXP allocations.
            let names_sexp = crate::sys::Rf_allocVector_unchecked(STRSXP, ncol as crate::R_xlen_t);
            crate::sys::Rf_protect_unchecked(names_sexp);
            for (i, name) in names.iter().enumerate() {
                let charsxp = SEXP::charsxp(name);
                names_sexp.set_string_elt_unchecked(i as isize, charsxp);
            }
            df.set_names(names_sexp);
            crate::sys::Rf_unprotect_unchecked(1); // names_sexp now reachable via df

            // Compact row.names: c(NA_integer_, -nrow).
            let row_names = crate::sys::Rf_allocVector_unchecked(INTSXP, 2);
            crate::sys::Rf_protect_unchecked(row_names);
            row_names.set_integer_elt(0, i32::MIN); // NA_integer_
            // SAFETY: `nrow <= i32::MAX` asserted in `build_sexp`, so the
            // narrowing cast cannot truncate the compact row.names count.
            #[allow(clippy::cast_possible_truncation)]
            let neg_nrow = -(nrow as i32);
            row_names.set_integer_elt(1, neg_nrow);
            df.set_row_names(row_names);
            crate::sys::Rf_unprotect_unchecked(1); // row_names now reachable via df

            // class = "data.frame" (cached STRSXP — no fresh allocation).
            df.set_class(crate::cached_class::data_frame_class_sexp());

            // Balance the remaining `df` protection. No allocation follows, so
            // `df` survives until the caller takes ownership.
            crate::sys::Rf_unprotect_unchecked(1);
            df
        })
    }
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
    ///
    /// The count is returned to R as an `int`; collections larger than
    /// `i32::MAX` saturate rather than silently wrapping.
    fn par_count(&self) -> i32 {
        i32::try_from(self.par_iter().count()).unwrap_or(i32::MAX)
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
        i32::try_from(self.par_iter().filter(|&x| x.into() > threshold).count()).unwrap_or(i32::MAX)
    }

    /// Counts elements less than threshold.
    fn par_count_lt(&self, threshold: f64) -> i32
    where
        Self::Item: Into<f64>,
    {
        i32::try_from(self.par_iter().filter(|&x| x.into() < threshold).count()).unwrap_or(i32::MAX)
    }

    /// Counts elements equal to value (within epsilon for floats).
    fn par_count_eq(&self, value: f64, epsilon: f64) -> i32
    where
        Self::Item: Into<f64>,
    {
        i32::try_from(
            self.par_iter()
                .filter(|&x| (x.into() - value).abs() <= epsilon)
                .count(),
        )
        .unwrap_or(i32::MAX)
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
/// - Backing Vec must have `len == n` (elements initialized — use `vec![default; n]`).
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
    /// Create a new writer for a pre-initialized Vec (use `vec![default; n]`).
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
            let sexp =
                crate::sys::Rf_allocVector_unchecked(Self::Item::SEXP_TYPE, len as crate::R_xlen_t);
            crate::sys::Rf_protect_unchecked(sexp);
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
    fn test_column_writer_parallel() {
        let n = 10_000;
        let mut col: Vec<i32> = vec![0; n];
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
    fn test_column_writer_string() {
        let n = 100;
        let mut col: Vec<String> = vec![String::new(); n];
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
    fn test_column_writer_option() {
        let n = 100;
        let mut col: Vec<Option<f64>> = vec![None; n];
        {
            let w = unsafe { ColumnWriter::new(&mut col) };
            (0..n).into_par_iter().for_each(|i| unsafe {
                w.write(i, if i % 2 == 0 { Some(i as f64) } else { None });
            });
        }
        assert_eq!(col[0], Some(0.0));
        assert_eq!(col[1], None);
    }

    // region: ParCollectR / ParCollectRIndexed tests

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
    // endregion
}
