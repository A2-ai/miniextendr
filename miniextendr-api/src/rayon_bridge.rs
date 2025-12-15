//! Rayon integration for parallel computation with R interop.
//!
//! This module provides comprehensive integration between Rayon's parallel iterators
//! and R's single-threaded API, enabling high-performance parallel Rust computation
//! while maintaining R's safety guarantees.
//!
//! # Design Philosophy
//!
//! **🚀 Rust computation: Parallel on Rayon threads (normal stacks)**
//! **🔒 R API calls: Serial on main thread (via `run_r`)**
//!
//! We use Rayon's default thread pool (2MB stacks) and route all R operations through
//! the main thread using the existing `with_r_thread` infrastructure. This means:
//!
//! - ✅ No stack size configuration needed
//! - ✅ R's stack checking stays enabled
//! - ✅ Pure Rust code runs at full parallel speed
//! - ✅ R calls are safe and properly synchronized
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │   Rayon Thread Pool (2MB stacks)        │
//! │                                         │
//! │   Thread 1   Thread 2   Thread 3        │
//! │      ↓          ↓          ↓            │
//! │   [Rust]    [Rust]    [Rust]           │ ← Parallel computation
//! │      ↓          ↓          ↓            │
//! │   run_r()   run_r()   run_r()          │ ← Need R API?
//! └──────┬──────────┬──────────┬────────────┘
//!        │          │          │
//!        └──────────┴──────────┘
//!                   ↓
//!        ┌──────────────────────┐
//!        │  Main R Thread       │
//!        │  (channel based)     │ ← Sequential R operations
//!        │                      │
//!        │  Rf_allocVector()    │
//!        │  Rf_ScalarReal()     │
//!        │  R_eval()            │
//!        └──────────┬───────────┘
//!                   ↓
//!        Results sent back to Rayon threads
//! ```
//!
//! # Quick Start Examples
//!
//! ## Pattern 1: Zero-Copy Parallel Fill (⚡ Most Efficient)
//!
//! Pre-allocate R vector, write directly from Rayon threads:
//!
//! ```ignore
//! use miniextendr_api::rayon_bridge::with_r_real_vec;
//! use rayon::prelude::*;
//!
//! #[miniextendr]
//! fn parallel_sqrt(x: &[f64]) -> SEXP {
//!     with_r_real_vec(x.len(), |output| {
//!         // Write directly into R memory (parallel, zero-copy!)
//!         output.par_iter_mut()
//!             .zip(x.par_iter())
//!             .for_each(|(out, &inp)| *out = inp.sqrt());
//!     })
//! }
//! ```
//!
//! ## Pattern 2: Builder API (Clean and Fluent)
//!
//! ```ignore
//! use miniextendr_api::rayon_bridge::RVecBuilder;
//!
//! #[miniextendr]
//! fn parallel_sequence(n: i32) -> SEXP {
//!     RVecBuilder::real(n as usize)
//!         .par_fill_with(|i| (i as f64).powi(2))
//! }
//! ```
//!
//! ## Pattern 3: Collect to Intermediate (Flexible)
//!
//! ```ignore
//! use rayon::prelude::*;
//!
//! #[miniextendr]
//! fn parallel_transform(x: &[f64]) -> SEXP {
//!     let results: RVec<f64> = x.par_iter()
//!         .map(|&v| v.log2())
//!         .collect();
//!
//!     results.into_r()  // Convert to R on main thread
//! }
//! ```
//!
//! ## Pattern 4: Parallel Reduction (Fast Aggregation)
//!
//! ```ignore
//! use miniextendr_api::rayon_bridge::reduce;
//!
//! #[miniextendr]
//! fn parallel_sum(x: &[f64]) -> SEXP {
//!     reduce::sum(x)  // Parallel sum, R scalar result
//! }
//! ```

use crate::ffi::SEXP;
use crate::worker::with_r_thread;
use crate::externalptr::SendableSexp;

#[cfg(feature = "rayon")]
pub use rayon;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

// region: Core R execution guards

