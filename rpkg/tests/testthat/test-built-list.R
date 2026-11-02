# BuiltList: the rooted plain-list finish of NamedListBuilder, classed and
# read by R while Rust builds it (built_list_tests.rs).

test_that("a BuiltList comes back with its class, values and attribute", {
  s <- built_list_summary(c(1, 2, 6))
  expect_identical(class(s), c("summary_of", "list"))
  expect_identical(names(s), c("n", "mean", "values"))
  expect_identical(s$n, 3L)
  expect_identical(s$mean, 3)
  expect_identical(s$values, c(1, 2, 6))
  # Computed by R from the list while the handle rooted it.
  expect_identical(attr(s, "fields"), "n,mean,values")
})

test_that("an empty input gives an NA mean and an empty values vector", {
  s <- built_list_summary(numeric())
  expect_identical(s$n, 0L)
  expect_identical(s$mean, NA_real_)
  expect_identical(s$values, numeric())
  expect_identical(attr(s, "fields"), "n,mean,values")
})

test_that("NA in the input is NA in the mean", {
  s <- built_list_summary(c(1, NA))
  expect_true(is.na(s$mean))
  expect_identical(s$values, c(1, NA))
})

test_that("a BuiltList survives gctorture while R reads it mid-build", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE))
  s <- gc_stress_built_list()
  gctorture(FALSE)
  expect_identical(s$n, 64L)
  expect_identical(s$values, as.numeric(1:64))
  expect_identical(attr(s, "fields"), "n,mean,values")
})
