# Tests for feature-specific type conversions
#
# These tests verify TryFromSexp/IntoR implementations for optional feature types.
# Each feature is skipped if not enabled using skip_if_missing_feature().

# =============================================================================
# UUID feature tests
# =============================================================================

test_that("uuid_roundtrip preserves UUID", {
  skip_if_missing_feature("uuid")
  original <- "550e8400-e29b-41d4-a716-446655440000"
  result <- uuid_roundtrip(original)
  expect_equal(result, original)
})

test_that("uuid_roundtrip_vec preserves vector of UUIDs", {
  skip_if_missing_feature("uuid")
  uuids <- c(
    "550e8400-e29b-41d4-a716-446655440000",
    "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
  )
  result <- uuid_roundtrip_vec(uuids)
  expect_equal(result, uuids)
})

test_that("uuid_new_v4 generates valid v4 UUID", {
  skip_if_missing_feature("uuid")
  uuid <- uuid_new_v4()
  expect_type(uuid, "character")
  expect_equal(nchar(uuid), 36L)
  expect_equal(uuid_version(uuid), 4L)
})

test_that("uuid_nil returns nil UUID", {
  skip_if_missing_feature("uuid")
  nil <- uuid_nil()
  expect_equal(nil, "00000000-0000-0000-0000-000000000000")
  expect_true(uuid_is_nil(nil))
})

test_that("uuid_max returns max UUID", {
  skip_if_missing_feature("uuid")
  max <- uuid_max()
  expect_equal(max, "ffffffff-ffff-ffff-ffff-ffffffffffff")
  expect_false(uuid_is_nil(max))
})

# =============================================================================
# Regex feature tests
# =============================================================================

test_that("regex_is_match works", {
  skip_if_missing_feature("regex")
  expect_true(regex_is_match("\\d+", "test123"))
  expect_false(regex_is_match("\\d+", "no digits"))
})

test_that("regex_find returns first match", {
  skip_if_missing_feature("regex")
  expect_equal(regex_find("\\d+", "test123abc456"), "123")
  # None becomes NA (not NULL) in R
  expect_true(is.na(regex_find("\\d+", "no digits")))
})

test_that("regex_find_all returns all matches", {
  skip_if_missing_feature("regex")
  expect_equal(regex_find_all("\\d+", "a1b2c3"), c("1", "2", "3"))
  expect_equal(regex_find_all("[a-z]+", "Hello World"), c("ello", "orld"))
})

test_that("regex_replace_first replaces first match", {
  skip_if_missing_feature("regex")
  expect_equal(regex_replace_first("\\d+", "abc123def456", "X"), "abcXdef456")
})

test_that("regex_replace_all replaces all matches", {
  skip_if_missing_feature("regex")
  expect_equal(regex_replace_all("\\d+", "abc123def456", "X"), "abcXdefX")
})

test_that("regex_split splits by pattern", {
  skip_if_missing_feature("regex")
  expect_equal(regex_split("\\s+", "hello world  test"), c("hello", "world", "test"))
})

# =============================================================================
# Time feature tests
# =============================================================================

test_that("time_roundtrip_posixct preserves POSIXct", {
  skip_if_missing_feature("time")
  # Use a fixed timestamp to avoid platform-specific precision issues
  # 2024-06-15 12:30:45.123 UTC
  fixed_time <- as.POSIXct("2024-06-15 12:30:45", tz = "UTC")
  result <- time_roundtrip_posixct(fixed_time)
  # Allow for second-level precision (platform differences in sub-second handling)
  expect_true(abs(as.numeric(result) - as.numeric(fixed_time)) < 1)
})

test_that("time_roundtrip_date preserves Date", {
  skip_if_missing_feature("time")
  today <- Sys.Date()
  result <- time_roundtrip_date(today)
  expect_equal(result, today)
})

test_that("time_get_year extracts year", {
  skip_if_missing_feature("time")
  date <- as.Date("2024-06-15")
  expect_equal(time_get_year(date), 2024L)
})

test_that("time_get_month extracts month", {
  skip_if_missing_feature("time")
  date <- as.Date("2024-06-15")
  expect_equal(time_get_month(date), 6L)
})

test_that("time_get_day extracts day", {
  skip_if_missing_feature("time")
  date <- as.Date("2024-06-15")
  expect_equal(time_get_day(date), 15L)
})

# =============================================================================
# OrderedFloat feature tests
# =============================================================================

test_that("ordered_float_roundtrip preserves value", {
  skip_if_missing_feature("ordered-float")
  expect_equal(ordered_float_roundtrip(3.14), 3.14)
})

test_that("ordered_float_roundtrip_vec preserves vector", {
  skip_if_missing_feature("ordered-float")
  x <- c(1.0, 2.0, 3.0)
  expect_equal(ordered_float_roundtrip_vec(x), x)
})