/// Execute a closure that calls R APIs, running it on the main thread.
///
/// This is the fundamental primitive for all R interaction from Rayon threads.
/// The closure is sent to the main R thread and executed there, with the result
/// sent back to the Rayon thread.
///
/// # Example
///
/// ```ignore
/// use rayon::prelude::*;
/// use miniextendr_api::rayon_bridge::run_r;
///
/// // Inside a run_on_worker context:
/// let results: Vec<SEXP> = (0..100).into_par_iter()
///     .map(|i| {
///         // Rust computation (on Rayon thread)
///         let doubled = i * 2;
///
///         // R call (on main thread)
///         run_r(move || unsafe {
///             ffi::Rf_ScalarInteger(doubled)
///         })
///     })
///     .collect();
/// ```
///
/// # Panics
///
/// Panics if called outside of a `run_on_worker` context.
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

// region: IntoParallelIterator implementations for R slices

/// Parallel iterator over R integer vector data.
///
/// This iterator is created from Rust slices extracted from R vectors using
/// the FFI. The data is borrowed from R's memory, so the source SEXP must
/// remain protected during iteration.
#[cfg(feature = "rayon")]
#[derive(Debug, Clone)]
pub struct RIntSliceParIter<'data> {
    slice: &'data [i32],
}

#[cfg(feature = "rayon")]
impl<'data> ParallelIterator for RIntSliceParIter<'data> {
    type Item = &'data i32;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
    {
        self.slice.par_iter().drive_unindexed(consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.slice.len())
    }
}

#[cfg(feature = "rayon")]
impl<'data> IndexedParallelIterator for RIntSliceParIter<'data> {
    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::Consumer<Self::Item>,
    {
        self.slice.par_iter().drive(consumer)
    }

    fn len(&self) -> usize {
        self.slice.len()
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: rayon::iter::plumbing::ProducerCallback<Self::Item>,
    {
        self.slice.par_iter().with_producer(callback)
    }
}

/// Parallel iterator over R real vector data.
#[cfg(feature = "rayon")]
#[derive(Debug, Clone)]
pub struct RRealSliceParIter<'data> {
    slice: &'data [f64],
}

#[cfg(feature = "rayon")]
impl<'data> ParallelIterator for RRealSliceParIter<'data> {
    type Item = &'data f64;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
    {
        self.slice.par_iter().drive_unindexed(consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.slice.len())
    }
}

#[cfg(feature = "rayon")]
impl<'data> IndexedParallelIterator for RRealSliceParIter<'data> {
    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::Consumer<Self::Item>,
    {
        self.slice.par_iter().drive(consumer)
    }

    fn len(&self) -> usize {
        self.slice.len()
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: rayon::iter::plumbing::ProducerCallback<Self::Item>,
    {
        self.slice.par_iter().with_producer(callback)
    }
}

// endregion

// region: Generic IntoParallelIterator for standard slices

/// Extension trait to create R-aware parallel iterators from slices.
///
/// This provides a safe way to parallelize R vector data that's been
/// extracted into Rust slices.
#[cfg(feature = "rayon")]
pub trait IntoRParallelIterator {
    /// The parallel iterator type.
    type Iter: ParallelIterator;

    /// Convert into a parallel iterator optimized for R interop.
    ///
    /// This creates a parallel iterator over the slice data. The slice must
    /// remain valid (protected from R's GC) during iteration.
    fn into_r_par_iter(self) -> Self::Iter;
}

#[cfg(feature = "rayon")]
impl<'data> IntoRParallelIterator for &'data [i32] {
    type Iter = RIntSliceParIter<'data>;

    fn into_r_par_iter(self) -> Self::Iter {
        RIntSliceParIter { slice: self }
    }
}

#[cfg(feature = "rayon")]
impl<'data> IntoRParallelIterator for &'data [f64] {
    type Iter = RRealSliceParIter<'data>;

