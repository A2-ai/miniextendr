# Tests for AsCharacter / AsCharacterVec: an argument read the way
# as.character() reads it, from any atomic vector, a factor by its labels,
# with NA of any type as NA.
#
# The conversion's own text ("as.character() failed", ...) is matched exactly;
# the "'x' must be coercible to character: " prefix before it is the macro's
# argument-error wording (#1591). R precondition failures are matched on stable fragments (the
# parameter name and the requirement), not on the full sentence the generated
# check happens to use. Both raise the same argument error: `rust_error`,
# `kind = "conversion"` and `e$param`, plus `e$rust_type` on a conversion.

as_chr_vec <- function(x) miniextendr:::test_as_character_vec(x)
as_chr <- function(x) miniextendr:::test_as_character(x)
as_chr_opt <- function(x) miniextendr:::test_as_character_opt(x)
n_distinct <- function(x) miniextendr:::test_as_character_n_distinct(x)
as_chr_no_na <- function(x) miniextendr:::test_as_character_no_na(x)

# region: the strings are as.character()'s

test_that("AsCharacterVec gives exactly as.character() for every atomic type", {
  inputs <- list(
    c("a", NA, "NA", ""),
    c(1L, NA, -3L),
    c(0.1 + 0.2, 1e6, 100, 1e5, 1 / 3, 2^53, NaN, Inf, -Inf, NA),
    c(1, NA) * 2,
    c(TRUE, NA, FALSE),
    c(1 + 2i, NA),
    as.raw(c(0, 255)),
    1:5,
    factor(c("b", "a", NA, "b")),
    factor(c("3", "1"), ordered = TRUE),
    as.Date(c("2024-01-15", NA)),
    as.POSIXct(c("2024-01-15 10:30:00", "2024-01-16", NA), tz = "UTC"),
    character(0),
    integer(0),
    double(0),
    factor(character(0))
  )
  for (x in inputs) {
    expect_identical(
      as_chr_vec(x), as.character(x),
      info = paste(deparse(x), collapse = " ")
    )
  }
})

test_that("AsCharacterVec formats doubles as R does, not as Rust does", {
  # format!("{}", 0.1 + 0.2) would give "0.30000000000000004".
  expect_identical(as_chr_vec(0.1 + 0.2), "0.3")
  expect_identical(as_chr_vec(c(1e6, 100, 1e5)), c("1e+06", "100", "1e+05"))
  expect_identical(as_chr_vec(c(NaN, Inf, NA)), c("NaN", "Inf", NA))
})

test_that("AsCharacterVec spells logical, complex and raw as as.character() does", {
  expect_identical(as_chr_vec(c(TRUE, FALSE, NA)), c("TRUE", "FALSE", NA))
  expect_identical(as_chr_vec(1 + 2i), "1+2i")
  expect_identical(as_chr_vec(as.raw(255)), "ff")
})

test_that("AsCharacterVec keeps the strings \"NA\" and \"\" as values", {
  expect_identical(as_chr_vec(c("NA", "", NA)), c("NA", "", NA))
})

# endregion

# region: factors and other classed vectors

test_that("AsCharacterVec reads factor labels, not codes", {
  # Codes are c(1L, 2L, 1L); the labels are "10" and "2".
  expect_identical(as_chr_vec(factor(c("10", "2", "10"))), c("10", "2", "10"))
  expect_identical(as_chr_vec(factor(c("S1", NA))), c("S1", NA))
})

test_that("AsCharacterVec formats Date and POSIXct through as.character()", {
  # The double underneath a Date is a day count: as.vector() would give "19737".
  expect_identical(as_chr_vec(as.Date("2024-01-15")), "2024-01-15")
  expect_identical(
    as_chr_vec(as.POSIXct("2024-01-15 10:30:00", tz = "UTC")),
    "2024-01-15 10:30:00"
  )
})

test_that("AsCharacterVec uses a registered as.character() method", {
  .S3method("as.character", "mx_test_label", function(x, ...) paste0("L", unclass(x)))
  expect_identical(
    as_chr_vec(structure(1:2, class = "mx_test_label")),
    c("L1", "L2")
  )
})

test_that("AsCharacterVec finds a method defined in the global environment", {
  # Unregistered, as a script would define it; a top-level as.character(x)
  # finds it, and so does the marker.
  assign(
    "as.character.mx_test_global",
    function(x, ...) rep("G", length(x)),
    envir = globalenv()
  )
  on.exit(rm("as.character.mx_test_global", envir = globalenv()), add = TRUE)
  expect_identical(
    as_chr_vec(structure(1:2, class = "mx_test_global")),
    c("G", "G")
  )
})

test_that("A global binding named as.character does not replace base's", {
  assign("as.character", function(...) "masked", envir = globalenv())
  on.exit(rm("as.character", envir = globalenv()), add = TRUE)
  expect_identical(as_chr_vec(factor(c("a", "b"))), c("a", "b"))
})

