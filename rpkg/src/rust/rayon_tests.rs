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

/// Test parallel sum of a numeric vector using rayon.
/// @param x Numeric vector to sum.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_parallel_sum(x: &[f64]) -> f64 {
    x.par_iter().sum()
}

/// Test parallel map computing the square root of each element.
/// @param x Numeric vector input.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_parallel_sqrt(x: &[f64]) -> Vec<f64> {
    x.par_iter().map(|v| v.sqrt()).collect()
}

/// Test parallel filter keeping only positive values.
/// @param x Numeric vector input.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_parallel_filter_positive(x: &[f64]) -> Vec<f64> {
    x.par_iter().filter(|&&v| v > 0.0).copied().collect()
}

/// Test collecting square roots from a parallel range iterator into a Vec.
/// @param n Number of elements to generate.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_vec_collect(n: i32) -> Vec<f64> {
    (0..n).into_par_iter().map(|i| (i as f64).sqrt()).collect()
}

/// Test with_r_vec for chunk-based parallel fill of an R numeric vector.
/// @param n Length of the output vector.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_with_r_vec(n: i32) -> SEXP {
    with_r_vec(n as usize, |chunk: &mut [f64], offset: usize| {
        for (i, slot) in chunk.iter_mut().enumerate() {
            *slot = ((offset + i) as f64).sqrt();
        }
    })
}

/// Test with_r_vec_map for element-wise parallel fill of an R numeric vector.
/// @param n Length of the output vector.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_with_r_vec_map(n: i32) -> SEXP {
    with_r_vec_map(n as usize, |i: usize| (i as f64).sqrt())
}

/// Test par_map: parallel transform of an input slice into an R numeric vector.
/// @param x Numeric vector input.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_par_map(x: &[f64]) -> SEXP {
    par_map(x, |&v| v.sqrt())
}

/// Test par_map2: element-wise parallel addition of two numeric vectors.
/// @param a Numeric vector input.
/// @param b Numeric vector input.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_par_map2(a: &[f64], b: &[f64]) -> SEXP {
    par_map2(a, b, |&x, &y| x + y)
}

/// Test par_map3: three-input element-wise parallel fused multiply-add (a*b + c).
/// @param a Numeric vector input.
/// @param b Numeric vector input.
/// @param c Numeric vector input.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_par_map3(a: &[f64], b: &[f64], c: &[f64]) -> SEXP {
    par_map3(a, b, c, |&x, &y, &z| x * y + z)
}

/// Test with_r_matrix for parallel column-wise fill of an R matrix.
/// @param nrow Number of rows.
/// @param ncol Number of columns.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_with_r_matrix(nrow: i32, ncol: i32) -> SEXP {
    with_r_matrix(nrow as usize, ncol as usize, |col, col_idx| {
        for (row, slot) in col.iter_mut().enumerate() {
            *slot = (row * col_idx) as f64;
        }
    })
}

/// Test parallel reduce computing sum, min, max, and mean of a numeric vector.
/// @param x Numeric vector input.
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

/// Test parallel sum of an integer vector using rayon.
/// @param x Integer vector to sum.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_parallel_sum_int(x: &[i32]) -> i32 {
    x.par_iter().sum()
}

/// Get the number of rayon worker threads in the global thread pool.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_num_threads() -> i32 {
    miniextendr_api::rayon_bridge::perf::num_threads() as i32
}

/// Check if the current thread is a rayon worker (should be FALSE when called from R).
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_in_thread() -> bool {
    miniextendr_api::rayon_bridge::perf::in_rayon_thread()
}
