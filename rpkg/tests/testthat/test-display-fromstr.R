# Tests for AsDisplay / AsFromStr conversion wrappers

# region: AsDisplay — Rust T: Display → R character

test_that("AsDisplay converts IpAddr to character", {
  result <- miniextendr:::test_display_ip()
  expect_type(result, "character")
  expect_length(result, 1)
  expect_equal(result, "127.0.0.1")
})

test_that("AsDisplay converts f64 to character", {
  result <- miniextendr:::test_display_number()
  expect_type(result, "character")
  expect_equal(result, "3.141592653589793")
})

test_that("AsDisplay converts bool to character", {
  result <- miniextendr:::test_display_bool()
  expect_type(result, "character")
  expect_equal(result, "true")
})

# endregion

# region: AsDisplayVec — Rust Vec<T: Display> → R character vector

test_that("AsDisplayVec converts Vec<IpAddr> to character vector", {
  result <- miniextendr:::test_display_vec_ips()
  expect_type(result, "character")
  expect_length(result, 3)
  expect_equal(result, c("127.0.0.1", "::1", "192.168.1.1"))
})

test_that("AsDisplayVec converts Vec<i32> to character vector", {
  result <- miniextendr:::test_display_vec_ints()
  expect_type(result, "character")
  expect_equal(result, c("1", "2", "3", "42"))
})

# endregion

# region: AsFromStr — R character → Rust T: FromStr

test_that("AsFromStr parses IP address", {
  expect_true(miniextendr:::test_fromstr_ip("127.0.0.1"))
  expect_false(miniextendr:::test_fromstr_ip("192.168.1.1"))
})

test_that("AsFromStr parses integer", {
  expect_equal(miniextendr:::test_fromstr_int("42"), 42)
  expect_equal(miniextendr:::test_fromstr_int("-100"), -100)
})

test_that("AsFromStr returns error on invalid input", {
  expect_error(miniextendr:::test_fromstr_bad_input("not-an-ip"))
})

test_that("AsFromStr quotes the value it cannot parse", {
  expect_error(
    miniextendr:::test_fromstr_int("n/a"),
    '"n/a": invalid digit found in string',
    fixed = TRUE
  )
})

test_that("AsFromStr refuses NA instead of parsing an empty string", {
  expect_error(
    miniextendr:::test_fromstr_int(NA_character_),
    "unexpected NA value",
    fixed = TRUE
  )
})

# endregion

# region: AsFromStrVec — R character vector → Rust Vec<T: FromStr>

test_that("AsFromStrVec parses IP addresses", {
  result <- miniextendr:::test_fromstr_vec_ips(c("127.0.0.1", "192.168.1.1"))
  expect_equal(result, c(TRUE, FALSE))
})

test_that("AsFromStrVec parses integers", {
  result <- miniextendr:::test_fromstr_vec_ints(c("1", "2", "3"))
  expect_equal(result, c(1L, 2L, 3L))
})

test_that("AsFromStrVec reports all parse errors", {
  expect_error(
    miniextendr:::test_fromstr_vec_ints(c("1", "abc", "3", "def")),
    "index 1.*index 3"
  )
})

test_that("AsFromStrVec quotes each value it cannot parse", {
  expect_error(
    miniextendr:::test_fromstr_vec_ints(c("1", "n/a")),
    'index 1: "n/a": invalid digit found in string',
    fixed = TRUE
  )
})

test_that("AsFromStrVec reports NA as NA, not as a failed parse of \"\"", {
  msg <- tryCatch(
    miniextendr:::test_fromstr_vec_ints(c("1", NA, "x")),
    error = conditionMessage
  )
  expect_match(msg, "NA at index 1 not allowed", fixed = TRUE)
  expect_match(msg, 'index 2: "x": invalid digit found in string', fixed = TRUE)
  expect_no_match(msg, "empty string", fixed = TRUE)
})

# endregion
