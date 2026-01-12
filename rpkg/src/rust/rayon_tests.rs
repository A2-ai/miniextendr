//! Tests for rayon parallel computation integration.

#[cfg(feature = "rayon")]
use miniextendr_api::ffi::SEXP;
#[cfg(feature = "rayon")]
use miniextendr_api::rayon_bridge::rayon::prelude::*;
#[cfg(feature = "rayon")]
use miniextendr_api::rayon_bridge::{with_r_matrix, with_r_vec};
#[cfg(feature = "rayon")]
use miniextendr_api::{miniextendr, miniextendr_module};

/// @noRd
/// Test parallel sum using rayon.
/// Takes a numeric vector and returns the parallel sum.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_parallel_sum(x: &[f64]) -> f64 {
    x.par_iter().sum()
}

/// @noRd
/// Test parallel map: compute sqrt of each element.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_parallel_sqrt(x: &[f64]) -> Vec<f64> {
    x.par_iter().map(|v| v.sqrt()).collect()
}

/// @noRd
/// Test parallel filter: keep only positive values.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_parallel_filter_positive(x: &[f64]) -> Vec<f64> {
    x.par_iter().filter(|&&v| v > 0.0).copied().collect()
}

/// @noRd
/// Test Vec collection from parallel iterator.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_vec_collect(n: i32) -> Vec<f64> {
    (0..n).into_par_iter().map(|i| (i as f64).sqrt()).collect()
}

/// @noRd
/// Test with_r_vec for zero-copy parallel fill.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_with_r_vec(n: i32) -> SEXP {
    with_r_vec(n as usize, |output: &mut [f64]| {
        output
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, slot)| *slot = (i as f64).sqrt());
    })
}

/// @noRd
/// Test with_r_matrix for parallel matrix fill.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_with_r_matrix(nrow: i32, ncol: i32) -> SEXP {
    with_r_matrix(nrow as usize, ncol as usize, |slice, nrow, _ncol| {
        slice.par_iter_mut().enumerate().for_each(|(i, slot)| {
            let row = i % nrow;
            let col = i / nrow;
            *slot = (row * col) as f64;
        });
    })
}

/// @noRd
/// Test parallel reduce operations.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_parallel_stats(x: &[f64]) -> Vec<f64> {
    let sum: f64 = x.par_iter().sum();
    let min: f64 = x
        .par_iter()
        .copied()
        .reduce(|| f64::INFINITY, |a, b| a.min(b));
    let max: f64 = x
        .par_iter()
        .copied()
        .reduce(|| f64::NEG_INFINITY, |a, b| a.max(b));
    let mean = if x.is_empty() {
        f64::NAN
    } else {
        sum / x.len() as f64
    };
    vec![sum, min, max, mean]
}

/// @noRd
/// Test parallel integer operations.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_parallel_sum_int(x: &[i32]) -> i32 {
    x.par_iter().sum()
}

/// @noRd
/// Get number of rayon threads.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_num_threads() -> i32 {
    miniextendr_api::rayon_bridge::perf::num_threads() as i32
}

/// @noRd
/// Check if currently in a rayon thread (should be false when called from R).
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_in_thread() -> bool {
    miniextendr_api::rayon_bridge::perf::in_rayon_thread()
}

#[cfg(feature = "rayon")]
miniextendr_module! {
    mod rayon_tests;

    fn rayon_parallel_sum;
    fn rayon_parallel_sqrt;
    fn rayon_parallel_filter_positive;
    fn rayon_vec_collect;
    fn rayon_with_r_vec;
    fn rayon_with_r_matrix;
    fn rayon_parallel_stats;
    fn rayon_parallel_sum_int;
    fn rayon_num_threads;
    fn rayon_in_thread;
}