    fn into_r_par_iter(self) -> Self::Iter {
        RRealSliceParIter { slice: self }
    }
}

// endregion

// region: Parallel collection into R vectors

/// A container for parallel computation results destined for R.
///
/// This type collects results from Rayon's parallel iterators and provides
/// methods to convert them to R SEXPs on the main thread.
///
/// # Example
///
/// ```ignore
/// use rayon::prelude::*;
///
/// let computed: RVec<f64> = (0..1000)
///     .into_par_iter()
///     .map(|i| (i as f64).sqrt())
///     .collect();  // Parallel collection
///
/// let r_vec = computed.into_r();  // Convert to R (main thread)
/// ```
#[cfg(feature = "rayon")]
#[derive(Debug, Clone)]
pub struct RVec<T> {
    data: Vec<T>,
}

#[cfg(feature = "rayon")]
impl<T> RVec<T> {
    /// Create an RVec from a Rust vector.
    pub fn from_vec(data: Vec<T>) -> Self {
        Self { data }
    }

    /// Get the length.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get a slice view of the data.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Consume and get the underlying vector.
    pub fn into_inner(self) -> Vec<T> {
        self.data
    }
}

// Conversion to R for common types
#[cfg(feature = "rayon")]
impl RVec<i32> {
    /// Convert to an R integer vector on the main thread.
    ///
    /// # Safety
    ///
    /// Must be called via `run_r` or from the main thread.
    pub fn into_r(self) -> SEXP {
        run_r(move || unsafe {
            let n = self.data.len();
            let vec = crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::INTSXP, n as crate::ffi::R_xlen_t);
            let ptr = crate::ffi::INTEGER(vec);
            std::ptr::copy_nonoverlapping(self.data.as_ptr(), ptr, n);
            vec
        })
    }
}

#[cfg(feature = "rayon")]
impl RVec<f64> {
    /// Convert to an R real vector on the main thread.
    ///
    /// # Safety
    ///
    /// Must be called via `run_r` or from the main thread.
    pub fn into_r(self) -> SEXP {
        run_r(move || unsafe {
            let n = self.data.len();
            let vec = crate::ffi::Rf_allocVector(crate::ffi::SEXPTYPE::REALSXP, n as crate::ffi::R_xlen_t);
            let ptr = crate::ffi::REAL(vec);
            std::ptr::copy_nonoverlapping(self.data.as_ptr(), ptr, n);
            vec
        })
    }
}

/// Implement Rayon's `FromParallelIterator` for `RVec`.
///
/// This allows `.collect::<RVec<T>>()` to work on parallel iterators.
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

// region: Extension methods for parallel R operations

/// Extension trait for parallel operations on slices with R conversion.
#[cfg(feature = "rayon")]
pub trait ParallelSliceExt<T: Sync>: ParallelSlice<T> {
    /// Parallel map that automatically converts results to R on the main thread.
    ///
    /// The map function runs in parallel on Rayon threads (pure Rust computation).
    /// The final collection into an R vector happens on the main thread.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use miniextendr_api::rayon_bridge::ParallelSliceExt;
    ///
    /// let data: &[f64] = ...; // From R vector
    /// let r_doubled = data.par_map_to_r(|&x| x * 2.0);
    /// ```
    fn par_map_to_r<F, R>(&self, f: F) -> SEXP
    where
        F: Fn(&T) -> R + Sync + Send,
        R: Send + 'static,
        RVec<R>: IntoRSexp;

    /// Parallel filter and map with R conversion.
    ///
    /// Like `par_map_to_r` but filters elements based on a predicate first.
    fn par_filter_map_to_r<F, P, R>(&self, predicate: P, map: F) -> SEXP
    where
        P: Fn(&T) -> bool + Sync + Send,
        F: Fn(&T) -> R + Sync + Send,
        R: Send + 'static,
        RVec<R>: IntoRSexp;
}

#[cfg(feature = "rayon")]
impl<T: Sync> ParallelSliceExt<T> for [T] {
    fn par_map_to_r<F, R>(&self, f: F) -> SEXP
    where
        F: Fn(&T) -> R + Sync + Send,
        R: Send + 'static,
        RVec<R>: IntoRSexp,
    {
        let mapped: RVec<R> = self.par_iter().map(f).collect();
        mapped.into_r_sexp()
    }

