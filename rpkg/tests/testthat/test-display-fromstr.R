# Tests for AsDisplay / AsFromStr conversion wrappers

# region: AsDisplay — Rust T: Display → R character

test_that("AsDisplay converts IpAddr to character", {
  result <- test_display_ip()
  expect_type(result, "character")
  expect_length(result, 1)
  expect_equal(result, "127.0.0.1")
})

test_that("AsDisplay converts f64 to character", {
  result <- test_display_number()
  expect_type(result, "character")
  expect_equal(result, "3.141592653589793")
})

test_that("AsDisplay converts bool to character", {
  result <- test_display_bool()
  expect_type(result, "character")
  expect_equal(result, "true")
})

# endregion

# region: AsDisplayVec — Rust Vec<T: Display> → R character vector

test_that("AsDisplayVec converts Vec<IpAddr> to character vector", {
  result <- test_display_vec_ips()
  expect_type(result, "character")
  expect_length(result, 3)
  expect_equal(result, c("127.0.0.1", "::1", "192.168.1.1"))
})

test_that("AsDisplayVec converts Vec<i32> to character vector", {
  result <- test_display_vec_ints()
  expect_type(result, "character")
  expect_equal(result, c("1", "2", "3", "42"))
})

# endregion

# region: AsFromStr — R character → Rust T: FromStr

test_that("AsFromStr parses IP address", {
  expect_true(test_fromstr_ip("127.0.0.1"))
  expect_false(test_fromstr_ip("192.168.1.1"))
})

test_that("AsFromStr parses integer", {
  expect_equal(test_fromstr_int("42"), 42)
  expect_equal(test_fromstr_int("-100"), -100)
})

test_that("AsFromStr returns error on invalid input", {
  expect_error(test_fromstr_bad_input("not-an-ip"))
})

# endregion

# region: AsFromStrVec — R character vector → Rust Vec<T: FromStr>

test_that("AsFromStrVec parses IP addresses", {
  result <- test_fromstr_vec_ips(c("127.0.0.1", "192.168.1.1"))
  expect_equal(result, c(TRUE, FALSE))
})

test_that("AsFromStrVec parses integers", {
  result <- test_fromstr_vec_ints(c("1", "2", "3"))
  expect_equal(result, c(1L, 2L, 3L))
})

test_that("AsFromStrVec reports all parse errors", {
  expect_error(
    test_fromstr_vec_ints(c("1", "abc", "3", "def")),
    "index 1.*index 3"
  )
})

# endregion
