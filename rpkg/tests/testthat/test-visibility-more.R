test_that("force_invisible_i32 returns 42 invisibly", {
  res <- withVisible(force_invisible_i32())
  expect_false(res$visible)
  expect_identical(res$value, 42L)
})

test_that("force_visible_unit returns visibly", {
  res <- withVisible(force_visible_unit())
  expect_true(res$visible)
  expect_null(res$value)
})

test_that("function with interrupt checking returns correct value", {
  expect_equal(with_interrupt_check(5L), 10L)
})

test_that("non-exported function uses default and explicit args", {
  expect_equal(miniextendr:::greet_hidden(), "Hello, World!")
  expect_equal(miniextendr:::greet_hidden("Bob"), "Hello, Bob!")
})

# Result handling tests

test_that("result_null_on_err returns value on Ok", {
  expect_equal(result_null_on_err(5L), 10L)
  expect_equal(result_null_on_err(0L), 0L)
})

test_that("result_null_on_err returns NULL on Err", {
  expect_null(result_null_on_err(-1L))
  expect_null(result_null_on_err(-100L))
})

test_that("result_unwrap_in_r returns value on Ok", {
  expect_equal(result_unwrap_in_r(5L), 10L)
  expect_equal(result_unwrap_in_r(0L), 0L)
})

test_that("result_unwrap_in_r returns list(error=...) on Err", {
  res <- result_unwrap_in_r(-1L)
  expect_type(res, "list")
  expect_true("error" %in% names(res))
  expect_true(grepl("negative input", res$error))
})

# Return-visibility markers (#1213): nothing is invisible unless it says so,
# except a bare function whose R value is NULL. Fixtures: visibility_tests.rs.

test_that("unit-NULL bare functions stay invisible; markers override in both directions", {
  expect_false(withVisible(invisibly_return_no_arrow())$visible)
  expect_false(withVisible(invisibly_return_arrow())$visible)
  expect_false(withVisible(invisibly_option_return_some())$visible)
  expect_false(withVisible(invisibly_result_return_ok())$visible)

  res <- withVisible(marker_invisible_i32())
  expect_false(res$visible)
  expect_identical(res$value, 7L)
  expect_false(withVisible(marker_invisible_unit())$visible)
  res <- withVisible(marker_visible_unit())
  expect_true(res$visible)
  expect_null(res$value)
  expect_false(withVisible(marker_and_attr_agree())$visible)

  # The marker is transparent to the Option shape: a bare function's
  # `Option<i32>` `None` is `NA`, exactly as without the marker, and it stays
  # invisible either way.
  res <- withVisible(marker_invisible_option(TRUE))
  expect_false(res$visible)
  expect_identical(res$value, 3L)
  res <- withVisible(marker_invisible_option(FALSE))
  expect_false(res$visible)
  expect_identical(res$value, NA_integer_)
})

test_that("R6 receiver-returning methods are visible unless marked", {
  counter <- VisibilityCounter$new()

  res <- withVisible(counter$tick())
  expect_true(res$visible)
  expect_identical(res$value, counter)
  expect_false(withVisible(counter$tick_quietly())$visible)
  expect_false(withVisible(counter$tick_attr())$visible)

  res <- withVisible(counter$add(2L))
  expect_true(res$visible)
  expect_identical(res$value, counter)
  res <- withVisible(counter$add_quietly(2L))
  expect_false(res$visible)
  expect_identical(res$value, counter)
  expect_identical(counter$value(), 7L)

  res <- withVisible(counter$peek())
  expect_false(res$visible)
  expect_identical(res$value, 7L)

  snap <- withVisible(counter$snapshot())
  expect_false(snap$visible)
  expect_true(inherits(snap$value, "VisibilityCounter"))
  expect_identical(snap$value$value(), 7L)

  # Chaining and pipes are unaffected by visibility.
  expect_identical(counter$tick()$tick_quietly()$tick_attr()$value(), 10L)
  expect_identical((counter |> (\(x) x$add(1L))() |> (\(x) x$add_quietly(1L))())$value(), 12L)
  expect_true(withVisible(counter$value())$visible)
})

test_that("S3 void methods return x visibly unless marked", {
  gauge <- new_visibilitygauge()
  res <- withVisible(nudge_gauge(gauge))
  expect_true(res$visible)
  expect_identical(res$value, gauge)
  res <- withVisible(quiet_nudge_gauge(gauge))
  expect_false(res$visible)
  expect_identical(res$value, gauge)
  res <- withVisible(gauge_level(gauge))
  expect_false(res$visible)
  expect_identical(res$value, 2L)
})

test_that("trait void methods return the receiver visibly unless the declaration marks them", {
  bell <- Bell$new()
  expect_true(withVisible(bell$QuietBell$ring())$visible)
  expect_false(withVisible(bell$QuietBell$ring_quietly())$visible)
  res <- withVisible(bell$QuietBell$rings_quietly())
  expect_false(res$visible)
  expect_identical(res$value, 2L)
  expect_identical(bell$rings(), 2L)
})