test_that("A failing as.character() method is a conversion error", {
  .S3method("as.character", "mx_test_boom", function(x, ...) stop("no labels here"))
  expect_error(
    as_chr_vec(structure(1L, class = "mx_test_boom")),
    "as.character() failed: ",
    fixed = TRUE
  )
  e <- tryCatch(as_chr_vec(structure(1L, class = "mx_test_boom")), error = identity)
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "x")
  expect_identical(e$rust_type, "AsCharacterVec")
  expect_match(
    conditionMessage(e),
    "^'x' must be coercible to character: as\\.character\\(\\) failed: "
  )
  expect_match(conditionMessage(e), "no labels here", fixed = TRUE)
  .S3method("as.character", "mx_test_numeric", function(x, ...) unclass(x))
  expect_error(
    as_chr_vec(structure(1L, class = "mx_test_numeric")),
    "as.character() returned INTSXP, not a character vector",
    fixed = TRUE
  )
  # A length-1 argument whose method gives two strings passes the R-side
  # checks; the conversion names what the result has to be.
  .S3method("as.character", "mx_test_pair", function(x, ...) c("a", "b"))
  e <- tryCatch(as_chr(structure(1L, class = "mx_test_pair")), error = identity)
  expect_identical(conditionMessage(e), "'x' must be coercible to a single string: got length 2")
  expect_identical(e$rust_type, "AsCharacter")
})

# endregion

# region: shape

test_that("AsCharacterVec drops names and dim, as as.character() does", {
  expect_identical(as_chr_vec(c(a = 1L, b = 2L)), c("1", "2"))
  m <- matrix(1:4, 2)
  expect_identical(as_chr_vec(m), as.character(m))
  a <- array(c(1.5, 2), dim = 2L)
  expect_identical(as_chr_vec(a), c("1.5", "2"))
})

test_that("AsCharacterVec refuses lists and data frames at the R boundary", {
  msg <- "'x'.*atomic"
  expect_error(as_chr_vec(list(1)), msg)
  expect_error(as_chr_vec(data.frame(id = 1:2)), msg)
  expect_error(as_chr_vec(NULL), msg)
  expect_error(as_chr_vec(as.POSIXlt("2024-01-15")), msg)
  e <- tryCatch(as_chr_vec(list(1)), error = identity)
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "x")
  expect_null(e$rust_type)
})

# endregion

# region: NA handling

test_that("AsCharacterVec maps NA of every type to NA_character_", {
  for (na in list(NA, NA_integer_, NA_real_, NA_complex_, NA_character_, factor(NA))) {
    expect_identical(as_chr_vec(na), NA_character_)
  }
})

test_that("no_na refuses NA in R, and NaN with it", {
  expect_identical(as_chr_no_na(c(1L, 2L)), c("1", "2"))
  expect_identical(as_chr_no_na(factor("a")), "a")
  msg <- "'x'.*not.*NA"
  expect_error(as_chr_no_na(c(1L, NA)), msg)
  # A vector marker "contains" an NA; the check raises the argument error.
  e <- tryCatch(as_chr_no_na(c(1L, NA)), error = identity)
  expect_identical(conditionMessage(e), "'x' must not contain NA")
  expect_s3_class(e, "rust_error")
  expect_identical(e$param, "x")
  expect_error(as_chr_no_na(factor(c("a", NA))), msg)
  expect_error(as_chr_no_na(c(1, NaN)), msg)
  # The string "NA" is a value, not a missing value.
  expect_identical(as_chr_no_na("NA"), "NA")
})

# endregion

# region: identifiers as labels

test_that("AsCharacterVec counts identifiers whatever their storage", {
  expect_identical(n_distinct(c(101L, 102L, 101L)), 2L)
  expect_identical(n_distinct(c(101, 102, 101)), 2L)
  expect_identical(n_distinct(factor(c("S1", "S2", NA, "S1"))), 2L)
  expect_identical(n_distinct(c("S1", "S2", "S1")), 2L)
  # Both read as "0.3".
  expect_identical(n_distinct(c(0.1 + 0.2, 0.3)), 1L)
})

# endregion

# region: scalar AsCharacter and Option<AsCharacter>

test_that("AsCharacter reads one value of any atomic type", {
  expect_identical(as_chr("S-01"), "S-01")
  expect_identical(as_chr(101L), "101")
  expect_identical(as_chr(0.1 + 0.2), "0.3")
  expect_identical(as_chr(TRUE), "TRUE")
  expect_identical(as_chr(factor("S-02")), "S-02")
  expect_identical(as_chr(as.Date("2024-01-15")), "2024-01-15")
  expect_identical(as_chr(NA), NA_character_)
})

test_that("AsCharacter requires length 1", {
  expect_error(as_chr(c(1, 2)), "'x'.*length 1")
  expect_error(as_chr(character(0)), "'x'.*length 1")
  expect_error(as_chr(list("a")), "'x'.*atomic")
})

test_that("Option<AsCharacter> takes NULL and keeps the marker's rules", {
  expect_identical(as_chr_opt(NULL), NA_character_)
  expect_identical(as_chr_opt(7L), "7")
  expect_error(as_chr_opt(list(1)), "'x'.*NULL.*atomic")
  expect_error(as_chr_opt(1:2), "'x'.*NULL.*length 1")
})

# endregion

# region: GC stress fixture

test_that("gc_stress_as_character reads every input", {
  out <- miniextendr:::gc_stress_as_character()
  expect_identical(
    out,
    c(
      as.character(0:49 + 0.5),
      c("1", NA),
      as.character(as.Date("2024-01-15") + 0:9),
      c("b", NA, "a", "b")
    )
  )
})

# endregion
