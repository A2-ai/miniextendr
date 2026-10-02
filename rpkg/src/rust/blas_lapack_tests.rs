//! Fixtures for R's BLAS/LAPACK (`blas-lapack` feature): the raw
//! `sys::dgemm_` / `sys::dgesv_` declarations and the safe
//! `miniextendr_api::linalg` adapters, called through `.Call`.
//!
//! The package build links `$(LAPACK_LIBS) $(BLAS_LIBS) $(FLIBS)` (configure
//! adds them when the feature is on), so these resolve against the libraries
//! R itself uses, as in a downstream package. The matrices arrive as
//! column-major `&[f64]` borrowed from R, so the testthat suite can check
//! that R's objects are left unchanged. The raw fixtures pass their arguments
//! through without validation: an invalid one reaches R's `xerbla`, whose R
//! error the `#[miniextendr]` body catches.

use miniextendr_api::linalg::{matrix_product, solve};
use miniextendr_api::miniextendr;
use miniextendr_api::sys::{dgemm_, dgesv_};
use std::os::raw::{c_char, c_int};

/// Dimensions arrive as R integers; negative ones are refused here.
fn dim(name: &str, value: i32) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| format!("`{name}` must be non-negative, got {value}"))
}

/// The first byte of a flag string, as a Fortran CHARACTER.
fn flag(s: &str) -> c_char {
    let byte = *s
        .as_bytes()
        .first()
        .expect("a transpose flag must not be empty");
    c_char::from_ne_bytes([byte])
}

/// The product `A B` through `linalg::matrix_product` (DGEMM).
/// @param m Integer: rows of `A` and of the result (of `op(A)` for `blas_dgemm_raw`).
/// @param n Integer: columns of `B` and of the result; the order of `A` for
///   `blas_solve` / `blas_dgesv_info`.
/// @param k Integer: columns of `A` and rows of `B` (of `op(A)` / `op(B)` for
///   `blas_dgemm_raw`).
/// @param a Double vector: `A`, column-major.
/// @param b Double vector: `B`, column-major.
/// @return Double vector: `A B`, column-major, length `m * n`.
#[miniextendr]
pub fn blas_matrix_product(
    m: i32,
    n: i32,
    k: i32,
    a: &[f64],
    b: &[f64],
) -> Result<Vec<f64>, String> {
    matrix_product(dim("m", m)?, dim("n", n)?, dim("k", k)?, a, b).map_err(|e| e.to_string())
}

/// The solution `X` of `A X = B` through `linalg::solve` (DGESV).
/// @param n Integer: columns of `B` and of the result; the order of `A` for
///   `blas_solve` / `blas_dgesv_info`.
/// @param nrhs Integer: columns of `B` (right-hand sides).
/// @param a Double vector: `A`, column-major.
/// @param b Double vector: `B`, column-major.
/// @return Double vector: `X`, column-major, length `n * nrhs`.
#[miniextendr]
pub fn blas_solve(n: i32, nrhs: i32, a: &[f64], b: &[f64]) -> Result<Vec<f64>, String> {
    solve(dim("n", n)?, dim("nrhs", nrhs)?, a, b).map_err(|e| e.to_string())
}

/// Raw `dgemm_`: `alpha * op(A) * op(B) + beta * C`, with every argument
/// passed through unchecked (an invalid one reaches R's `xerbla`).
/// @param transa Character scalar: `"N"` or `"T"` for `op(A)` (only the
///   first character is passed, with a hidden length of 1).
/// @param transb Character scalar: `"N"` or `"T"` for `op(B)`.
/// @param m Integer: rows of `A` and of the result (of `op(A)` for `blas_dgemm_raw`).
/// @param n Integer: columns of `B` and of the result; the order of `A` for
///   `blas_solve` / `blas_dgesv_info`.
/// @param k Integer: columns of `A` and rows of `B` (of `op(A)` / `op(B)` for
///   `blas_dgemm_raw`).
/// @param alpha Double scalar multiplying `op(A) op(B)`.
/// @param a Double vector: `A`, column-major.
/// @param lda Integer: leading dimension of `A`.
/// @param b Double vector: `B`, column-major.
/// @param ldb Integer: leading dimension of `B`.
/// @param beta Double scalar multiplying `C`.
/// @param c Double vector: `C`, `m x n` (leading dimension `m`).
/// @return Double vector: the updated `C`.
#[miniextendr]
#[allow(clippy::too_many_arguments)]
pub fn blas_dgemm_raw(
    transa: &str,
    transb: &str,
    m: i32,
    n: i32,
    k: i32,
    alpha: f64,
    a: &[f64],
    lda: i32,
    b: &[f64],
    ldb: i32,
    beta: f64,
    c: &[f64],
) -> Vec<f64> {
    let (ta, tb) = (flag(transa), flag(transb));
    let mut out = c.to_vec();
    // SAFETY: the R test passes buffers that match the dimensions, except
    // in the xerbla cases, where DGEMM checks the arguments and calls
    // xerbla (an R error, caught by the #[miniextendr] body) before reading
    // any matrix. One hidden length per flag.
    unsafe {
        dgemm_(
            &ta,
            &tb,
            &m,
            &n,
            &k,
            &alpha,
            a.as_ptr(),
            &lda,
            b.as_ptr(),
            &ldb,
            &beta,
            out.as_mut_ptr(),
            &m,
            1,
            1,
        );
    }
    out
}

/// Raw `dgesv_` with every argument passed through unchecked: returns
/// DGESV's `INFO` (an invalid argument reaches R's `xerbla` instead).
/// @param n Integer: columns of `B` and of the result; the order of `A` for
///   `blas_solve` / `blas_dgesv_info`.
/// @param nrhs Integer: columns of `B` (right-hand sides).
/// @param a Double vector: `A`, column-major.
/// @param lda Integer: leading dimension of `A`.
/// @param b Double vector: `B`, column-major.
/// @param ldb Integer: leading dimension of `B`.
/// @return Integer: `INFO` (0, or the 1-based index of a zero pivot).
#[miniextendr]
pub fn blas_dgesv_info(n: i32, nrhs: i32, a: &[f64], lda: i32, b: &[f64], ldb: i32) -> i32 {
    let mut lu = a.to_vec();
    let mut x = b.to_vec();
    let mut ipiv: Vec<c_int> = vec![0; a.len().max(1)];
    let mut info: c_int = 0;
    // SAFETY: as for blas_dgemm_raw; ipiv has at least n entries whenever
    // DGESV gets past its argument checks (a holds lda * n >= n values).
    unsafe {
        dgesv_(
            &n,
            &nrhs,
            lu.as_mut_ptr(),
            &lda,
            ipiv.as_mut_ptr(),
            x.as_mut_ptr(),
            &ldb,
            &mut info,
        );
    }
    info
}
