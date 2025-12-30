//! Benchmarks for `miniextendr-api/src/rarray.rs`.
//!
//! Measures common access patterns on protected REAL matrices:
//! - whole-buffer iteration via `as_slice`
//! - element access via `get_rc`
//! - column-wise access via `column`
//! - copying via `to_vec`

use miniextendr_api::rarray::RMatrix;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[inline(always)]
fn fixtures() -> miniextendr_bench::Fixtures {
    miniextendr_bench::fixtures()
}

#[divan::bench(args = [0usize, 1])]
fn rmatrix_sum_as_slice(size_idx: usize) {
    let sexp = fixtures().real_matrix(size_idx);
    let mat = unsafe { RMatrix::<f64>::from_sexp_unchecked(sexp) };
    let sum: f64 = unsafe { mat.as_slice() }.iter().copied().sum();
    divan::black_box(sum);
}

#[divan::bench(args = [0usize, 1])]
fn rmatrix_sum_get_rc(size_idx: usize) {
    let sexp = fixtures().real_matrix(size_idx);
    let mat = unsafe { RMatrix::<f64>::from_sexp_unchecked(sexp) };
    unsafe {
        let nrow = mat.nrow();
        let ncol = mat.ncol();
        let mut sum = 0.0f64;
        for col in 0..ncol {
            for row in 0..nrow {
                sum += mat.get_rc(row, col);
            }
        }
        divan::black_box(sum);
    }
}

#[divan::bench(args = [0usize, 1])]
fn rmatrix_sum_by_column_slices(size_idx: usize) {
    let sexp = fixtures().real_matrix(size_idx);
    let mat = unsafe { RMatrix::<f64>::from_sexp_unchecked(sexp) };
    unsafe {
        let ncol = mat.ncol();
        let mut sum = 0.0f64;
        for col in 0..ncol {
            sum += mat.column(col).iter().copied().sum::<f64>();
        }
        divan::black_box(sum);
    }
}

#[divan::bench(args = [0usize, 1])]
fn rmatrix_to_vec(size_idx: usize) {
    let sexp = fixtures().real_matrix(size_idx);
    let mat = unsafe { RMatrix::<f64>::from_sexp_unchecked(sexp) };
    let vec = unsafe { mat.to_vec() };
    divan::black_box(vec.len());
}
