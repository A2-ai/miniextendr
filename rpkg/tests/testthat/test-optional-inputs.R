# `Option<DataFrame>` / `Option<Either<L, R>>` parameters:
# src/rust/optional_input_tests.rs. NULL is None; any other value converts as
# the inner type and becomes Some, so it behaves as the bare parameter does:
# the same value, the same condition class, `kind`, `e$param` and reason, the
# message's expectation prefixed with "NULL or".

layers <- c("rust_error", "simpleError", "error", "condition")
caught <- function(expr) tryCatch(expr, error = identity)

framed <- structure(
  data.frame(id = 1:3, v = c(1, NA, 3), row.names = c("a", "b", "c")),
  class = c("tbl_df", "tbl", "data.frame"),
  note = "kept"
)

# `optional` refuses `x` as `bare` does: the same condition, with "NULL or"
# in front of the expectation and the Option type in `e$rust_type`.
expect_refused_like_bare <- function(optional, bare, x, rust_type) {
  e <- caught(optional(x))
  b <- caught(bare(x))
  expect_s3_class(e, "rust_error")
  expect_identical(class(e), class(b))
  expect_identical(e$kind, b$kind)
  expect_identical(e$param, b$param)
  expect_identical(e$rust_type, rust_type)
  expect_identical(
    conditionMessage(e),
    sub("' must be ", "' must be NULL or ", conditionMessage(b), fixed = TRUE)
  )
  invisible(e)
}

test_that("the three optional declarations take NULL and their inner values", {
  skip_if_not(miniextendr_has_feature("either"), "either feature not compiled in")
  for (x in list(NULL, framed, data.frame())) {
    expect_null(optional_frame(x))
  }
  for (x in list(NULL, 1.5, NA_real_, "abc")) {
    expect_null(optional_choice(x))
  }
  for (x in list(NULL, c(1, NA), "3", factor("10"), framed)) {
    expect_null(optional_grid(x))
  }
  # What they refuse, they refuse as the conversion error.
  expect_identical(
    conditionMessage(caught(optional_frame(1))),
    "'x' must be NULL or a data frame: got numeric"
  )
  expect_identical(
    conditionMessage(caught(optional_choice(1L))),
    "'x' must be NULL or a single double or a single string: got integer"
  )
  expect_identical(
    conditionMessage(caught(optional_grid(list(1, 2)))),
    "'x' must be NULL or numeric or a data frame: got list"
  )
})

test_that("Option<DataFrame>: NULL is None, a data frame is the same object", {
  skip_if_not(miniextendr_has_feature("either"), "either feature not compiled in")
  echo <- miniextendr:::optional_frame_echo
  expect_null(echo(NULL))
  # Class, attributes, row names and NA cells are untouched: it is the frame.
  expect_identical(echo(framed), framed)
  expect_identical(attr(echo(framed), "note"), "kept")
  expect_identical(class(echo(framed)), c("tbl_df", "tbl", "data.frame"))
  expect_identical(rownames(echo(framed)), c("a", "b", "c"))
  expect_identical(echo(framed)$v, c(1, NA, 3))
  expect_identical(echo(framed), miniextendr:::required_frame_echo(framed))
  zero <- data.frame(a = integer())
  expect_identical(echo(zero), zero)
})

test_that("Option<DataFrame> refuses what DataFrame refuses, in the same words", {
  skip_if_not(miniextendr_has_feature("either"), "either feature not compiled in")
  for (x in list(1, "x", list(a = 1), TRUE)) {
    e <- expect_refused_like_bare(
      miniextendr:::optional_frame_echo,
      miniextendr:::required_frame_echo,
      x,
      "Option<DataFrame>"
    )
    expect_identical(class(e), layers)
    expect_identical(e$param, "x")
  }
  expect_identical(
    conditionMessage(caught(miniextendr:::optional_frame_echo(list(a = 1)))),
    "'x' must be NULL or a data frame: got list"
  )
  # A data frame that fails later keeps its own reason.
  expect_identical(
    conditionMessage(caught(miniextendr:::optional_frame_echo(
      structure(list(1), class = "data.frame")
    ))),
    "'x' must be NULL or a data frame: data.frame has no column names"
  )
})

