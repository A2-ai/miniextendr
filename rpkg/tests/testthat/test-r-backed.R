# Tests for R-backed zero-copy storage (nalgebra RDVector/RDMatrix, ndarray RndVec/RndMat)

# region: nalgebra RDVector

test_that("RDVector roundtrip preserves values", {
  v <- c(1.5, 2.5, 3.5)
  expect_equal(r_backed_rdvector_roundtrip(v), v)
})

test_that("RDVector norm computes correctly", {
  expect_equal(r_backed_rdvector_norm(c(3, 4)), 5)
})

test_that("RDVector sum computes correctly", {
  expect_equal(r_backed_rdvector_sum(c(1, 2, 3, 4)), 10)
})

test_that("RDVector len returns correct length", {
  expect_equal(r_backed_rdvector_len(c(1, 2, 3)), 3L)
})

test_that("RDVector dot product computes correctly", {
  expect_equal(r_backed_rdvector_dot(c(1, 2, 3), c(4, 5, 6)), 32)
})

test_that("RDVector in-place scaling works", {
  v <- c(1, 2, 3)
  expect_equal(r_backed_rdvector_scale(v, 2), c(2, 4, 6))
})

test_that("RDVector integer roundtrip preserves values", {
  v <- 1:5
  expect_equal(r_backed_rdvector_int_roundtrip(v), v)
})

test_that("RDVector integer sum computes correctly", {
  expect_equal(r_backed_rdvector_int_sum(1:10), 55L)
})

test_that("RDVector empty vector roundtrip works", {
  v <- numeric(0)
  expect_equal(r_backed_rdvector_empty_roundtrip(v), v)
})

# endregion

# region: nalgebra RDMatrix

test_that("RDMatrix roundtrip preserves values", {
  m <- matrix(c(1, 2, 3, 4, 5, 6), nrow = 2, ncol = 3)
  result <- r_backed_rdmatrix_roundtrip(m)
  expect_equal(result, m)
})

test_that("RDMatrix nrow/ncol correct", {
  m <- matrix(as.double(1:6), nrow = 2, ncol = 3)
  expect_equal(r_backed_rdmatrix_nrow(m), 2L)
  expect_equal(r_backed_rdmatrix_ncol(m), 3L)
})

test_that("RDMatrix sum computes correctly", {
  m <- matrix(c(1, 2, 3, 4), nrow = 2)
  expect_equal(r_backed_rdmatrix_sum(m), 10)
})

test_that("RDMatrix trace computes correctly", {
  # Column-major: matrix(c(1,2,3,4), nrow=2) = [[1,3],[2,4]]
  # trace = 1 + 4 = 5
  m <- matrix(c(1, 2, 3, 4), nrow = 2)
  expect_equal(r_backed_rdmatrix_trace(m), 5)
})

test_that("RDMatrix scaling works", {
  m <- matrix(c(1, 2, 3, 4), nrow = 2)
  expected <- m * 3
  result <- r_backed_rdmatrix_scale(m, 3)
  expect_equal(result, expected)
})

# endregion

# region: ndarray RndVec

test_that("RndVec roundtrip preserves values", {
  v <- c(1.5, 2.5, 3.5)
  expect_equal(r_backed_rndvec_roundtrip(v), v)
})

test_that("RndVec sum computes correctly", {
  expect_equal(r_backed_rndvec_sum(c(1, 2, 3, 4)), 10)
})

test_that("RndVec len returns correct length", {
  expect_equal(r_backed_rndvec_len(c(10, 20, 30)), 3L)
})

test_that("RndVec double via view_mut works", {
  v <- c(1, 2, 3)
  expect_equal(r_backed_rndvec_double(v), c(2, 4, 6))
})

test_that("RndVec integer roundtrip preserves values", {
  v <- 1:5
  expect_equal(r_backed_rndvec_int_roundtrip(v), v)
})

test_that("RndVec empty sum returns 0", {
  expect_equal(r_backed_rndvec_empty_sum(numeric(0)), 0)
})

# endregion

# region: ndarray RndMat

test_that("RndMat roundtrip preserves values", {
  m <- matrix(c(1, 2, 3, 4, 5, 6), nrow = 2, ncol = 3)
  result <- r_backed_rndmat_roundtrip(m)
  expect_equal(result, m)
})

test_that("RndMat sum computes correctly", {
  m <- matrix(c(1, 2, 3, 4), nrow = 2)
  expect_equal(r_backed_rndmat_sum(m), 10)
})

test_that("RndMat nrow/ncol correct", {
  m <- matrix(as.double(1:6), nrow = 2, ncol = 3)
  expect_equal(r_backed_rndmat_nrow(m), 2L)
  expect_equal(r_backed_rndmat_ncol(m), 3L)
})

test_that("RndMat trace via view().diag().sum()", {
  # matrix(c(1,2,3,4), nrow=2) = [[1,3],[2,4]], trace = 1+4 = 5
  m <- matrix(c(1, 2, 3, 4), nrow = 2)
  expect_equal(r_backed_rndmat_trace(m), 5)
})

test_that("RndMat fill via view_mut works", {
  m <- matrix(c(1, 2, 3, 4), nrow = 2)
  result <- r_backed_rndmat_fill(m, 42)
  expect_equal(result, matrix(42, nrow = 2, ncol = 2))
})

# endregion
