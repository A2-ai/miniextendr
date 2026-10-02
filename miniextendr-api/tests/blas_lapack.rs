//! Integration tests for the `R_ext/BLAS.h` / `R_ext/Lapack.h` declarations
//! in `miniextendr_api::sys` (`dgemm_`, `dgesv_`) and the safe adapters in
//! `miniextendr_api::linalg`, against R's BLAS/LAPACK in an embedded R.
//!
//! Products are checked three ways: against products worked out by hand,
//! against a naive Rust triple loop (integer-valued inputs, so every
//! summation order gives the exact same doubles), and against R's own `%*%`
//! and `solve()` on matrices built in R and read back. `solve()` calls the
//! same DGESV on copies of the same inputs, so its results must be
//! bit-identical. The raw `dgemm_` calls cover all four transpose flag
//! combinations on non-square shapes, which is where the hidden CHARACTER
//! length arguments (`FC_LEN_T`) sit, and a wrong leading dimension would
//! show. No case passes an invalid argument: R's `xerbla` would raise an R
//! error (a longjmp); that path is exercised through `.Call` in the rpkg
//! testthat suite.

#![cfg(feature = "blas-lapack")]

mod r_test_utils;

use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::linalg::{LinalgError, matrix_product, solve};
use miniextendr_api::r_str;
use miniextendr_api::sys::{dgemm_, dgesv_};
use std::os::raw::{c_char, c_int};

#[test]
fn blas_lapack_suite() {
    r_test_utils::with_r_thread(|| {
        define_inputs();
        dgemm_all_transpose_flags();
        dgemm_alpha_beta();
        matrix_product_hand_example();
        matrix_product_matches_r();
        matrix_product_vector_shapes();
        matrix_product_zero_sizes();
        matrix_product_rejects_bad_input();
        dgesv_raw_reports_pivots_and_zero_pivot();
        solve_hand_example();
        solve_matches_r_with_pivoting();
        solve_singular();
        solve_zero_sizes();
        solve_rejects_bad_input();
    });
}

// region: helpers

/// Evaluate R code that returns a double vector and copy it out.
fn r_reals(code: &str) -> Vec<f64> {
    let sexp = r_str!(code).unwrap_or_else(|e| panic!("R failed on `{code}`: {e}"));
    Vec::<f64>::try_from_sexp(sexp).unwrap_or_else(|e| panic!("`{code}` is not double: {e:?}"))
}

/// Element `(i, j)` of `op(X)`, where `X` is stored column-major with
/// leading dimension `ld` and `trans` says whether `op` transposes.
fn op_elt(x: &[f64], ld: usize, trans: bool, i: usize, j: usize) -> f64 {
    if trans { x[j + i * ld] } else { x[i + j * ld] }
}

/// `alpha * op(A) * op(B) + beta * C` by the textbook triple loop.
#[allow(clippy::too_many_arguments)]
fn naive_gemm(
    (ta, tb): (bool, bool),
    (m, n, k): (usize, usize, usize),
    alpha: f64,
    a: &[f64],
    lda: usize,
    b: &[f64],
    ldb: usize,
    beta: f64,
    c: &[f64],
) -> Vec<f64> {
    let mut out = vec![0.0; m * n];
    for j in 0..n {
        for i in 0..m {
            let dot: f64 = (0..k)
                .map(|l| op_elt(a, lda, ta, i, l) * op_elt(b, ldb, tb, l, j))
                .sum();
            out[i + j * m] = alpha * dot + beta * c[i + j * m];
        }
    }
    out
}

/// Transpose of the column-major `rows x cols` matrix `x`.
fn transpose(x: &[f64], rows: usize, cols: usize) -> Vec<f64> {
    let mut t = vec![0.0; x.len()];
    for j in 0..cols {
        for i in 0..rows {
            t[j + i * cols] = x[i + j * rows];
        }
    }
    t
}

fn to_c_int(x: usize) -> c_int {
    c_int::try_from(x).expect("test dimension fits c_int")
}

