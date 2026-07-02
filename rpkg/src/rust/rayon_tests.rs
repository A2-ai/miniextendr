//! Tests for rayon parallel computation integration.

#[cfg(feature = "rayon")]
use miniextendr_api::dataframe::DataFrame;
#[cfg(feature = "rayon")]
use miniextendr_api::miniextendr;
#[cfg(feature = "rayon")]
use miniextendr_api::prelude::SEXP;
#[cfg(feature = "rayon")]
use miniextendr_api::rayon_bridge::rayon::prelude::*;
#[cfg(feature = "rayon")]
use miniextendr_api::rayon_bridge::{
    RDataFrameBuilder, par_map, par_map2, par_map3, with_r_matrix, with_r_vec, with_r_vec_map,
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

/// Test RDataFrameBuilder: parallel column-fill into a heterogeneous data.frame.
///
/// Builds a three-column data.frame (numeric `x`, integer `y`, character
/// `label`) of `n` rows, with each column's buffer filled in parallel via rayon.
/// The character column also exercises the `None -> NA_character_` path on every
/// fifth row.
///
/// @param n Number of rows to create.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_with_r_dataframe(n: i32) -> DataFrame {
    RDataFrameBuilder::new(n as usize)
        .column::<f64>("x", |chunk: &mut [f64], offset: usize| {
            for (i, slot) in chunk.iter_mut().enumerate() {
                *slot = ((offset + i) as f64).sqrt();
            }
        })
        .column::<i32>("y", |chunk: &mut [i32], offset: usize| {
            for (i, slot) in chunk.iter_mut().enumerate() {
                *slot = (offset + i) as i32 * 2;
            }
        })
        .column_str("label", |i: usize| {
            if i % 5 == 4 {
                None
            } else {
                Some(format!("row_{i}"))
            }
        })
        .build()
}

/// Test RDataFrameBuilder with a **wide** shape (many short columns).
///
/// Builds a data.frame of `ncol` numeric columns named `c0`, `c1`, … each of
/// `nrow` rows. Column `j` holds `row * 1000 + j` so the test can verify every
/// cell landed in the right column and row. This is the column-dominated end of
/// the flattened scheduler.
///
/// @param nrow Number of rows.
/// @param ncol Number of columns.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_dataframe_wide(nrow: i32, ncol: i32) -> DataFrame {
    let mut builder = RDataFrameBuilder::new(nrow as usize);
    for j in 0..ncol as usize {
        builder =
            builder.column::<f64>(format!("c{j}"), move |chunk: &mut [f64], offset: usize| {
                for (i, slot) in chunk.iter_mut().enumerate() {
                    *slot = ((offset + i) * 1000 + j) as f64;
                }
            });
    }
    builder.build()
}

/// Test RDataFrameBuilder with a **skewed** shape (one long numeric column plus
/// several tiny character columns).
///
/// The numeric column `big` has `nrow` rows (`big[i] = i`); two character
/// columns `t0`, `t1` also have `nrow` rows but compute a trivial value with an
/// `NA` on every third row. Under the flattened scheduler the long column's
/// chunks should be stolen by threads that finish the small columns first.
///
/// @param nrow Number of rows.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_dataframe_skewed(nrow: i32) -> DataFrame {
    RDataFrameBuilder::new(nrow as usize)
        .column::<f64>("big", |chunk: &mut [f64], offset: usize| {
            for (i, slot) in chunk.iter_mut().enumerate() {
                *slot = (offset + i) as f64;
            }
        })
        .column_str("t0", |i: usize| {
            if i % 3 == 2 {
                None
            } else {
                Some(format!("a{i}"))
            }
        })
        .column_str("t1", |i: usize| {
            if i % 3 == 2 {
                None
            } else {
                Some(format!("b{i}"))
            }
        })
        .build()
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

// region: RParallelIterator / RParallelExtend adapter traits (audit A7)