test_that("Option<Either>: NULL is None, other values take the bare Either's arm", {
  skip_if_not(miniextendr_has_feature("either"), "either feature not compiled in")
  choice <- miniextendr:::optional_choice_tag
  expect_identical(choice(NULL), "none")
  for (x in list(1.5, NA_real_, "abc", NA_character_)) {
    expect_identical(choice(x), miniextendr:::required_choice_tag(x))
  }
  expect_identical(choice(1.5), "left:1.5")
  expect_identical(choice("abc"), "right:abc")
  # `String` reads NA_character_ as "" (lossy), with or without the Option.
  expect_identical(choice(NA_character_), "right:")

  grid <- miniextendr:::optional_grid_tag
  expect_identical(grid(NULL), "none")
  for (x in list(c(1, NA), "3", factor(c("10", "20")), framed)) {
    expect_identical(grid(x), miniextendr:::required_grid_tag(x))
  }
  # Text both arms could read goes to the left (number) arm, as on the bare
  # Either: arm order is unchanged by the Option layer.
  expect_identical(grid(c("1", "NA")), "numbers:2:1")
  expect_identical(grid(framed), "frame:3x2")
})

test_that("Option<Either> refuses what the bare Either refuses, in the same words", {
  skip_if_not(miniextendr_has_feature("either"), "either feature not compiled in")
  for (x in list(1L, c(1, 2), TRUE, list(1), c("a", "b"))) {
    e <- expect_refused_like_bare(
      miniextendr:::optional_choice_tag,
      miniextendr:::required_choice_tag,
      x,
      "Option<Either<f64, String>>"
    )
    expect_identical(class(e), layers)
  }
  for (x in list(list(1, 2), quote(x), structure(list(1), class = "data.frame"))) {
    expect_refused_like_bare(
      miniextendr:::optional_grid_tag,
      miniextendr:::required_grid_tag,
      x,
      "Option<Either<AsNumericVec, DataFrame>>"
    )
  }
  # Both arms' reasons survive.
  expect_identical(
    conditionMessage(caught(miniextendr:::optional_grid_tag(c("1", "n/a")))),
    conditionMessage(caught(miniextendr:::required_grid_tag(c("1", "n/a")))) |>
      sub(pattern = "' must be ", replacement = "' must be NULL or ", fixed = TRUE)
  )
})

test_that("no_na on Option<Either> is checked by the arm taken; NULL passes", {
  skip_if_not(miniextendr_has_feature("either"), "either feature not compiled in")
  f <- miniextendr:::optional_grid_no_na
  expect_identical(f(NULL), "none")
  expect_identical(f(c(1, 2)), "numbers:2:0")
  # The data frame arm keeps its NA cells: no anyNA() runs on the whole value.
  expect_identical(f(framed), "frame:3x2")
  # Numbers with an NA, or text the marker reads as NA, are refused.
  for (x in list(c(1, NA), "NA", c("1", ""), NA)) {
    e <- caught(f(x))
    expect_identical(conditionMessage(e), "'x' must not contain NA")
    expect_identical(class(e), layers)
    expect_identical(e$kind, "conversion")
    expect_identical(e$param, "x")
    expect_null(e$rust_type)
  }
})

test_that("no_na on Option<DataFrame> keeps the R guard: NA cells are refused", {
  skip_if_not(miniextendr_has_feature("either"), "either feature not compiled in")
  f <- miniextendr:::optional_frame_no_na
  expect_identical(f(NULL), "none")
  expect_identical(f(data.frame(id = 1:2, v = c(1, 2))), "frame:2x2")
  e <- caught(f(framed))
  expect_identical(class(e), layers)
  expect_identical(e$param, "x")
  # Worded as on a bare `DataFrame` parameter.
  expect_identical(conditionMessage(e), "'x' must not be NA")
  expect_null(e$rust_type)
})