/// Raw `dgemm_` with ASCII flags and `ld = rows of the stored matrix`.
#[allow(clippy::too_many_arguments)]
fn raw_gemm(
    (ta, tb): (u8, u8),
    (m, n, k): (usize, usize, usize),
    alpha: f64,
    a: &[f64],
    lda: usize,
    b: &[f64],
    ldb: usize,
    beta: f64,
    c: &mut [f64],
) {
    let (transa, transb) = (c_char::from_ne_bytes([ta]), c_char::from_ne_bytes([tb]));
    let (m, n, k) = (to_c_int(m), to_c_int(n), to_c_int(k));
    let (lda, ldb, ldc) = (to_c_int(lda), to_c_int(ldb), m);
    unsafe {
        dgemm_(
            &transa,
            &transb,
            &m,
            &n,
            &k,
            &alpha,
            a.as_ptr(),
            &lda,
            b.as_ptr(),
            &ldb,
            &beta,
            c.as_mut_ptr(),
            &ldc,
            1,
            1,
        );
    }
}

/// Build the R-side inputs. Integer-valued entries keep every product
/// exact in any summation order.
///
/// `.mx_A` (5 x 3), `.mx_B` (3 x 4): integer-valued, non-square.
/// `.mx_X` (6 x 4), `.mx_Y` (4 x 7): real-valued (`rnorm`).
/// `.mx_S` (4 x 4, `[1, 1]` is zero so DGESV must pivot), `.mx_R` (4 x 3).
fn define_inputs() {
    r_str!(
        "set.seed(20261002); \
         .mx_A <- matrix(as.double(sample(-9:9, 15, TRUE)), 5, 3); \
         .mx_B <- matrix(as.double(sample(-9:9, 12, TRUE)), 3, 4); \
         .mx_X <- matrix(rnorm(24), 6, 4); \
         .mx_Y <- matrix(rnorm(28), 4, 7); \
         .mx_S <- matrix(c(0, 2, -1, 3,  1, 1, 4, -2,  2, -3, 1, 1,  5, 1, 0, 2), 4, 4); \
         .mx_R <- matrix(rnorm(12), 4, 3)"
    )
    .expect("define inputs");
}

// endregion

// region: raw dgemm_

fn dgemm_all_transpose_flags() {
    let a = r_reals(".mx_A"); // 5 x 3
    let b = r_reals(".mx_B"); // 3 x 4
    let want = r_reals(".mx_A %*% .mx_B"); // 5 x 4
    let (m, n, k) = (5, 4, 3);
    let at = transpose(&a, 5, 3); // stored 3 x 5
    let bt = transpose(&b, 3, 4); // stored 4 x 3

    for (flags, sa, lda, sb, ldb) in [
        ((b'N', b'N'), &a, 5, &b, 3),
        ((b'T', b'N'), &at, 3, &b, 3),
        ((b'N', b'T'), &a, 5, &bt, 4),
        ((b'T', b'T'), &at, 3, &bt, 4),
        // Lower-case flags mean the same (LSAME is case-insensitive).
        ((b't', b'n'), &at, 3, &b, 3),
    ] {
        let mut c = vec![f64::NAN; m * n]; // beta = 0: C is never read
        raw_gemm(flags, (m, n, k), 1.0, sa, lda, sb, ldb, 0.0, &mut c);
        let ta = flags.0.eq_ignore_ascii_case(&b'T');
        let tb = flags.1.eq_ignore_ascii_case(&b'T');
        let naive = naive_gemm((ta, tb), (m, n, k), 1.0, sa, lda, sb, ldb, 0.0, &c);
        let shown = (char::from(flags.0), char::from(flags.1));
        assert_eq!(c, want, "dgemm {shown:?} vs R's %*%");
        assert_eq!(c, naive, "dgemm {shown:?} vs the triple loop");
    }

    // op(A) = A' with a 3 x 5 result: crossprod(A, A2) style, k = 5.
    let a2 = r_reals(".mx_A[, c(3, 1)]"); // 5 x 2
    let want = r_reals("crossprod(.mx_A, .mx_A[, c(3, 1)])"); // 3 x 2
    let mut c = vec![0.0; 6];
    raw_gemm((b'T', b'N'), (3, 2, 5), 1.0, &a, 5, &a2, 5, 0.0, &mut c);
    assert_eq!(c, want, "dgemm T,N with k = 5");
}

