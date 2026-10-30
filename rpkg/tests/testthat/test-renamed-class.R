# Classes renamed with `class = "..."`, named from other expansions (#2265,
# #2266). Fixtures: rpkg/src/rust/renamed_class_tests.rs.

test_that("a free function's WrapAsS3 return has the renamed S3 class", {
  for (make in list(renamed_tally_wrapped, renamed_tally_wrapped_attr)) {
    tally <- make(4L)
    expect_identical(class(tally), "renamed_tally")
    expect_identical(custom_get(tally), 4L)
  }
})

test_that("a free function's WrapAsR6 return is the renamed R6 class", {
  board <- renamed_board_wrapped(6L)
  expect_true(R6::is.R6(board))
  expect_s3_class(board, "RenamedBoard")
  expect_identical(board$width(), 6L)
})

test_that("a free function's WrapAsS7 return is the renamed S7 class", {
  skip_if_not_installed("S7")
  point <- renamed_point_wrapped(2.5)
  expect_true(S7::S7_inherits(point, RenamedPoint))
  expect_identical(renamed_point_x(point), 2.5)
})

test_that("an S3 trait impl on a renamed class dispatches on its R name", {
  tally <- renamed_tally$new(10L)
  expect_identical(class(tally), "renamed_tally")
  expect_identical(custom_get(tally), 10L)
  custom_add(tally, 5L)
  expect_identical(custom_get(tally), 15L)
  expect_true(is.function(getS3method("custom_get", "renamed_tally")))
  expect_null(getS3method("custom_get", "RenamedTallyS3", optional = TRUE))
  expect_identical(renamed_tally$MatrixCounter$default_value(), 11L)
})

test_that("an R6 trait impl on a renamed class lives in its R class", {
  board <- RenamedBoard$new(3L)
  expect_identical(RenamedBoard$MatrixCounter$custom_get(board), 3L)
  RenamedBoard$MatrixCounter$custom_add(board, 4L)
  expect_identical(board$width(), 7L)
  expect_identical(RenamedBoard$MatrixCounter$default_value(), 12L)
})
