# Tests for R's BLAS/LAPACK through miniextendr (`blas-lapack` feature): the
# raw sys::dgemm_ / sys::dgesv_ declarations and the safe linalg adapters,
# called through .Call fixtures. The package links
# $(LAPACK_LIBS) $(BLAS_LIBS) $(FLIBS), so these are the same routines R's
# %*% and solve() use.
#
# Integer-valued matrices keep every product exact in any summation order.
# solve() runs the same DGESV on copies of the same inputs, so its results
# must be identical.

set.seed(20261002)
A <- matrix(as.double(sample(-9:9, 15, TRUE)), 5, 3)
B <- matrix(as.double(sample(-9:9, 12, TRUE)), 3, 4)
S <- matrix(c(0, 2, -1, 3,  1, 1, 4, -2,  2, -3, 1, 1,  5, 1, 0, 2), 4, 4)
R <- matrix(rnorm(12), 4, 3)

mprod <- function(a, b) {
  matrix(blas_matrix_product(nrow(a), ncol(b), ncol(a), a, b), nrow(a), ncol(b))
}

gemm <- function(ta, tb, a, b, alpha = 1, beta = 0, c = NULL) {
  m <- if (toupper(ta) == "N") nrow(a) else ncol(a)
  k <- if (toupper(ta) == "N") ncol(a) else nrow(a)
  n <- if (toupper(tb) == "N") ncol(b) else nrow(b)
  if (is.null(c)) c <- matrix(0, m, n)
  out <- blas_dgemm_raw(ta, tb, m, n, k, alpha, a, nrow(a), b, nrow(b), beta, c)
  matrix(out, m, n)
}

test_that("blas_matrix_product matches hand-computed and %*% products", {
  skip_if_missing_feature("blas-lapack")
  a <- matrix(c(1, 2, 3, 4, 5, 6), 2, 3)
  b <- matrix(1:12 + 0, 3, 4)
  expect_identical(mprod(a, b), matrix(c(22, 28, 49, 64, 76, 100, 103, 136), 2, 4))
  expect_identical(mprod(A, B), A %*% B)
  # Matrix-vector, vector-matrix, inner and outer products.
  expect_identical(mprod(A, B[, 2, drop = FALSE]), A %*% B[, 2])
  expect_identical(mprod(A[2, , drop = FALSE], B), A[2, , drop = FALSE] %*% B)
  expect_identical(mprod(t(1:3 + 0), matrix(c(4, -5, 6))), matrix(12))
  expect_identical(mprod(matrix(c(1, 2)), t(c(3, 4, 5))), c(1, 2) %o% c(3, 4, 5))
})

test_that("blas_matrix_product agrees with %*% on real-valued matrices", {
  skip_if_missing_feature("blas-lapack")
  x <- matrix(rnorm(24), 6, 4)
  y <- matrix(rnorm(28), 4, 7)
  expect_equal(mprod(x, y), x %*% y, tolerance = 1e-13)
})

test_that("blas_matrix_product leaves its inputs unchanged", {
  skip_if_missing_feature("blas-lapack")
  a <- A + 0
  b <- B + 0
  mprod(a, b)
  expect_identical(a, A)
  expect_identical(b, B)
})

test_that("blas_matrix_product handles zero-size dimensions without the BLAS", {
  skip_if_missing_feature("blas-lapack")
  expect_identical(blas_matrix_product(0L, 4L, 3L, double(), double(12)), double())
  expect_identical(blas_matrix_product(2L, 0L, 3L, double(6), double()), double())
  # k = 0: an m x n matrix of zeros, as %*% gives.
  expect_identical(
    blas_matrix_product(2L, 3L, 0L, double(), double()),
    as.vector(matrix(0, 2, 0) %*% matrix(0, 0, 3))
  )
})

test_that("blas_matrix_product rejects mismatched lengths", {
  skip_if_missing_feature("blas-lapack")
  expect_error(blas_matrix_product(2L, 2L, 3L, double(5), double(6)),
               "`a` has 5 elements, its dimensions need 6")
  expect_error(blas_matrix_product(2L, 2L, 3L, double(6), double(7)),
               "`b` has 7 elements, its dimensions need 6")
})

