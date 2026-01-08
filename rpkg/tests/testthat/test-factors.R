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
