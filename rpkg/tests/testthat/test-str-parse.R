# try_from_sexp_via_str_parse! on a type defined in the package (#1766):
# `Version` converts as `T`, `Option<T>`, `Vec<T>` and `Vec<Option<T>>`, with
# the NA policy and batched element errors of the built-in uuid / url / regex /
# num-bigint parsers.

test_that("the scalar parses and refuses NA", {
  expect_equal(miniextendr:::str_parse_version("1.02.3"), "1.2.3")
  expect_error(
    miniextendr:::str_parse_version("1.2"),
    "invalid version: expected major.minor.patch",
    fixed = TRUE
  )
  expect_error(miniextendr:::str_parse_version(NA_character_), "NA", fixed = TRUE)
})

test_that("Option reads NA and NULL as None", {
  expect_equal(miniextendr:::str_parse_version_opt("0.1.0"), "0.1.0")
  expect_identical(miniextendr:::str_parse_version_opt(NA_character_), NA_character_)
  expect_identical(miniextendr:::str_parse_version_opt(NULL), NA_character_)
  expect_error(
    miniextendr:::str_parse_version_opt("x"),
    "invalid version: expected major.minor.patch",
    fixed = TRUE
  )
})

test_that("Vec batches NA and parse failures with 1-based positions", {
  expect_equal(
    miniextendr:::str_parse_version_vec(c("1.0.0", "2.10.01")),
    c("1.0.0", "2.10.1")
  )
  msg <- tryCatch(
    miniextendr:::str_parse_version_vec(c("1.0.0", "bad", NA, "1.2", "3.0.0")),
    error = conditionMessage
  )
  expect_match(msg, "invalid version: expected major.minor.patch (elements 2, 4)", fixed = TRUE)
  expect_match(msg, "NA is not allowed (element 3)", fixed = TRUE)
})

test_that("Vec lists the first ten failures and counts the rest", {
  msg <- tryCatch(
    miniextendr:::str_parse_version_vec(rep("bad", 13)),
    error = conditionMessage
  )
  expect_match(
    msg,
    "invalid version: expected major.minor.patch (elements 1, 2, 3, 4, 5, 6, 7, 8, 9, 10); and 3 more",
    fixed = TRUE
  )
})

test_that("Vec<Option> keeps NA and batches parse failures", {
  expect_identical(
    miniextendr:::str_parse_version_vec_opt(c("1.0.0", NA, "0.0.1")),
    c("1.0.0", NA, "0.0.1")
  )
  msg <- tryCatch(
    miniextendr:::str_parse_version_vec_opt(c(NA, "bad", "1.0.0", "worse")),
    error = conditionMessage
  )
  expect_match(msg, "invalid version: expected major.minor.patch (elements 2, 4)", fixed = TRUE)
  expect_no_match(msg, "NA is not allowed", fixed = TRUE)
})
