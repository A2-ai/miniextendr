test_that("an Env instance returns usable objects in each selected class system", {
  factory <- WrapFactory$new()
  for (name in c("build", "build_attr")) {
    board <- factory[[name]](3L, 4L)
    expect_s3_class(board, "WrapBoard")
    expect_identical(board$dimensions(), c(3L, 4L))
  }
  for (suffix in c("", "_attr")) {
    expect_identical(s7_value(factory[[paste0("s7", suffix)]]()), 7L)
    expect_identical(s4_value(factory[[paste0("s4", suffix)]]()), 4L)
    expect_identical(s3_value(factory[[paste0("s3", suffix)]]()), 3L)
    expect_identical(factory[[paste0("env", suffix)]]()$value(), 8L)
  }
})

test_that("explicit free-function and trait factory returns remain callable", {
  for (make in list(wrapped_board, wrapped_board_attr)) {
    board <- make(5L, 6L)
    expect_identical(board$dimensions(), c(5L, 6L))
    copy <- WrapBoard$WrapBoardFactory$copy_board(board)
    expect_identical(copy$dimensions(), c(5L, 6L))
  }
  board <- WrapBoard$WrapBoardFactory$create_board(7L, 8L)
  expect_identical(board$dimensions(), c(7L, 8L))
})

test_that("wrapping preserves container boundaries, errors, and visibility", {
  factory <- WrapFactory$new()
  expect_identical(factory$fallible(FALSE)$dimensions(), c(2L, 5L))
  expect_error(factory$fallible(TRUE), "board construction failed", class = "rust_error")
  expected <- list(c(1L, 2L), c(3L, 4L))
  expect_identical(lapply(factory$boards(), function(x) x$dimensions()), expected)
  expect_identical(lapply(factory$optional_boards(TRUE), function(x) x$dimensions()), expected)
  expect_error(factory$optional_boards(FALSE), "returned no value", class = "rust_error")
  expect_identical(factory$optional_board(TRUE)$dimensions(), c(6L, 7L))
  expect_error(factory$optional_board(FALSE), class = "rust_error")
  hidden <- withVisible(factory$hidden())
  expect_false(hidden$visible)
  expect_identical(hidden$value$dimensions(), c(8L, 9L))
})

test_that("explicit vctrs wrapping retains record behavior and parent classes", {
  skip_if_missing_feature("vctrs")
  for (make in list(wrapped_record, wrapped_record_attr)) {
    value <- make()
    expect_identical(class(value), c("WrappedRecord", "vctrs_rcrd", "vctrs_vctr"))
    expect_identical(vctrs::vec_size(value), 2L)
    expect_identical(vctrs::field(value, "x"), c(1L, 2L))
    expect_identical(vctrs::field(value, "y"), c(3L, 4L))
    expect_identical(class(vctrs::vec_slice(value, 1L)), class(value))
  }
})

test_that("worker returns produce usable explicitly selected objects", {
  skip_if_missing_feature("worker-thread")
  expect_identical(wrapped_board_worker()$dimensions(), c(9L, 10L))
})

test_that("cross-class and trait factory wrapping survives GC stress", {
  skip_on_cran()
  factory <- WrapFactory$new()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  board <- factory$build(3L, 4L)
  expect_identical(board$dimensions(), c(3L, 4L))
  expect_identical(WrapBoard$WrapBoardFactory$copy_board(board)$dimensions(), c(3L, 4L))
})

test_that("explicit free-function class containers unwrap before converting", {
  for (make in list(wrapped_optional_boards, wrapped_optional_boards_attr)) {
    expect_error(make(FALSE), "returned no value", class = "rust_error")
    expect_identical(make(TRUE)[[1L]]$dimensions(), c(1L, 2L))
  }
  expect_identical(wrapped_result_boards(FALSE)[[1L]]$dimensions(), c(1L, 2L))
  expect_error(wrapped_result_boards(TRUE), "board list failed", class = "rust_error")
})