    fn par_filter_map_to_r<F, P, R>(&self, predicate: P, map: F) -> SEXP
    where
        P: Fn(&T) -> bool + Sync + Send,
        F: Fn(&T) -> R + Sync + Send,
        R: Send + 'static,
        RVec<R>: IntoRSexp,
    {
        let filtered_mapped: RVec<R> = self
            .par_iter()
            .filter(|item| predicate(item))
            .map(map)
            .collect();
        filtered_mapped.into_r_sexp()
    }
}

/// Helper trait for converting RVec to R SEXP.
#[cfg(feature = "rayon")]
pub trait IntoRSexp {
    fn into_r_sexp(self) -> SEXP;
}

#[cfg(feature = "rayon")]
impl IntoRSexp for RVec<i32> {
    fn into_r_sexp(self) -> SEXP {
        self.into_r()
    }
}

#[cfg(feature = "rayon")]
impl IntoRSexp for RVec<f64> {
    fn into_r_sexp(self) -> SEXP {
        self.into_r()
    }
}

// endregion

// region: Parallel reduction with R types

/// Parallel reduction operations optimized for R.
///
/// These functions compute aggregations in parallel (Rust), then convert
/// the final result to R on the main thread.
#[cfg(feature = "rayon")]
pub mod reduce {
    use super::*;

    /// Parallel sum returning an R scalar.
    ///
    /// Computes the sum in parallel, then creates an R scalar on the main thread.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let data: &[f64] = ...; // From R vector
    /// let sum_sexp = reduce::sum(data);  // Returns R scalar
    /// ```
    pub fn sum(slice: &[f64]) -> SEXP {
        let total: f64 = slice.par_iter().sum();
        run_r(move || unsafe { crate::ffi::Rf_ScalarReal(total) })
    }

    /// Parallel sum for integers.
    pub fn sum_int(slice: &[i32]) -> SEXP {
        let total: i32 = slice.par_iter().sum();
        run_r(move || unsafe { crate::ffi::Rf_ScalarInteger(total) })
    }

    /// Parallel minimum.
    pub fn min(slice: &[f64]) -> SEXP {
        let min_val = slice.par_iter().copied().reduce(
            || f64::INFINITY,
            |a, b| a.min(b),
        );
        run_r(move || unsafe { crate::ffi::Rf_ScalarReal(min_val) })
    }

    /// Parallel maximum.
    pub fn max(slice: &[f64]) -> SEXP {
        let max_val = slice.par_iter().copied().reduce(
            || f64::NEG_INFINITY,
            |a, b| a.max(b),
        );
        run_r(move || unsafe { crate::ffi::Rf_ScalarReal(max_val) })
    }

    /// Parallel mean (average).
    pub fn mean(slice: &[f64]) -> SEXP {
        if slice.is_empty() {
            return run_r(|| unsafe { crate::ffi::R_NaString });
        }

        let (sum, count) = slice.par_iter().fold(
            || (0.0_f64, 0_usize),
            |(s, c), &x| (s + x, c + 1),
        ).reduce(
            || (0.0, 0),
            |(s1, c1), (s2, c2)| (s1 + s2, c1 + c2),
        );

        let mean_val = sum / count as f64;
        run_r(move || unsafe { crate::ffi::Rf_ScalarReal(mean_val) })
    }
}

// endregion

// region: Rayon ThreadPool configuration

#[cfg(feature = "rayon")]
use rayon::ThreadPoolBuilder;

/// Build a Rayon thread pool optimized for R interop.
///
/// Creates a pool with normal stack sizes. R calls are routed to the main thread,
/// so large stacks are not needed.
///
/// # Example
///
/// ```ignore
/// let pool = build_r_thread_pool()
///     .num_threads(4)
///     .build()
///     .unwrap();
///
/// pool.install(|| {
///     // Parallel work with run_r for R calls
/// });
/// ```
#[cfg(feature = "rayon")]
pub fn build_r_thread_pool() -> ThreadPoolBuilder {
    ThreadPoolBuilder::new()
        .thread_name(|i| format!("rayon-r-{}", i))
        // Use default stack size - we don't call R directly!
}

// endregion

// region: Convenient parallel operations