fn dgemm_alpha_beta() {
    // C := 2 * A B - C on integer values, exact.
    let a = r_reals(".mx_A");
    let b = r_reals(".mx_B");
    let c0: Vec<f64> = (0..20).map(|i| f64::from(i) - 7.0).collect();
    let mut c = c0.clone();
    raw_gemm((b'N', b'N'), (5, 4, 3), 2.0, &a, 5, &b, 3, -1.0, &mut c);
    let naive = naive_gemm((false, false), (5, 4, 3), 2.0, &a, 5, &b, 3, -1.0, &c0);
    assert_eq!(c, naive);
}

// endregion

// region: matrix_product

fn matrix_product_hand_example() {
    // [1 3 5]   [1 4]   [22 49]
    // [2 4 6] * [2 5] = [28 64]
    //           [3 6]
    let a = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let b = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let c = matrix_product(2, 2, 3, &a, &b).unwrap();
    assert_eq!(c, [22.0, 28.0, 49.0, 64.0]);

    // Non-square 2 x 3 times 3 x 4.
    let b = [
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
    ];
    let c = matrix_product(2, 4, 3, &a, &b).unwrap();
    assert_eq!(c, [22.0, 28.0, 49.0, 64.0, 76.0, 100.0, 103.0, 136.0]);
}

fn matrix_product_matches_r() {
    let a = r_reals(".mx_A");
    let b = r_reals(".mx_B");
    let (a_before, b_before) = (a.clone(), b.clone());
    let c = matrix_product(5, 4, 3, &a, &b).unwrap();
    assert_eq!(c, r_reals(".mx_A %*% .mx_B"));
    assert_eq!((a, b), (a_before, b_before), "inputs unchanged");

    // Real-valued: R's %*% calls the same DGEMM here (no NaN, both
    // dimensions > 1), but allow for reassociation in an optimised BLAS.
    let x = r_reals(".mx_X");
    let y = r_reals(".mx_Y");
    let got = matrix_product(6, 7, 4, &x, &y).unwrap();
    let want = r_reals(".mx_X %*% .mx_Y");
    assert_eq!(got.len(), want.len());
    for (i, (g, w)) in got.iter().zip(&want).enumerate() {
        assert!(
            (g - w).abs() <= 1e-13 * w.abs().max(1.0),
            "element {i}: {g} vs R's {w}"
        );
    }
}

fn matrix_product_vector_shapes() {
    let a = r_reals(".mx_A");
    // Matrix-vector (n = 1) and vector-matrix (m = 1).
    let v = r_reals(".mx_B[, 2]");
    assert_eq!(
        matrix_product(5, 1, 3, &a, &v).unwrap(),
        r_reals(".mx_A %*% .mx_B[, 2]")
    );
    let row = r_reals(".mx_A[2, ]");
    assert_eq!(
        matrix_product(1, 4, 3, &row, &r_reals(".mx_B")).unwrap(),
        r_reals(".mx_A[2, , drop = FALSE] %*% .mx_B")
    );
    // Inner product (1 x 1) and outer product (k = 1).
    assert_eq!(
        matrix_product(1, 1, 3, &[1.0, 2.0, 3.0], &[4.0, -5.0, 6.0]).unwrap(),
        [12.0]
    );
    assert_eq!(
        matrix_product(2, 3, 1, &[1.0, 2.0], &[3.0, 4.0, 5.0]).unwrap(),
        [3.0, 6.0, 4.0, 8.0, 5.0, 10.0]
    );
}

