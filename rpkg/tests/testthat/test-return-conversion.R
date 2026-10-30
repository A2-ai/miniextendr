# Failed return conversions (#2272) and computed getters returning `Result`
# (#2269). Fixtures: src/rust/return_conversion_tests.rs.
#
# A return value whose `IntoR` conversion fails raises the classed error of a
# `Result` return (`e$kind == "result_err"`): the error's classes and fields
# when it implements `RConditionData`, else its `Display` text, and never a
# panic message with a file location.

caught <- function(expr) tryCatch(expr, error = identity)

capped <- miniextendr:::return_conversion_capped
plain <- miniextendr:::return_conversion_plain
capped_ok <- miniextendr:::return_conversion_capped_ok
raise_by_hand <- miniextendr:::return_conversion_raise_by_hand
CapGauge <- miniextendr:::CapGauge

# region: fallible `IntoR` returns

test_that("a return value whose conversion fails raises its classed error", {
  expect_identical(capped(3L), 3L)
  e <- caught(capped(20L))
  expect_s3_class(e, "over_cap_error")
  expect_s3_class(e, "cap_error")
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "result_err")
  expect_identical(conditionMessage(e), "20 exceeds the cap 10")
  expect_identical(e$value, 20L)
  expect_identical(e$cap, 10L)
  expect_equal(conditionCall(e), quote(capped(20L)))
})

test_that("an error with only Display gives its text, with no file location", {
  expect_identical(plain(3L), 3L)
  e <- caught(plain(20L))
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "result_err")
  expect_identical(conditionMessage(e), "20 is not convertible")
  expect_equal(conditionCall(e), quote(plain(20L)))
})

test_that("the Ok value of a Result return converts like any return value", {
  expect_identical(capped_ok(3L), 3L)
  e <- caught(capped_ok(20L))
  expect_s3_class(e, "over_cap_error")
  expect_identical(e$kind, "result_err")
  expect_identical(e$value, 20L)
})

test_that("a method's return conversion raises the same error", {
  expect_identical(CapGauge$new(4L)$capped(), 4L)
  gauge <- CapGauge$new(20L)
  e <- caught(gauge$capped())
  expect_s3_class(e, "over_cap_error")
  expect_identical(e$kind, "result_err")
  expect_identical(conditionMessage(e), "20 exceeds the cap 10")
})

test_that("RCondition::from_data(&e).raise() raises a typed error by hand", {
  expect_identical(raise_by_hand(3L), 3L)
  e <- caught(raise_by_hand(20L))
  expect_s3_class(e, "over_cap_error")
  expect_s3_class(e, "cap_error")
  expect_identical(e$kind, "error")
  expect_identical(e$value, 20L)
  expect_identical(conditionMessage(e), "20 exceeds the cap 10")
  expect_equal(conditionCall(e), quote(raise_by_hand(20L)))
})

# endregion

# region: serde values serde refuses

test_that("an AsSerialize return serde refuses raises miniextendr_serde_error", {
  e <- caught(miniextendr:::return_conversion_serde_integer_keys())
  expect_s3_class(e, "miniextendr_serde_error")
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "result_err")
  expect_identical(conditionMessage(e), "map keys must be strings for R named lists")
  expect_false(grepl("(at ", conditionMessage(e), fixed = TRUE))

  e <- caught(miniextendr:::return_conversion_serialize_integer_keys())
  expect_s3_class(e, "miniextendr_serde_error")
  expect_identical(e$kind, "result_err")
  expect_identical(conditionMessage(e), "map keys must be strings for R named lists")

  expect_identical(
    miniextendr:::return_conversion_serde_string_keys(),
    list(a = 2.5, b = 5)
  )
})

test_that("inside a tuple the batched conversion error keeps serde's message", {
  e <- caught(miniextendr:::return_conversion_serde_in_tuple())
  expect_s3_class(e, "rust_error")
  expect_false(inherits(e, "miniextendr_serde_error"))
  expect_identical(e$kind, "result_err")
  expect_match(
    conditionMessage(e),
    "map keys must be strings for R named lists",
    fixed = TRUE
  )
  expect_false(grepl("(at ", conditionMessage(e), fixed = TRUE))
})

# endregion

# region: computed getters returning `Result`

test_that("a computed getter's Ok reads the value", {
  x <- new_computedresult(3L)
  expect_identical(x$checked, 3L)
  expect_identical(x[["checked"]], 3L)
  expect_identical(ComputedResult_get_checked(x), 3L)
  expect_identical(x$doubled_or_null, 6L)
  expect_identical(x$capped, 3L)
  expect_identical(x$value, 3L)
})

test_that("a computed getter's Err raises its classed error", {
  y <- new_computedresult(20L)
  e <- caught(y$checked)
  expect_s3_class(e, "over_cap_error")
  expect_s3_class(e, "cap_error")
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "result_err")
  expect_identical(e$value, 20L)
  expect_identical(conditionMessage(e), "20 exceeds the cap 10")
  expect_equal(conditionCall(e), quote(y$checked))

  e <- caught(y[["checked"]])
  expect_s3_class(e, "over_cap_error")
  expect_equal(conditionCall(e), quote(y[["checked"]]))

  e <- caught(ComputedResult_get_checked(y))
  expect_s3_class(e, "over_cap_error")
  expect_equal(conditionCall(e), quote(ComputedResult_get_checked(y)))
})

test_that("a computed getter's Result<T, ()> reads NULL on Err", {
  y <- new_computedresult(20L)
  expect_null(y$doubled_or_null)
  expect_null(y[["doubled_or_null"]])
  expect_null(ComputedResult_get_doubled_or_null(y))
})

test_that("as.list reads every field, so a raising field raises", {
  fields <- as.list(new_computedresult(3L))
  expect_identical(fields$checked, 3L)
  expect_identical(fields$doubled_or_null, 6L)
  expect_identical(fields$capped, 3L)
  expect_error(as.list(new_computedresult(20L)), class = "over_cap_error")
})

test_that("a computed value whose conversion fails raises its classed error", {
  y <- new_computedresult(20L)
  e <- caught(y$capped)
  expect_s3_class(e, "over_cap_error")
  expect_identical(e$kind, "result_err")
  expect_equal(conditionCall(e), quote(y$capped))
})

# endregion