/// Parallel map with automatic R conversion.
///
/// This is a convenience function that handles the common pattern of:
/// 1. Parallel Rust computation
/// 2. Conversion to R on main thread
///
/// # Example
///
/// ```ignore
/// let data = vec![1.0, 2.0, 3.0, 4.0];
/// let r_vec = par_map_real(&data, |&x| x.sqrt());
/// ```
#[cfg(feature = "rayon")]
pub fn par_map_real<F>(slice: &[f64], f: F) -> SEXP
where
    F: Fn(&f64) -> f64 + Sync + Send,
{
    let results: Vec<f64> = slice.par_iter().map(f).collect();
    run_r(move || unsafe {
        let n = results.len();
        let vec = crate::ffi::Rf_allocVector(
            crate::ffi::SEXPTYPE::REALSXP,
            n as crate::ffi::R_xlen_t,
        );
        let ptr = crate::ffi::REAL(vec);
        std::ptr::copy_nonoverlapping(results.as_ptr(), ptr, n);
        vec
    })
}

/// Parallel map for integers.
#[cfg(feature = "rayon")]
pub fn par_map_int<F>(slice: &[i32], f: F) -> SEXP
where
    F: Fn(&i32) -> i32 + Sync + Send,
{
    let results: Vec<i32> = slice.par_iter().map(f).collect();
    run_r(move || unsafe {
        let n = results.len();
        let vec = crate::ffi::Rf_allocVector(
            crate::ffi::SEXPTYPE::INTSXP,
            n as crate::ffi::R_xlen_t,
        );
        let ptr = crate::ffi::INTEGER(vec);
        std::ptr::copy_nonoverlapping(results.as_ptr(), ptr, n);
        vec
    })
}

/// Parallel filter.
#[cfg(feature = "rayon")]
pub fn par_filter_real<P>(slice: &[f64], predicate: P) -> SEXP
where
    P: Fn(&f64) -> bool + Sync + Send,
{
    let filtered: Vec<f64> = slice.par_iter().copied().filter(|x| predicate(x)).collect();
    run_r(move || unsafe {
        let n = filtered.len();
        let vec = crate::ffi::Rf_allocVector(
            crate::ffi::SEXPTYPE::REALSXP,
            n as crate::ffi::R_xlen_t,
        );
        let ptr = crate::ffi::REAL(vec);
        std::ptr::copy_nonoverlapping(filtered.as_ptr(), ptr, n);
        vec
    })
}

// endregion

// region: Parallel chunks for complex operations

/// Process an R vector in parallel chunks.
///
/// This is often the most efficient pattern: split the data into chunks,
/// process each chunk in parallel (pure Rust), then combine and convert to R.
///
/// # Example
///
/// ```ignore
/// let data: &[f64] = ...; // From R vector
///
/// let results = par_chunks_process(data, 1000, |chunk| {
///     // Process chunk in pure Rust (parallel)
///     chunk.iter().map(|&x| x.powi(2)).sum::<f64>()
/// });
/// // results is Vec<f64> of chunk sums
/// ```
#[cfg(feature = "rayon")]
pub fn par_chunks_process<T, F, R>(slice: &[T], chunk_size: usize, f: F) -> Vec<R>
where
    T: Sync,
    F: Fn(&[T]) -> R + Sync + Send,
    R: Send,
{
    slice.par_chunks(chunk_size).map(f).collect()
}

/// Process chunks and combine into a single R vector.
#[cfg(feature = "rayon")]
pub fn par_chunks_to_r<T, F>(slice: &[T], chunk_size: usize, f: F) -> SEXP
where
    T: Sync,
    F: Fn(&[T]) -> Vec<f64> + Sync + Send,
{
    let chunk_results: Vec<Vec<f64>> = slice.par_chunks(chunk_size).map(f).collect();

    // Flatten and convert to R
    run_r(move || unsafe {
        let total_len: usize = chunk_results.iter().map(|v| v.len()).sum();
        let vec = crate::ffi::Rf_allocVector(
            crate::ffi::SEXPTYPE::REALSXP,
            total_len as crate::ffi::R_xlen_t,
        );
        let ptr = crate::ffi::REAL(vec);

        let mut offset = 0;
        for chunk in chunk_results {
            std::ptr::copy_nonoverlapping(chunk.as_ptr(), ptr.add(offset), chunk.len());
            offset += chunk.len();
        }

        vec
    })
}

