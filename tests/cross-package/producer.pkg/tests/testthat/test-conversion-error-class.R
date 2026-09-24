# Crate-level `conversion_error_class` default
#
# producer.pkg's Cargo.toml sets `[package.metadata.miniextendr]
# conversion_error_class = ["producer_error_argument", "producer_error"]`.
# Every argument-conversion condition of the crate carries those classes: after
# the error type's own `RConditionError` classes (duplicates skipped), and on
# their own for a plain `SexpError`. `kind` stays "conversion".

layers <- c("rust_error", "simpleError", "error", "condition")

test_that("a plain conversion error gets the crate default classes", {
  e <- tryCatch(producer_int_twice("a"), error = function(e) e)
  expect_equal(class(e), c("producer_error_argument", "producer_error", layers))
  expect_equal(e$kind, "conversion")
  expect_equal(e$param, "x")
  expect_match(
    conditionMessage(e),
    "failed to convert parameter 'x' to i32: type mismatch",
    fixed = TRUE
  )
  expect_equal(tryCatch(producer_int_twice(NA_integer_), producer_error = function(e) e$param), "x")
  expect_identical(producer_int_twice(2L), 4L)
})

test_that("the error type's own classes come first, without duplicates", {
  e <- tryCatch(producer_count_twice(-1L), error = function(e) e)
  expect_equal(
    class(e),
    c("producer_error_not_positive", "producer_error", "producer_error_argument", layers)
  )
  expect_equal(e$kind, "conversion")
  expect_equal(e$param, "n")
  expect_equal(e$value, -1L)
  expect_equal(
    conditionMessage(e),
    "failed to convert parameter 'n' to ProducerCount: expected a positive count, got -1"
  )

  e <- tryCatch(producer_count_twice("a"), error = function(e) e)
  expect_equal(
    class(e),
    c("producer_error_not_integer", "producer_error", "producer_error_argument", layers)
  )
  expect_identical(producer_count_twice(3L), 6L)
})