fn matrix_product_zero_sizes() {
    assert_eq!(
        matrix_product(0, 4, 3, &[], &[0.0; 12]).unwrap(),
        Vec::<f64>::new()
    );
    assert_eq!(
        matrix_product(2, 0, 3, &[0.0; 6], &[]).unwrap(),
        Vec::<f64>::new()
    );
    assert_eq!(
        matrix_product(0, 0, 0, &[], &[]).unwrap(),
        Vec::<f64>::new()
    );
    // k = 0: an m x n matrix of zeros, as R's %*% gives.
    assert_eq!(matrix_product(2, 3, 0, &[], &[]).unwrap(), [0.0; 6]);
    assert_eq!(
        matrix_product(2, 3, 0, &[], &[]).unwrap(),
        r_reals("matrix(0, 2, 0) %*% matrix(0, 0, 3)")
    );
}

fn matrix_product_rejects_bad_input() {
    assert_eq!(
        matrix_product(2, 2, 3, &[0.0; 5], &[0.0; 6]),
        Err(LinalgError::LengthMismatch {
            what: "a",
            expected: 6,
            actual: 5
        })
    );
    assert_eq!(
        matrix_product(2, 2, 3, &[0.0; 6], &[0.0; 7]),
        Err(LinalgError::LengthMismatch {
            what: "b",
            expected: 6,
            actual: 7
        })
    );
    let too_big = usize::try_from(c_int::MAX).unwrap() + 1;
    assert_eq!(
        matrix_product(too_big, 0, 0, &[], &[]),
        Err(LinalgError::DimensionTooLarge {
            what: "m",
            value: too_big
        })
    );
    assert_eq!(
        matrix_product(0, 1, too_big, &[], &[]),
        Err(LinalgError::DimensionTooLarge {
            what: "k",
            value: too_big
        })
    );
}

// endregion

// region: dgesv_ and solve

fn dgesv_raw_reports_pivots_and_zero_pivot() {
    // [0 1] x = [2]: a zero leading entry, so row 2 is the first pivot.
    // [1 0]     [3]
    let mut a = [0.0, 1.0, 1.0, 0.0];
    let mut b = [2.0, 3.0];
    let mut ipiv: [c_int; 2] = [0; 2];
    let (n, nrhs) = (2, 1);
    let mut info: c_int = -99;
    unsafe {
        dgesv_(
            &n,
            &nrhs,
            a.as_mut_ptr(),
            &n,
            ipiv.as_mut_ptr(),
            b.as_mut_ptr(),
            &n,
            &mut info,
        );
    }
    assert_eq!(info, 0);
    assert_eq!(ipiv, [2, 2]);
    assert_eq!(b, [3.0, 2.0]);

    // Second column zero: U(2, 2) is exactly zero, INFO = 2.
    let mut a = [1.0, 2.0, 0.0, 0.0];
    let mut b = [1.0, 1.0];
    info = -99;
    unsafe {
        dgesv_(
            &n,
            &nrhs,
            a.as_mut_ptr(),
            &n,
            ipiv.as_mut_ptr(),
            b.as_mut_ptr(),
            &n,
            &mut info,
        );
    }
    assert_eq!(info, 2);
}

fn solve_hand_example() {
    // [2 1] X = [3 1]  =>  X = [1 0.2]
    // [1 3]     [4 2]           [1 0.6]
    // The first column is exact; 0.2 and 0.6 are not binary fractions.
    let a = [2.0, 1.0, 1.0, 3.0];
    let b = [3.0, 4.0, 1.0, 2.0];
    let x = solve(2, 2, &a, &b).unwrap();
    assert_eq!(&x[..2], [1.0, 1.0]);
    assert!(
        (x[2] - 0.2).abs() < 1e-15 && (x[3] - 0.6).abs() < 1e-15,
        "{x:?}"
    );
}