/// Materialized source for the `RParallelIterator` bridge. The contract the
/// fixture pins: data is copied out of the SEXP (here by `Vec<f64>` argument
/// conversion) *before* any parallel iteration — SEXP is not Send, so the
/// bridge never touches R memory from rayon workers.
#[cfg(feature = "rayon")]
struct ParVec {
    values: Vec<f64>,
}

#[cfg(feature = "rayon")]
impl miniextendr_api::rayon_bridge::RParallelIterator for ParVec {
    type Item = f64;

    fn par_iter(
        &self,
    ) -> impl miniextendr_api::rayon_bridge::rayon::iter::ParallelIterator<Item = f64> + '_ {
        self.values.par_iter().copied()
    }

    fn par_len(&self) -> i32 {
        self.values.len() as i32
    }
}

/// Aggregations through the `RParallelIterator` trait: returns
/// `c(par_len, par_sum, par_mean, par_min_f64, par_max_f64)`.
/// @param x Numeric vector (materialized into Rust before par-iteration).
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_trait_par_stats(x: Vec<f64>) -> Vec<f64> {
    use miniextendr_api::rayon_bridge::RParallelIterator;

    let pv = ParVec { values: x };
    vec![
        f64::from(RParallelIterator::par_len(&pv)),
        RParallelIterator::par_sum(&pv),
        RParallelIterator::par_mean(&pv),
        RParallelIterator::par_min_f64(&pv),
        RParallelIterator::par_max_f64(&pv),
    ]
}

/// Mutex-guarded sink for the `RParallelExtend` bridge (interior mutability —
/// ExternalPtr-style receivers only get `&self`).
#[cfg(feature = "rayon")]
struct ParBuf {
    data: std::sync::Mutex<Vec<f64>>,
}

#[cfg(feature = "rayon")]
impl miniextendr_api::rayon_bridge::RParallelExtend<f64> for ParBuf {
    fn par_extend(&self, items: Vec<f64>) {
        let mut guard = self.data.lock().expect("ParBuf mutex poisoned");
        guard.par_extend(items);
    }

    fn par_len(&self) -> i32 {
        self.data.lock().expect("ParBuf mutex poisoned").len() as i32
    }

    fn par_clear(&self) {
        self.data.lock().expect("ParBuf mutex poisoned").clear();
    }
}

/// Extend a collection through the `RParallelExtend` trait (`par_extend`,
/// `par_extend_from_slice`, `par_len`, `par_is_empty`, `par_clear`); returns
/// the sorted combined contents.
/// @param a Numeric vector fed through par_extend.
/// @param b Numeric vector fed through par_extend_from_slice.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_trait_par_extend(a: Vec<f64>, b: Vec<f64>) -> Vec<f64> {
    use miniextendr_api::rayon_bridge::RParallelExtend;

    let buf = ParBuf {
        data: std::sync::Mutex::new(Vec::new()),
    };
    RParallelExtend::par_extend(&buf, a);
    RParallelExtend::par_extend_from_slice(&buf, &b);
    let mut out = buf.data.into_inner().expect("ParBuf mutex poisoned");
    out.sort_by(f64::total_cmp);
    out
}

/// `par_clear` / `par_is_empty` through the trait: fill, clear, report
/// `c(len_before, len_after, is_empty_after)`.
/// @param a Numeric vector to fill with before clearing.
#[cfg(feature = "rayon")]
#[miniextendr]
pub fn rayon_trait_par_clear(a: Vec<f64>) -> Vec<i32> {
    use miniextendr_api::rayon_bridge::RParallelExtend;

    let buf = ParBuf {
        data: std::sync::Mutex::new(Vec::new()),
    };
    RParallelExtend::par_extend(&buf, a);
    let before = RParallelExtend::par_len(&buf);
    RParallelExtend::par_clear(&buf);
    let after = RParallelExtend::par_len(&buf);
    let empty = i32::from(RParallelExtend::par_is_empty(&buf));
    vec![before, after, empty]
}

// endregion
