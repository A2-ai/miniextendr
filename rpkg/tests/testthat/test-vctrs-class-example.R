# Tests for the Rust-implemented vctrs "percent" class
#
# This tests the vctrs_class_example module which demonstrates implementing
# a vctrs-compatible S3 class entirely in Rust.

# Helper to skip if vctrs feature is not enabled
skip_if_vctrs_disabled <- function() {
  skip_if_not("vctrs" %in% miniextendr::rpkg_enabled_features(), "vctrs feature not enabled")
  skip_if_not_installed("vctrs")
}

# Use functions from the miniextendr namespace
new_percent <- miniextendr::new_percent

test_that("new_percent creates a vctrs vector", {
  skip_if_vctrs_disabled()

  x <- new_percent(c(0.25, 0.5, 0.75))

  expect_true(vctrs::vec_is(x))
  expect_s3_class(x, "percent")
  expect_equal(length(x), 3)
})

test_that("format.percent formats values as percentages", {
  skip_if_vctrs_disabled()

  x <- new_percent(c(0.25, 0.5, 0.75))

  expect_equal(format(x), c("25.0%", "50.0%", "75.0%"))
})

test_that("vec_ptype_abbr.percent returns '%'", {
  skip_if_vctrs_disabled()

  x <- new_percent(c(0.25))

  expect_equal(vctrs::vec_ptype_abbr(x), "%")
})

test_that("vec_c combines percent vectors", {
  skip_if_vctrs_disabled()

  x <- new_percent(c(0.25, 0.5))
  y <- new_percent(c(0.75))

  combined <- vctrs::vec_c(x, y)

  expect_s3_class(combined, "percent")
  expect_equal(length(combined), 3)
  expect_equal(as.numeric(combined), c(0.25, 0.5, 0.75))
})

test_that("subsetting preserves percent class", {
  skip_if_vctrs_disabled()

  x <- new_percent(c(0.25, 0.5, 0.75))

  subset <- x[1:2]

  expect_s3_class(subset, "percent")
  expect_equal(length(subset), 2)
})

test_that("double coerces to percent", {
  skip_if_vctrs_disabled()

  x <- new_percent(c(0.25))
  y <- 0.5

  combined <- vctrs::vec_c(x, y)

  expect_s3_class(combined, "percent")
  expect_equal(length(combined), 2)
})

test_that("percent coerces to double", {
  skip_if_vctrs_disabled()

  x <- new_percent(c(0.25))

  as_double <- vctrs::vec_cast(x, double())

  expect_type(as_double, "double")
  expect_equal(as_double, 0.25)
})

test_that("percent S3 method implementations are registered correctly", {
  skip_if_vctrs_disabled()

  x <- new_percent(c(0.25, 0.5))
  proxy <- vec_proxy.percent(x)
  expect_false(inherits(proxy, "percent"))
  expect_equal(as.numeric(proxy), as.numeric(x))

  restored <- vec_restore.percent(proxy, x)
  expect_s3_class(restored, "percent")
  expect_equal(as.numeric(restored), as.numeric(x))

  ptype <- vec_ptype2.percent.percent(x, x)
  expect_s3_class(ptype, "percent")
  expect_equal(length(ptype), 0L)

  identity_cast <- vec_cast.percent.percent(x, new_percent(numeric(0)))
  expect_identical(identity_cast, x)

  percent_cast <- vec_cast.percent.double(x, double())
  expect_s3_class(percent_cast, "percent")
  expect_equal(as.numeric(percent_cast), as.numeric(x))

  ptype2_pd <- vec_ptype2.percent.double(x, 0.75)
  expect_s3_class(ptype2_pd, "percent")
  expect_equal(length(ptype2_pd), 0L)

  ptype2_dp <- vec_ptype2.double.percent(0.5, x)
  expect_s3_class(ptype2_dp, "percent")
  expect_equal(length(ptype2_dp), 0L)

  double_cast <- vec_cast.double.percent(0.5, x)
  expect_type(double_cast, "double")
  expect_false(inherits(double_cast, "percent"))
  expect_equal(double_cast, 0.5)
})