// endregion

// region: Zero-copy parallel writes to pre-allocated R vectors

/// Pre-allocate an R real vector and fill it in parallel.
///
/// This is the zero-copy pattern:
/// 1. Allocates R vector on main thread
/// 2. Extracts data pointer on worker thread
/// 3. Lets Rayon threads write directly into R memory
/// 4. Returns the filled SEXP
///
/// No intermediate Vec allocation needed!
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rayon_bridge::with_r_real_vec;
/// use rayon::prelude::*;
///
/// let r_vec = with_r_real_vec(1000, |output| {
///     // output: &mut [f64] - write directly into R vector!
///     output.par_iter_mut()
///         .enumerate()
///         .for_each(|(i, slot)| {
///             *slot = (i as f64).sqrt();  // Parallel write to R memory
///         });
/// });
/// ```
#[cfg(feature = "rayon")]
pub fn with_r_real_vec<F>(len: usize, f: F) -> SEXP
where
    F: FnOnce(&mut [f64]),
{
    // Allocate R vector on main thread, return SEXP only
    let sexp = run_r(move || unsafe {
        crate::ffi::Rf_allocVector(
            crate::ffi::SEXPTYPE::REALSXP,
            len as crate::ffi::R_xlen_t,
        )
    });

    // Extract pointer on worker/Rayon thread (safe because we're just reading the pointer)
    let ptr = unsafe { crate::ffi::REAL(sexp) };
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };

    // Fill the slice (potentially in parallel)
    f(slice);

    // Return the filled SEXP
    sexp
}

/// Pre-allocate an R integer vector and fill it in parallel.
///
/// # Example
///
/// ```ignore
/// let r_vec = with_r_int_vec(1000, |output| {
///     output.par_iter_mut()
///         .enumerate()
///         .for_each(|(i, slot)| {
///             *slot = i as i32 * 2;
///         });
/// });
/// ```
#[cfg(feature = "rayon")]
pub fn with_r_int_vec<F>(len: usize, f: F) -> SEXP
where
    F: FnOnce(&mut [i32]),
{
    let sexp = run_r(move || unsafe {
        crate::ffi::Rf_allocVector(
            crate::ffi::SEXPTYPE::INTSXP,
            len as crate::ffi::R_xlen_t,
        )
    });

    let ptr = unsafe { crate::ffi::INTEGER(sexp) };
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
    f(slice);
    sexp
}

/// Pre-allocate an R logical vector and fill it in parallel.
#[cfg(feature = "rayon")]
pub fn with_r_logical_vec<F>(len: usize, f: F) -> SEXP
where
    F: FnOnce(&mut [i32]),
{
    let sexp = run_r(move || unsafe {
        crate::ffi::Rf_allocVector(
            crate::ffi::SEXPTYPE::LGLSXP,
            len as crate::ffi::R_xlen_t,
        )
    });

    let ptr = unsafe { crate::ffi::LOGICAL(sexp) };
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
    f(slice);
    sexp
}

/// Generic pre-allocation builder for different R vector types.
///
/// This provides a fluent interface for pre-allocating and filling R vectors.
///
/// # Example
///
/// ```ignore
/// let r_vec = RVecBuilder::real(1000)
///     .par_fill_with(|i| (i as f64).sqrt());
/// ```
#[cfg(feature = "rayon")]
pub struct RVecBuilder {
    sexp_type: crate::ffi::SEXPTYPE,
    len: usize,
}

#[cfg(feature = "rayon")]
impl RVecBuilder {
    /// Create a builder for an R real vector.
    pub fn real(len: usize) -> Self {
        Self {
            sexp_type: crate::ffi::SEXPTYPE::REALSXP,
            len,
        }
    }

    /// Create a builder for an R integer vector.
    pub fn integer(len: usize) -> Self {
        Self {
            sexp_type: crate::ffi::SEXPTYPE::INTSXP,
            len,
        }
    }

