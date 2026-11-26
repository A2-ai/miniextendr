# Class lambdas raise what the Rust side refuses (#1675).

test_that("an S7 integer property takes an integer and refuses everything else", {
  ns <- asNamespace("miniextendr")
  g <- ns$S7Gauge(10L)
  g@level <- 3L
  expect_identical(g@level, 3L)
  # S7 calls the setter without checking the class, so the refusal is the
  # setter's: before #1675 each of these left the level at 3 without a word.
  for (value in list(4, 3.5, "4", NA, NA_integer_, 1:2)) {
    err <- tryCatch(g@level <- value, error = identity)
    expect_s3_class(err, "rust_error")
    expect_match(conditionMessage(err), "^'value' must be a single integer: ")
    expect_identical(err$param, "value")
    expect_identical(g@level, 3L)
  }
})

test_that("an S7 property getter raises its Err instead of returning it", {
  ns <- asNamespace("miniextendr")
  g <- ns$S7Gauge(2L)
  expect_identical(g@checked, 2L)
  g@level <- -1L
  expect_error(g@checked, "level -1 is negative", class = "rust_error")
})

test_that("an R6 deep_clone handler's Err is raised, not stored in the clone", {
  ns <- asNamespace("miniextendr")
  ok <- ns$R6CloneRefusal$new(4L)
  expect_identical(ok$clone(deep = TRUE)$value(), 4L)
  refused <- ns$R6CloneRefusal$new(-4L)
  expect_error(refused$clone(deep = TRUE), "value -4 is not cloned", class = "rust_error")
})
