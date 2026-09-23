# Tests for AsNumeric / AsNumericVec: an argument read the way as.numeric()
# reads it, from numbers, text, or factor labels, with NA of any type as NA.
#
# Messages are matched on the marker's own text only (fixed = TRUE); the
# "failed to convert parameter" prefix belongs to the macro's conversion error.

as_num_vec <- function(x) miniextendr:::test_as_numeric_vec(x)
as_num <- function(x) miniextendr:::test_as_numeric(x)
as_num_opt <- function(x) miniextendr:::test_as_numeric_opt(x)

# region: numbers pass through or widen

test_that("AsNumericVec keeps doubles, including NaN apart from NA", {
  expect_identical(as_num_vec(c(1.5, NA, NaN, -Inf)), c(1.5, NA, NaN, -Inf))
  expect_identical(as_num_vec(c(1, NA) * 2), c(2, NA))
  expect_identical(as_num_vec(double(0)), double(0))
})

test_that("AsNumericVec widens integer and logical", {
  expect_identical(as_num_vec(c(1L, NA, -3L)), c(1, NA, -3))
  expect_identical(as_num_vec(c(TRUE, NA, FALSE)), c(1, NA, 0))
})

test_that("AsNumericVec maps NA of every type to NA_real_", {
  for (na in list(NA_real_, NA_integer_, NA, NA_character_, factor(NA))) {
    expect_identical(as_num_vec(na), NA_real_)
  }
})

# endregion

# region: character parses as as.numeric() does

test_that("AsNumericVec parses spaces, Inf, exponents and hex", {
  expect_identical(
    as_num_vec(c(" 1.5 ", "Inf", "-inf", "1e3", "0x1A", "0x1p3", "+3", "5.")),
    c(1.5, Inf, -Inf, 1000, 26, 8, 3, 5)
  )
})

test_that("AsNumericVec reads blank strings and the NA token as NA", {
  # as.numeric("") and as.numeric("   ") give NA without a warning; the
  # token "NA" is read as missing, as scan() and type.convert() do.
  expect_identical(
    as_num_vec(c("", "   ", "NA", " NA ", NA_character_, "2")),
    c(NA, NA, NA, NA, NA, 2)
  )
})

test_that("AsNumericVec agrees with as.numeric() on every number it accepts", {
  ok <- c(
    " 1.5 ", "Inf", "-inf", "Infinity", "1e3", "1E-2", "0x1A", "0X1a",
    "0x1p3", "+3", "  +.5", "5.", "NaN", "-nan", "1e999", "-1e999",
    "1e-400", "\t2\n", "", "   ", NA
  )
  expect_no_warning(reference <- as.numeric(ok))
  expect_identical(as_num_vec(ok), reference)
})

# endregion

# region: factors are read by their labels

test_that("AsNumericVec reads factor labels, not codes", {
  # Codes are c(1L, 2L); the labels are 10 and 2.
  expect_identical(as_num_vec(factor(c("10", "2"))), c(10, 2))
  expect_identical(as_num_vec(factor(c("5", NA, "", "5"))), c(5, NA, NA, 5))
  expect_identical(as_num_vec(factor(c("3", "1"), ordered = TRUE)), c(3, 1))
})

test_that("AsNumericVec names a bad factor label at every element using it", {
  expect_error(
    as_num_vec(factor(c("1", "n/a", "n/a"))),
    'non-numeric value(s): "n/a", "n/a" (elements 2, 3)',
    fixed = TRUE
  )
})

# endregion

# region: failures are collected

test_that("AsNumericVec reports every non-numeric value with 1-based elements", {
  expect_error(
    as_num_vec(c("1", "n/a", "3", "4", "<0.1")),
    'non-numeric value(s): "n/a", "<0.1" (elements 2, 5)',
    fixed = TRUE
  )
})

test_that("AsNumericVec lists at most 10 values, then counts the rest", {
  bad <- c(paste0("x", 1:12), "7")
  msg <- tryCatch(as_num_vec(bad), error = conditionMessage)
  expect_match(msg, 'non-numeric value(s): "x1", "x2",', fixed = TRUE)
  expect_match(
    msg,
    '"x10" (elements 1, 2, 3, 4, 5, 6, 7, 8, 9, 10); and 2 more',
    fixed = TRUE
  )
  expect_no_match(msg, '"x11"', fixed = TRUE)
})

test_that("AsNumericVec refuses other types at the R boundary", {
  msg <- "'x' must be numeric, logical, character, or factor"
  expect_error(as_num_vec(list(1)), msg, fixed = TRUE)
  expect_error(as_num_vec(as.raw(1)), msg, fixed = TRUE)
  expect_error(as_num_vec(1i), msg, fixed = TRUE)
})

# endregion

# region: scalar AsNumeric and Option<AsNumeric>

test_that("AsNumeric reads one value of any accepted type", {
  expect_identical(as_num(2L), 2)
  expect_identical(as_num(" 1e3 "), 1000)
  expect_identical(as_num(factor("10")), 10)
  expect_identical(as_num(NA_character_), NA_real_)
  expect_identical(as_num(TRUE), 1)
})

test_that("AsNumeric requires length 1 and names a bad value", {
  expect_error(as_num(c(1, 2)), "'x' must have length 1", fixed = TRUE)
  expect_error(as_num(character(0)), "'x' must have length 1", fixed = TRUE)
  expect_error(
    as_num("n/a"),
    'non-numeric value(s): "n/a" (element 1)',
    fixed = TRUE
  )
  expect_error(
    as_num(list(1)),
    "'x' must be numeric, logical, character, or factor",
    fixed = TRUE
  )
})

test_that("Option<AsNumeric> takes NULL and keeps the marker's rules", {
  expect_identical(as_num_opt(NULL), NA_real_)
  expect_identical(as_num_opt("4"), 4)
  expect_error(
    as_num_opt(list(1)),
    "'x' must be NULL or numeric, logical, character, or factor",
    fixed = TRUE
  )
})

# endregion
