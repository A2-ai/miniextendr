//! Tests for rayon parallel computation integration.

#[cfg(feature = "rayon")]
use miniextendr_api::ffi::SEXP;
#[cfg(feature = "rayon")]
use miniextendr_api::miniextendr;
#[cfg(feature = "rayon")]
use miniextendr_api::rayon_bridge::rayon::prelude::*;
#[cfg(feature = "rayon")]
use miniextendr_api::rayon_bridge::{
    par_map, par_map2, par_map3, with_r_matrix, with_r_vec, with_r_vec_map,
};

/// @noRd
/// Test parallel sum using rayon.
/// Takes a numeric vector and returns the parallel sum.
#[cfg(feature = "rayon")]
/// @noRd
#[miniextendr]
pub fn rayon_parallel_sum(x: &[f64]) -> f64 {
    x.par_iter().sum()
}

/// @noRd
/// Test parallel map: compute sqrt of each element.
#[cfg(feature = "rayon")]
/// @noRd
#[miniextendr]
pub fn rayon_parallel_sqrt(x: &[f64]) -> Vec<f64> {
    x.par_iter().map(|v| v.sqrt()).collect()
}

/// @noRd
/// Test parallel filter: keep only positive values.
#[cfg(feature = "rayon")]
/// @noRd
#[miniextendr]
pub fn rayon_parallel_filter_positive(x: &[f64]) -> Vec<f64> {
    x.par_iter().filter(|&&v| v > 0.0).copied().collect()
}

/// @noRd
/// Test Vec collection from parallel iterator.
#[cfg(feature = "rayon")]
/// @noRd
#[miniextendr]
pub fn rayon_vec_collect(n: i32) -> Vec<f64> {
    (0..n).into_par_iter().map(|i| (i as f64).sqrt()).collect()
}

/// @noRd
/// Test with_r_vec for chunk-based parallel fill.
#[cfg(feature = "rayon")]
/// @noRd
#[miniextendr]
pub fn rayon_with_r_vec(n: i32) -> SEXP {
    with_r_vec(n as usize, |chunk: &mut [f64], offset: usize| {
        for (i, slot) in chunk.iter_mut().enumerate() {
            *slot = ((offset + i) as f64).sqrt();
        }
    })
}

/// @noRd
/// Test with_r_vec_map for element-wise parallel fill.
#[cfg(feature = "rayon")]
/// @noRd
#[miniextendr]
pub fn rayon_with_r_vec_map(n: i32) -> SEXP {
    with_r_vec_map(n as usize, |i: usize| (i as f64).sqrt())
}

/// @noRd
/// Test par_map: transform input slice → R vector.
#[cfg(feature = "rayon")]
/// @noRd
#[miniextendr]
pub fn rayon_par_map(x: &[f64]) -> SEXP {
    par_map(x, |&v| v.sqrt())
}

/// @noRd
/// Test par_map2: two-input element-wise parallel map.
#[cfg(feature = "rayon")]
/// @noRd
#[miniextendr]
pub fn rayon_par_map2(a: &[f64], b: &[f64]) -> SEXP {
    par_map2(a, b, |&x, &y| x + y)
}

/// @noRd
/// Test par_map3: three-input element-wise parallel map (fused multiply-add).
#[cfg(feature = "rayon")]
/// @noRd
#[miniextendr]
pub fn rayon_par_map3(a: &[f64], b: &[f64], c: &[f64]) -> SEXP {
    par_map3(a, b, c, |&x, &y, &z| x * y + z)
}

/// @noRd
/// Test with_r_matrix for parallel column-wise fill.
#[cfg(feature = "rayon")]
/// @noRd
#[miniextendr]
pub fn rayon_with_r_matrix(nrow: i32, ncol: i32) -> SEXP {
    with_r_matrix(nrow as usize, ncol as usize, |col, col_idx| {
        for (row, slot) in col.iter_mut().enumerate() {
            *slot = (row * col_idx) as f64;
        }
    })
}

/// @noRd
/// Test parallel reduce operations.
#[cfg(feature = "rayon")]
/// @noRd
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
/// @noRd
#[miniextendr]
pub fn rayon_parallel_sum_int(x: &[i32]) -> i32 {
    x.par_iter().sum()
}

/// @noRd
/// Get number of rayon threads.
#[cfg(feature = "rayon")]
/// @noRd
#[miniextendr]
pub fn rayon_num_threads() -> i32 {
    miniextendr_api::rayon_bridge::perf::num_threads() as i32
}

/// @noRd
/// Check if currently in a rayon thread (should be false when called from R).
#[cfg(feature = "rayon")]
/// @noRd
#[miniextendr]
pub fn rayon_in_thread() -> bool {
    miniextendr_api::rayon_bridge::perf::in_rayon_thread()
}