fn solve_matches_r_with_pivoting() {
    let s = r_reals(".mx_S");
    let r = r_reals(".mx_R");
    let (s_before, r_before) = (s.clone(), r.clone());
    // Multiple right-hand sides; .mx_S[1, 1] == 0 forces a row interchange.
    let x = solve(4, 3, &s, &r).unwrap();
    assert_eq!(
        x,
        r_reals("solve(.mx_S, .mx_R)"),
        "bit-identical to solve()"
    );
    assert_eq!((s, r), (s_before, r_before), "inputs unchanged");

    // Independent check: the residual S X - R is small.
    let sx = matrix_product(4, 3, 4, &r_reals(".mx_S"), &x).unwrap();
    for (i, (got, want)) in sx.iter().zip(r_reals(".mx_R")).enumerate() {
        assert!((got - want).abs() < 1e-12, "residual {i}: {got} vs {want}");
    }

    // A single right-hand side and the identity (B = I gives the inverse).
    let one = r_reals(".mx_R[, 2]");
    assert_eq!(
        solve(4, 1, &r_reals(".mx_S"), &one).unwrap(),
        r_reals("solve(.mx_S, .mx_R[, 2])")
    );
    assert_eq!(
        solve(4, 4, &r_reals(".mx_S"), &r_reals("diag(4)")).unwrap(),
        r_reals("solve(.mx_S)")
    );
}

fn solve_singular() {
    // A zero column: U(2, 2) is exactly zero after elimination.
    let a = [1.0, 3.0, 5.0, 0.0, 0.0, 0.0, 2.0, 4.0, 6.0];
    let b = [1.0, 2.0, 3.0];
    let (a_before, b_before) = (a, b);
    assert_eq!(solve(3, 1, &a, &b), Err(LinalgError::Singular { index: 2 }));
    assert_eq!((a, b), (a_before, b_before), "inputs unchanged");
    // R's solve() reports the same pivot.
    let msg = r_str!(
        "tryCatch(solve(matrix(c(1, 3, 5, 0, 0, 0, 2, 4, 6), 3), c(1, 2, 3)), \
         error = function(e) conditionMessage(e))"
    )
    .expect("R solve()");
    let msg = String::try_from_sexp(msg).unwrap();
    assert!(msg.contains("U[2,2] = 0"), "R said: {msg}");

    // Rows that are exact power-of-two multiples: 2 - 0.5 * 4 == 0 exactly.
    assert_eq!(
        solve(2, 2, &[1.0, 2.0, 2.0, 4.0], &[1.0, 0.0, 0.0, 1.0]),
        Err(LinalgError::Singular { index: 2 })
    );
    // The zero matrix fails at the first pivot.
    assert_eq!(
        solve(2, 1, &[0.0; 4], &[1.0, 1.0]),
        Err(LinalgError::Singular { index: 1 })
    );
    let shown = LinalgError::Singular { index: 2 }.to_string();
    assert!(shown.contains("U[2,2] = 0"), "{shown}");
}

fn solve_zero_sizes() {
    assert_eq!(solve(0, 3, &[], &[]).unwrap(), Vec::<f64>::new());
    assert_eq!(solve(0, 0, &[], &[]).unwrap(), Vec::<f64>::new());
    // nrhs = 0: nothing to solve, A is not factored (so not inspected).
    assert_eq!(solve(2, 0, &[0.0; 4], &[]).unwrap(), Vec::<f64>::new());
}

fn solve_rejects_bad_input() {
    assert_eq!(
        solve(2, 1, &[0.0; 3], &[0.0; 2]),
        Err(LinalgError::LengthMismatch {
            what: "a",
            expected: 4,
            actual: 3
        })
    );
    assert_eq!(
        solve(2, 2, &[1.0, 0.0, 0.0, 1.0], &[0.0; 2]),
        Err(LinalgError::LengthMismatch {
            what: "b",
            expected: 4,
            actual: 2
        })
    );
    let too_big = usize::try_from(c_int::MAX).unwrap() + 1;
    assert_eq!(
        solve(0, too_big, &[], &[]),
        Err(LinalgError::DimensionTooLarge {
            what: "nrhs",
            value: too_big
        })
    );
}

// endregion
