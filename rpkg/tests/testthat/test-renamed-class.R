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

# The names a trait impl builds from a renamed class take its R name too
# (#2285): the S4 trait statics, the S7 trait shortcuts and the S7 trait
# namespace. The Rust-named ones are gone, not kept as aliases.
test_that("an S4 trait impl's static on a renamed class is named after it", {
  shape <- RenamedShape(3)
  expect_true(methods::is(shape, "RenamedShape"))
  expect_identical(s4_trait_Area_area(shape), 9)
  expect_identical(RenamedShape_Area_unit(), "shape")
  ns <- asNamespace("miniextendr")
  expect_false(exists("RenamedShapeS4_Area_unit", envir = ns, inherits = FALSE))
  exports <- getNamespaceExports("miniextendr")
  expect_true("RenamedShape_Area_unit" %in% exports)
  expect_false("RenamedShapeS4_Area_unit" %in% exports)
})

test_that("an S7 trait impl's shortcut and namespace on a renamed class are named after it", {
  skip_if_not_installed("S7")
  point <- RenamedPoint(3)
  expect_identical(s7_trait_Area_area(point), 9)
  expect_identical(RenamedPoint_area(point), 9)
  expect_identical(attr(RenamedPoint, "Area")$unit(), "point")
  ns <- asNamespace("miniextendr")
  expect_identical(get(".RenamedPoint__Area", envir = ns, inherits = FALSE)$unit(), "point")
  rust_named <- c("RenamedPointS7_area", ".RenamedPointS7__Area", ".s7_class_RenamedPointS7")
  for (name in rust_named) {
    expect_false(exists(name, envir = ns, inherits = FALSE), info = name)
  }
  exports <- getNamespaceExports("miniextendr")
  expect_true("RenamedPoint_area" %in% exports)
  expect_false("RenamedPointS7_area" %in% exports)
})
