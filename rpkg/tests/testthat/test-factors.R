test_that("factor color helpers describe and return colors", {
  levels <- factor_color_levels()
  expect_equal(levels, c("Red", "Green", "Blue"))

  colors <- factor("Green", levels = levels)
  expect_equal(factor_describe_color(colors), "The color is green!")

  result <- factor_get_color("blue")
  expect_s3_class(result, "factor")
  expect_equal(levels(result), levels)
  expect_equal(as.character(result), "Blue")

  all_colors <- factor_get_all_colors()
  expect_s3_class(all_colors, "factor")
  expect_equal(as.character(all_colors), c("Red", "Green", "Blue"))
  expect_equal(levels(all_colors), levels)
})

test_that("status and priority helpers expose renamed levels", {
  status_levels <- factor_status_levels()
  expect_equal(status_levels, c("in_progress", "completed", "not_started"))

  status <- factor("completed", levels = status_levels)
  expect_equal(factor_describe_status(status), "Work is completed")

  priority_levels <- factor_priority_levels()
  expect_equal(priority_levels, c("low", "med", "high"))

  priority <- factor("med", levels = priority_levels)
  expect_equal(factor_describe_priority(priority), "Medium priority")
})

test_that("factor count helpers handle counts and NA", {
  colors <- factor(c("Red", "Red", "Blue"), levels = factor_color_levels())
  expect_equal(factor_count_colors(colors), c(2L, 0L, 1L))

  with_na <- factor(c("Red", NA, "Green"), levels = factor_color_levels())
  expect_equal(factor_colors_with_na(with_na), c("red", "NA", "green"))
})

test_that("generic unit enum Signal<T> emits a valid factor column", {
  result <- make_signal_factor()
  expect_s3_class(result, "factor")
  expect_equal(levels(result), c("Red", "Yellow", "Green"))
  expect_equal(as.character(result), c("Red", NA, "Green", "Yellow"))
})

test_that("factor enum with variants named like associated items round-trips (#1730)", {
  levels <- factor_outcome_levels()
  expect_equal(levels, c("error", "value", "output"))
  for (lvl in levels) {
    result <- factor_outcome_round_trip(factor(lvl, levels = levels))
    expect_s3_class(result, "factor")
    expect_equal(levels(result), levels)
    expect_equal(as.character(result), lvl)
  }

  stage_levels <- factor_outcome_stage_levels()
  expect_equal(
    stage_levels,
    c(
      "Error.error", "Error.value", "Error.output",
      "Value.error", "Value.value", "Value.output"
    )
  )
  for (lvl in c("Error.error", "Value.output")) {
    result <- factor_outcome_stage_round_trip(factor(lvl, levels = stage_levels))
    expect_s3_class(result, "factor")
    expect_equal(levels(result), stage_levels)
    expect_equal(as.character(result), lvl)
  }
})

test_that("a Vec of an RFactor enum is a factor, like the scalar (#2365)", {
  levels <- factor_color_levels()

  colors <- factor_colors_vec()
  expect_s3_class(colors, "factor")
  expect_equal(levels(colors), levels)
  expect_equal(as.character(colors), c("Blue", "Red", "Blue"))

  boxed <- factor_colors_boxed()
  expect_s3_class(boxed, "factor")
  expect_equal(levels(boxed), levels)
  expect_equal(as.character(boxed), c("Green", "Red"))

  maybe <- factor_colors_maybe(TRUE)
  expect_s3_class(maybe, "factor")
  expect_equal(levels(maybe), levels)
  expect_equal(as.character(maybe), "Green")
  expect_null(factor_colors_maybe(FALSE))

  missing <- factor_colors_with_missing()
  expect_s3_class(missing, "factor")
  expect_equal(levels(missing), levels)
  expect_equal(as.character(missing), c("Green", NA, "Blue"))

  stages <- factor_outcome_stages()
  expect_s3_class(stages, "factor")
  expect_equal(levels(stages), factor_outcome_stage_levels())
  expect_equal(as.character(stages), c("Value.output", "Error.error"))
})

test_that("containers of an RFactor enum read a factor back (#2365)", {
  levels <- factor_color_levels()
  colors <- factor(c("Green", "Blue", "Red"), levels = levels)
  expect_identical(factor_colors_round_trip(colors), colors)
  expect_identical(factor_colors_round_trip(factor_colors_vec()), factor_colors_vec())
  expect_identical(
    factor_colors_boxed_reversed(colors),
    factor(c("Red", "Blue", "Green"), levels = levels)
  )
  expect_identical(
    factor_colors_round_trip(factor(character(), levels = levels)),
    factor(character(), levels = levels)
  )

  with_na <- factor(c("Red", NA, "Blue"), levels = levels)
  expect_identical(factor_colors_option_round_trip(with_na), with_na)
  expect_identical(
    factor_colors_option_round_trip(factor_colors_with_missing()),
    factor_colors_with_missing()
  )

  expect_equal(
    factor_describe_optional_color(factor("Blue", levels = levels)),
    "The color is blue!"
  )
  expect_equal(factor_describe_optional_color(NULL), "No color")
  expect_equal(factor_describe_optional_color(NA), "No color")
  expect_equal(
    factor_describe_optional_color(factor(NA, levels = levels)),
    "No color"
  )

  stages <- factor_outcome_stages()
  expect_identical(factor_outcome_stages_round_trip(stages), stages)
})

test_that("containers of an RFactor enum refuse what the scalar refuses (#2365)", {
  levels <- factor_color_levels()
  expect_error(factor_colors_round_trip(c("Red", "Blue")), "factor")
  expect_error(
    factor_colors_round_trip(factor("Red", levels = c("Red", "Green"))),
    "levels"
  )
  expect_error(
    factor_colors_round_trip(factor(c("Red", NA), levels = levels)),
    "NA is not allowed"
  )
  expect_error(factor_colors_option_round_trip("Red"), "factor")
})

test_that("a Vec of a MatchArg enum stays a character vector (#2365)", {
  result <- match_arg_return_modes(c("Safe", "Fast"))
  expect_type(result, "character")
  expect_equal(result, c("Safe", "Fast"))
})