    /// Create a builder for an R logical vector.
    pub fn logical(len: usize) -> Self {
        Self {
            sexp_type: crate::ffi::SEXPTYPE::LGLSXP,
            len,
        }
    }

    /// Fill the vector in parallel using an index-based function.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let r_vec = RVecBuilder::real(1000)
    ///     .par_fill_with(|i| (i as f64).powi(2));
    /// ```
    pub fn par_fill_with<F>(self, f: F) -> SEXP
    where
        F: Fn(usize) -> f64 + Sync + Send,
    {
        match self.sexp_type {
            crate::ffi::SEXPTYPE::REALSXP => with_r_real_vec(self.len, move |output| {
                output
                    .par_iter_mut()
                    .enumerate()
                    .for_each(|(i, slot)| *slot = f(i));
            }),
            crate::ffi::SEXPTYPE::INTSXP => with_r_int_vec(self.len, move |output| {
                output
                    .par_iter_mut()
                    .enumerate()
                    .for_each(|(i, slot)| *slot = f(i) as i32);
            }),
            crate::ffi::SEXPTYPE::LGLSXP => with_r_logical_vec(self.len, move |output| {
                output
                    .par_iter_mut()
                    .enumerate()
                    .for_each(|(i, slot)| *slot = f(i) as i32);
            }),
            _ => panic!("Unsupported SEXP type for RVecBuilder"),
        }
    }
}

// endregion

// region: Examples and documentation

/// # Complete Examples
///
/// ## Example 1: Parallel Numeric Transformation
///
/// ```ignore
/// use miniextendr_api::prelude::*;
/// use rayon::prelude::*;
///
/// #[miniextendr]
/// fn parallel_sqrt(x: &[f64]) -> SEXP {
///     // Extract R data (main thread)
///     // Compute in parallel (Rayon threads)
///     // Convert to R (main thread via run_r)
///
///     x.par_iter()
///         .map(|&val| val.sqrt())
///         .collect::<RVec<f64>>()
///         .into_r()
/// }
/// ```
///
/// ## Example 2: Parallel Aggregation
///
/// ```ignore
/// #[miniextendr]
/// fn parallel_sum(x: &[f64]) -> SEXP {
///     use miniextendr_api::rayon_bridge::reduce;
///     reduce::sum(x)
/// }
/// ```
///
/// ## Example 3: Complex Per-Element R Operations
///
/// ```ignore
/// #[miniextendr]
/// fn parallel_with_r_calls(x: &[f64]) -> SEXP {
///     use rayon::prelude::*;
///
///     let results: Vec<SEXP> = x.par_iter()
///         .map(|&val| {
///             // Rust computation (parallel)
///             let processed = expensive_rust_function(val);
///
///             // R conversion (main thread)
///             run_r(move || unsafe {
///                 ffi::Rf_ScalarReal(processed)
///             })
///         })
///         .collect();
///
///     // Combine into R list (main thread)
///     run_r(move || build_r_list(results))
/// }
/// ```
///
/// ## Example 4: Chunked Processing for Better Performance
///
/// ```ignore
/// #[miniextendr]
/// fn parallel_chunked(x: &[f64]) -> SEXP {
///     use miniextendr_api::rayon_bridge::par_chunks_to_r;
///
///     par_chunks_to_r(x, 1000, |chunk| {
///         // Process entire chunk in parallel (no R calls!)
///         chunk.iter().map(|&x| x * x).collect()
///     })
/// }
/// ```
#[cfg(feature = "rayon")]
pub mod examples {
    //! Complete working examples of Rayon + R integration.
}

// endregion

// region: Performance monitoring

/// Performance monitoring utilities for Rayon + R operations.
#[cfg(feature = "rayon")]
pub mod perf {
    /// Get the number of threads in the global Rayon pool.
    pub fn num_threads() -> usize {
        rayon::current_num_threads()
    }

    /// Check if currently executing in a Rayon thread.
    pub fn in_rayon_thread() -> bool {
        rayon::current_thread_index().is_some()
    }

    /// Get current thread index in the Rayon pool (if any).
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
    fn test_parallel_collect_to_rvec() {
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