test_that("ordered_float_sort handles NaN correctly", {
  skip_if_missing_feature("ordered-float")
  # In OrderedFloat's total ordering, NaN is greater than all values
  result <- ordered_float_sort(c(NaN, 3.0, 1.0, 5.0, NaN))
  # First 3 should be sorted normally
  expect_equal(result[1:3], c(1.0, 3.0, 5.0))
  # NaN should come last
  expect_true(is.nan(result[4]))
  expect_true(is.nan(result[5]))
})

test_that("ordered_float_is_nan detects NaN", {
  skip_if_missing_feature("ordered-float")
  expect_true(ordered_float_is_nan(NaN))
  expect_false(ordered_float_is_nan(3.14))
})

test_that("ordered_float_is_finite works", {
  skip_if_missing_feature("ordered-float")
  expect_true(ordered_float_is_finite(3.14))
  expect_false(ordered_float_is_finite(Inf))
  expect_false(ordered_float_is_finite(NaN))
})

# =============================================================================
# num-bigint feature tests
# =============================================================================

test_that("bigint_roundtrip preserves value", {
  skip_if_missing_feature("num-bigint")
  result <- bigint_roundtrip("12345678901234567890")
  expect_equal(result, "12345678901234567890")
})

test_that("bigint_add adds large integers", {
  skip_if_missing_feature("num-bigint")
  result <- bigint_add("1000000000000000000", "1")
  expect_equal(result, "1000000000000000001")
})

test_that("bigint_mul multiplies large integers", {
  skip_if_missing_feature("num-bigint")
  result <- bigint_mul("1000000000", "1000000000")
  expect_equal(result, "1000000000000000000")
})

test_that("bigint_factorial computes large factorials", {
  skip_if_missing_feature("num-bigint")
  expect_equal(bigint_factorial(20L), "2432902008176640000")
  # 100! is huge
  fact100 <- bigint_factorial(100L)
  expect_true(nchar(fact100) > 100)  # 100! has 158 digits
})

test_that("bigint_is_positive works", {
  skip_if_missing_feature("num-bigint")
  expect_true(bigint_is_positive("123"))
  expect_false(bigint_is_positive("-456"))
  expect_false(bigint_is_positive("0"))
})

# =============================================================================
# rust_decimal feature tests
# =============================================================================

test_that("decimal_roundtrip preserves value", {
  skip_if_missing_feature("rust_decimal")
  result <- decimal_roundtrip("123.456")
  expect_equal(result, "123.456")
})

test_that("decimal_add adds decimals precisely", {
  skip_if_missing_feature("rust_decimal")
  # This would have floating point errors with double
  result <- decimal_add("0.1", "0.2")
  expect_equal(result, "0.3")
})

test_that("decimal_mul multiplies decimals", {
  skip_if_missing_feature("rust_decimal")
  result <- decimal_mul("100.5", "2")
  expect_equal(result, "201.0")
})

test_that("decimal_round rounds to decimal places", {
  skip_if_missing_feature("rust_decimal")
  expect_equal(decimal_round("123.456", 2L), "123.46")
  expect_equal(decimal_round("123.456", 0L), "123")
})

test_that("decimal_scale returns decimal places", {
  skip_if_missing_feature("rust_decimal")
  expect_equal(decimal_scale("123.456"), 3L)
  expect_equal(decimal_scale("100"), 0L)
})

test_that("decimal_is_zero works", {
  skip_if_missing_feature("rust_decimal")
  expect_true(decimal_is_zero("0"))
  expect_true(decimal_is_zero("0.00"))
  expect_false(decimal_is_zero("0.001"))
})

# =============================================================================
# indexmap feature tests
# =============================================================================

test_that("indexmap_roundtrip_int preserves order", {
  skip_if_missing_feature("indexmap")
  input <- list(z = 1L, a = 2L, m = 3L)
  result <- indexmap_roundtrip_int(input)
  expect_equal(names(result), c("z", "a", "m"))
  expect_equal(result$z, 1L)
  expect_equal(result$a, 2L)
  expect_equal(result$m, 3L)
})

test_that("indexmap_roundtrip_str preserves strings", {
  skip_if_missing_feature("indexmap")
  input <- list(foo = "bar", baz = "qux")
  result <- indexmap_roundtrip_str(input)
  expect_equal(result$foo, "bar")
  expect_equal(result$baz, "qux")
})

test_that("indexmap_roundtrip_dbl preserves doubles", {
  skip_if_missing_feature("indexmap")
  input <- list(pi = 3.14, e = 2.71)
  result <- indexmap_roundtrip_dbl(input)
  expect_equal(result$pi, 3.14)
  expect_equal(result$e, 2.71)
})

test_that("indexmap_keys returns keys in insertion order", {
  skip_if_missing_feature("indexmap")
  input <- list(third = 3L, first = 1L, second = 2L)
  keys <- indexmap_keys(input)
  expect_equal(keys, c("third", "first", "second"))
})

test_that("indexmap_len returns correct length", {
  skip_if_missing_feature("indexmap")
  expect_equal(indexmap_len(list(a = 1L, b = 2L, c = 3L)), 3L)
  expect_equal(indexmap_len(list()), 0L)
})
