# Iterator and stream ALTREP adaptors whose source ends before the declared
# length fill the missing elements with R's NA (NA_real_, NA_complex_), not
# an ordinary NaN. Each fixture declares 4 elements; its source yields 2.

is_na_not_nan <- function(x) is.na(x) & !is.nan(x)

double_adaptors <- c("iter", "iter_coerce", "windowed", "sparse", "stream")
complex_adaptors <- c("iter_complex", "sparse_complex")

test_that("double adaptors read elements past a short source as NA_real_", {
  for (adaptor in double_adaptors) {
    x <- altrep_short_source(adaptor)
    expect_identical(length(x), 4L, label = adaptor)
    expect_identical(x[1:2], c(1, 2), label = adaptor)
    expect_identical(is_na_not_nan(x[3:4]), c(TRUE, TRUE), label = adaptor)
  }
})

test_that("double adaptors fill a region past a short source with NA_real_", {
  # sum() reads through Get_region; with na.rm it must not see garbage
  for (adaptor in double_adaptors) {
    x <- altrep_short_source(adaptor)
    expect_identical(sum(x, na.rm = TRUE), 3, label = adaptor)
    expect_true(is.na(sum(x)), label = adaptor)
  }
})

test_that("complex adaptors read elements past a short source as NA_complex_", {
  for (adaptor in complex_adaptors) {
    z <- altrep_short_source(adaptor)
    expect_identical(z[1:2], c(1 + 1i, 2 + 2i), label = adaptor)
    expect_identical(is_na_not_nan(z[3:4]), c(TRUE, TRUE), label = adaptor)
    expect_identical(Re(z[3:4]), c(NA_real_, NA_real_), label = adaptor)
    expect_identical(Im(z[3:4]), c(NA_real_, NA_real_), label = adaptor)
  }
})

test_that("missing doubles and complexes stay NA through saveRDS/readRDS", {
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  for (adaptor in c(double_adaptors, complex_adaptors)) {
    saveRDS(altrep_short_source(adaptor), tmp)
    restored <- readRDS(tmp)
    expect_identical(length(restored), 4L, label = adaptor)
    expect_identical(is_na_not_nan(restored), c(FALSE, FALSE, TRUE, TRUE), label = adaptor)
  }
})
