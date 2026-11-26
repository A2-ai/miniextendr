# Tests for AsInteger / AsIntegerVec: an argument read the way as.integer()
# reads it, from numbers, text, or factor labels, with NA of any type as NA,
# refusing the values as.integer() would truncate or make NA.
#
# Messages are matched on the marker's own text only (fixed = TRUE) unless the
# whole message is the point; the `'x' must be integer: ` prefix belongs to
# the macro's conversion error.

as_int_vec <- function(x) miniextendr:::test_as_integer_vec(x)
as_int <- function(x) miniextendr:::test_as_integer(x)
as_int_opt <- function(x) miniextendr:::test_as_integer_opt(x)
as_int_no_na <- function(x) miniextendr:::test_as_integer_no_na(x)

# region: numbers pass through when whole

test_that("AsIntegerVec keeps integers and logicals", {
  expect_identical(as_int_vec(c(1L, NA, -3L, .Machine$integer.max)), c(1L, NA, -3L, .Machine$integer.max))
  expect_identical(as_int_vec(c(TRUE, NA, FALSE)), c(1L, NA, 0L))
  expect_identical(as_int_vec(integer(0)), integer(0))
})

test_that("AsIntegerVec stores whole doubles, and NaN as NA as as.integer() does", {
  x <- c(3, -0, NA, NaN, 2^31 - 1, -(2^31 - 1))
  expect_identical(as_int_vec(x), as.integer(x))
})

test_that("AsIntegerVec maps NA of every type to NA_integer_", {
  for (na in list(NA_real_, NA_integer_, NA, NA_character_, factor(NA))) {
    expect_identical(as_int_vec(na), NA_integer_)
  }
})

# endregion

# region: character parses as as.numeric() does

test_that("AsIntegerVec agrees with as.integer() on every whole number it accepts", {
  ok <- c(" 15 ", "1e3", "0x1A", "0X1a", "+3", "5.", "-0", "\t2\n", "", "   ", NA)
  expect_no_warning(reference <- as.integer(ok))
  expect_identical(as_int_vec(ok), reference)
})

test_that("AsIntegerVec reads the NA token and NaN text as NA", {
  # as.integer("NA") and as.integer("NaN") give NA with a coercion warning;
  # the marker reads both as missing, as AsNumericVec does.
  expect_identical(as_int_vec(c("NA", " NA ", "NaN", "7")), c(NA, NA, NA, 7L))
})

# endregion

# region: factors are read by their labels

test_that("AsIntegerVec reads factor labels, not codes", {
  # Codes are c(1L, 2L); the labels are 10 and 2.
  expect_identical(as_int_vec(factor(c("10", "2"))), c(10L, 2L))
  expect_identical(as_int_vec(factor(c("5", NA, "", "5"))), c(5L, NA, NA, 5L))
})

test_that("AsIntegerVec names a refused factor label at every element using it", {
  expect_error(
    as_int_vec(factor(c("2", "2.5", "2", "2.5"))),
    'non-integer value(s): "2.5", "2.5" (elements 2, 4)',
    fixed = TRUE
  )
})

# endregion

# region: refusals are collected

test_that("AsIntegerVec refuses fractions and out-of-range values that as.integer() would hide", {
  # as.integer() truncates 3.5 and gives NA (with a warning) out of range;
  # -2^31 is NA_integer_'s bit pattern, so it is out of range too.
  expect_error(as_int_vec(3.5), 'non-integer value(s): "3.5" (element 1)', fixed = TRUE)
  expect_error(as_int_vec("3.5"), 'non-integer value(s): "3.5" (element 1)', fixed = TRUE)
  for (x in list(Inf, -Inf, 2^31, -2^31, "-Inf", "1e10", "-2147483648")) {
    expect_error(as_int_vec(x), "non-integer value(s): ", fixed = TRUE)
  }
})

test_that("AsIntegerVec reports every kind of refusal in one message", {
  expect_identical(
    tryCatch(as_int_vec(c("1", "n/a", "3.5", "1e10", "-Inf")), error = conditionMessage),
    paste0(
      "'x' must be integer: ",
      'non-integer value(s): "n/a", "3.5", "1e10", "-Inf" (elements 2, 3, 4, 5)'
    )
  )
})

test_that("AsIntegerVec shows a refused double as as.character() does", {
  x <- c(1, 2.5, 3e10, 0.1 + 0.2, -2^31)
  shown <- paste0('"', as.character(x[-1]), '"', collapse = ", ")
  expect_error(
    as_int_vec(x),
    paste0("non-integer value(s): ", shown, " (elements 2, 3, 4, 5)"),
    fixed = TRUE
  )
})

test_that("AsIntegerVec lists at most 10 values, then counts the rest", {
  msg <- tryCatch(as_int_vec(c(seq_len(12) + 0.5, 7)), error = conditionMessage)
  expect_match(msg, 'non-integer value(s): "1.5", "2.5",', fixed = TRUE)
  expect_match(
    msg,
    '"10.5" (elements 1, 2, 3, 4, 5, 6, 7, 8, 9, 10); and 2 more',
    fixed = TRUE
  )
})

test_that("AsIntegerVec refuses other types at the R boundary", {
  msg <- "'x' must be numeric, logical, character, or factor"
  expect_error(as_int_vec(list(1)), msg, fixed = TRUE)
  expect_error(as_int_vec(as.raw(1)), msg, fixed = TRUE)
  expect_error(as_int_vec(1i), msg, fixed = TRUE)
})

# endregion

# region: no_na

test_that("no_na refuses what the marker reads as missing", {
  expect_identical(as_int_no_na(c("1", " 2 ")), c(1L, 2L))
  # anyNA() sees none of these, so the refusal comes from the check after the
  # conversion; NA and NaN are refused by the R guard first.
  for (x in list(c("1", "NA"), "NaN", "", factor(c("3", "")), NA, NaN)) {
    expect_error(as_int_no_na(x), "'x' must not contain NA", fixed = TRUE)
  }
})

# endregion

# region: scalar AsInteger and Option<AsInteger>

test_that("AsInteger reads one value of any accepted type", {
  expect_identical(as_int(2), 2L)
  expect_identical(as_int(" 1e3 "), 1000L)
  expect_identical(as_int(factor("10")), 10L)
  expect_identical(as_int(NA_character_), NA_integer_)
  expect_identical(as_int(TRUE), 1L)
})

test_that("AsInteger requires length 1 and names a refused value", {
  expect_error(as_int(c(1, 2)), "'x' must have length 1", fixed = TRUE)
  expect_error(as_int(integer(0)), "'x' must have length 1", fixed = TRUE)
  expect_identical(
    tryCatch(as_int(3.5), error = conditionMessage),
    "'x' must be a single whole number: non-integer value(s): \"3.5\" (element 1)"
  )
  expect_error(
    as_int(list(1)),
    "'x' must be numeric, logical, character, or factor",
    fixed = TRUE
  )
})

test_that("Option<AsInteger> takes NULL and keeps the marker's rules", {
  expect_identical(as_int_opt(NULL), NA_integer_)
  expect_identical(as_int_opt("4"), 4L)
  expect_error(as_int_opt(4.5), 'non-integer value(s): "4.5" (element 1)', fixed = TRUE)
  expect_error(
    as_int_opt(list(1)),
    "'x' must be NULL or numeric, logical, character, or factor",
    fixed = TRUE
  )
})

# endregion
