# Cross-package vctrs type export tests.
#
# producer.pkg exports a vctrs type `producer_temp` (via #[derive(Vctrs)]) and
# its constructor new_temperature(). consumer.pkg owns NONE of that type's
# definition — it merely loads producer.pkg and dispatches on the type.
#
# This proves gap #1 of #1027: a consumer can construct and dispatch on a
# producer's vctrs type purely through R's S3 method registry (populated from
# producer.pkg's S3method() NAMESPACE entries when the package loads) plus an
# imported constructor. No R_GetCCallable / vtable shim is involved — vctrs
# export rides entirely on S3 dispatch, unlike trait-ABI ExternalPtr objects.

test_that("consumer can construct a producer-owned vctrs type", {
  skip_if_not_installed("producer.pkg")
  skip_if_not_installed("vctrs")
  library(producer.pkg)

  t <- new_temperature(c(20, 25, 30))

  # The object carries the producer's class vector.
  expect_true(inherits(t, "producer_temp"))
  expect_true(inherits(t, "vctrs_vctr"))
  expect_equal(vctrs::vec_size(t), 3L)
})

test_that("producer's vctrs S3 methods dispatch from the consumer", {
  skip_if_not_installed("producer.pkg")
  skip_if_not_installed("vctrs")
  library(producer.pkg)

  t <- new_temperature(c(0, 100))

  # format.producer_temp (a producer-registered S3 method) dispatches here.
  expect_equal(format(t), format(c(0, 100)))

  # vec_ptype_abbr.producer_temp resolves the abbreviation set in Rust.
  expect_equal(vctrs::vec_ptype_abbr(t), "degC")
})

test_that("self-coercion (vec_c) works on a producer vctrs type from consumer", {
  skip_if_not_installed("producer.pkg")
  skip_if_not_installed("vctrs")
  library(producer.pkg)

  a <- new_temperature(c(1, 2))
  b <- new_temperature(c(3, 4))

  # vec_c relies on vec_ptype2.producer_temp.producer_temp +
  # vec_cast.producer_temp.producer_temp, both registered by producer.pkg.
  combined <- vctrs::vec_c(a, b)
  expect_true(inherits(combined, "producer_temp"))
  expect_equal(vctrs::vec_size(combined), 4L)
  expect_equal(as.double(unclass(combined)), c(1, 2, 3, 4))
})

test_that("cross-coercion to double works (coerce = double)", {
  skip_if_not_installed("producer.pkg")
  skip_if_not_installed("vctrs")
  library(producer.pkg)

  t <- new_temperature(c(10, 20))

  # vec_cast.double.producer_temp (from `coerce = "double"`) lets the consumer
  # cast a producer temperature to a bare double.
  as_dbl <- vctrs::vec_cast(t, double())
  expect_identical(as_dbl, c(10, 20))

  # And the reverse: vec_cast.producer_temp.double.
  as_temp <- vctrs::vec_cast(c(5, 15), new_temperature(double()))
  expect_true(inherits(as_temp, "producer_temp"))
  expect_equal(as.double(unclass(as_temp)), c(5, 15))
})