test_that("raw dgemm_ honours all four transpose flags", {
  skip_if_missing_feature("blas-lapack")
  At <- t(A)
  Bt <- t(B)
  expect_identical(gemm("N", "N", A, B), A %*% B)
  expect_identical(gemm("T", "N", At, B), A %*% B)
  expect_identical(gemm("N", "T", A, Bt), A %*% B)
  expect_identical(gemm("T", "T", At, Bt), A %*% B)
  expect_identical(gemm("t", "n", At, B), A %*% B)
  # Non-square op(A) = A': a 3 x 2 crossproduct over k = 5.
  expect_identical(gemm("T", "N", A, A[, c(3, 1)]), crossprod(A, A[, c(3, 1)]))
  expect_identical(gemm("N", "T", B, B), tcrossprod(B))
})

test_that("raw dgemm_ applies alpha and beta", {
  skip_if_missing_feature("blas-lapack")
  c0 <- matrix(as.double(0:19) - 7, 5, 4)
  expect_identical(gemm("N", "N", A, B, alpha = 2, beta = -1, c = c0), 2 * (A %*% B) - c0)
})

test_that("an invalid dgemm_ argument raises R's xerbla error", {
  skip_if_missing_feature("blas-lapack")
  # Flag 'X' is argument 1; lda = 1 < m = 5 is argument 8.
  expect_error(
    blas_dgemm_raw("X", "N", 5L, 4L, 3L, 1, A, 5L, B, 3L, 0, double(20)),
    "DGEMM"
  )
  expect_error(
    blas_dgemm_raw("N", "N", 5L, 4L, 3L, 1, A, 1L, B, 3L, 0, double(20)),
    "DGEMM"
  )
  # The process carries on normally afterwards.
  expect_identical(gemm("N", "N", A, B), A %*% B)
})

test_that("blas_solve matches solve() with multiple right-hand sides and pivoting", {
  skip_if_missing_feature("blas-lapack")
  # S[1, 1] == 0 forces a row interchange.
  x <- matrix(blas_solve(4L, 3L, S, R), 4, 3)
  expect_identical(x, solve(S, R))
  expect_equal(S %*% x, R, tolerance = 1e-12)
  expect_identical(blas_solve(4L, 1L, S, R[, 2]), solve(S, R[, 2]))
  expect_identical(matrix(blas_solve(4L, 4L, S, diag(4)), 4, 4), solve(S))
  # A hand example with an exact solution.
  expect_identical(blas_solve(2L, 1L, matrix(c(2, 1, 1, 3), 2), c(3, 4)), c(1, 1))
})

test_that("blas_solve leaves its inputs unchanged", {
  skip_if_missing_feature("blas-lapack")
  s <- S + 0
  r <- R + 0
  blas_solve(4L, 3L, s, r)
  expect_identical(s, S)
  expect_identical(r, R)
})

test_that("blas_solve reports an exactly singular matrix like solve()", {
  skip_if_missing_feature("blas-lapack")
  sing <- matrix(c(1, 3, 5, 0, 0, 0, 2, 4, 6), 3)
  expect_error(blas_solve(3L, 1L, sing, c(1, 2, 3)), "U\\[2,2\\] = 0")
  expect_error(solve(sing, c(1, 2, 3)), "U\\[2,2\\] = 0")
  expect_error(blas_solve(2L, 2L, matrix(c(1, 2, 2, 4), 2), diag(2)), "U\\[2,2\\] = 0")
  expect_error(blas_solve(2L, 1L, matrix(0, 2, 2), c(1, 1)), "U\\[1,1\\] = 0")
  # Raw dgesv_ returns INFO = 2 for the same matrix.
  expect_identical(blas_dgesv_info(3L, 1L, sing, 3L, c(1, 2, 3), 3L), 2L)
  expect_identical(blas_dgesv_info(4L, 3L, S, 4L, R, 4L), 0L)
})

test_that("blas_solve handles zero-size systems without LAPACK", {
  skip_if_missing_feature("blas-lapack")
  expect_identical(blas_solve(0L, 3L, double(), double()), double())
  # nrhs = 0: nothing to solve, A is not factored (solve() errors instead).
  expect_identical(blas_solve(2L, 0L, matrix(0, 2, 2), double()), double())
})

test_that("blas_solve rejects mismatched lengths", {
  skip_if_missing_feature("blas-lapack")
  expect_error(blas_solve(2L, 1L, double(3), double(2)), "`a` has 3 elements")
  expect_error(blas_solve(2L, 2L, diag(2), double(2)), "`b` has 2 elements")
})

test_that("an invalid dgesv_ argument raises R's xerbla error", {
  skip_if_missing_feature("blas-lapack")
  # lda = 1 < n = 2 is argument 4.
  expect_error(blas_dgesv_info(2L, 1L, diag(2), 1L, c(1, 1), 2L), "DGESV")
})
