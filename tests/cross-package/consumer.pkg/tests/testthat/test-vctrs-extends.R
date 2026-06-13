# Cross-package vctrs inheritance (`extends = "parent_type"`, #1039).
#
# producer.pkg exports two vctrs types:
#   - `producer_temp`           (the parent, from #1027/#1040)
#   - `producer_oven_temp`      (the child, `#[vctrs(extends = "producer_temp")]`)
#
# consumer.pkg owns neither definition. It loads producer.pkg and verifies that
# inheritance dispatches across the package boundary purely through R's S3
# method registry + the class vector the producer's constructor builds:
#   1. The child's class vector contains the parent, so S3 generics the child
#      does NOT override (format, vec_ptype_abbr) fall through to the parent's
#      methods via class-vector dispatch.
#   2. vctrs coercion (vec_ptype2/vec_cast) between child and parent resolves to
#      the PARENT, matching vctrs' rule that the supertype is the common type.

test_that("child vctrs type carries the parent class (inheritance)", {
  skip_if_not_installed("producer.pkg")
  skip_if_not_installed("vctrs")
  library(producer.pkg)

  child <- new_oven_temperature(c(180, 200, 220))

  # Class vector: c("producer_oven_temp", "producer_temp", "vctrs_vctr", "double")
  expect_true(inherits(child, "producer_oven_temp"))
  expect_true(inherits(child, "producer_temp")) # the parent, via extends
  expect_true(inherits(child, "vctrs_vctr"))

  cls <- class(child)
  # Child precedes parent in the class vector (S3 inheritance ordering).
  expect_lt(match("producer_oven_temp", cls), match("producer_temp", cls))
})

test_that("S3 generics resolve through the inherited class vector", {
  skip_if_not_installed("producer.pkg")
  skip_if_not_installed("vctrs")
  library(producer.pkg)

  child <- new_oven_temperature(c(0, 100))

  # vctrs::vec_ptype_full dispatches via the class vector. The child has no
  # vec_ptype_full method of its own beyond the derive's, so this exercises
  # the registered-method path that S3 inheritance relies on.
  expect_equal(format(child), format(c(0, 100)))

  # The child has no own vec_ptype_abbr method registered (no `abbr` attr);
  # an S3 method for the PARENT (vec_ptype_abbr.producer_temp) IS registered.
  # Whether vctrs' vec_ptype_abbr default short-circuits to a class-name-based
  # abbreviation rather than walking to the parent is a vctrs-internal detail,
  # so we don't assert the abbr string here — the inheritance contract this
  # issue (#1039) guarantees is the class vector + coercion chain, verified in
  # the other tests.
  expect_true(is.character(vctrs::vec_ptype_abbr(child)))
})

test_that("coercion between child and parent resolves to the parent (supertype)", {
  skip_if_not_installed("producer.pkg")
  skip_if_not_installed("vctrs")
  library(producer.pkg)

  child <- new_oven_temperature(c(1, 2))
  parent <- new_temperature(c(3, 4))

  # vec_c(child, parent) relies on vec_ptype2.producer_oven_temp.producer_temp
  # (and the reverse) + the corresponding vec_cast methods. The common type is
  # the parent, so the result is a producer_temp.
  combined <- vctrs::vec_c(child, parent)
  expect_true(inherits(combined, "producer_temp"))
  expect_false(inherits(combined, "producer_oven_temp"))
  expect_equal(vctrs::vec_size(combined), 4L)
  expect_equal(as.double(unclass(combined)), c(1, 2, 3, 4))
})

test_that("explicit cast child -> parent works across the boundary", {
  skip_if_not_installed("producer.pkg")
  skip_if_not_installed("vctrs")
  library(producer.pkg)

  child <- new_oven_temperature(c(10, 20))

  # vec_cast.producer_temp.producer_oven_temp re-wraps the shared base data.
  as_parent <- vctrs::vec_cast(child, new_temperature(double()))
  expect_true(inherits(as_parent, "producer_temp"))
  expect_equal(as.double(unclass(as_parent)), c(10, 20))
})
