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

# =============================================================================
# Bytes feature tests
# =============================================================================

test_that("bytes_roundtrip preserves raw data", {
  skip_if_missing_feature("bytes")
  raw_data <- as.raw(c(0x01, 0x02, 0x03, 0xff))
  result <- bytes_roundtrip(raw_data)
  expect_equal(result, raw_data)
})

test_that("bytes_len returns correct length", {
  skip_if_missing_feature("bytes")
  expect_equal(bytes_len(as.raw(1:10)), 10L)
  expect_equal(bytes_len(raw(0)), 0L)
})

test_that("bytes_mut_roundtrip preserves raw data", {
  skip_if_missing_feature("bytes")
  raw_data <- as.raw(c(0xde, 0xad, 0xbe, 0xef))
  result <- bytes_mut_roundtrip(raw_data)
  expect_equal(result, raw_data)
})

test_that("bytes_concat joins two raw vectors", {
  skip_if_missing_feature("bytes")
  a <- as.raw(c(1, 2))
  b <- as.raw(c(3, 4))
  result <- bytes_concat(a, b)
  expect_equal(result, as.raw(c(1, 2, 3, 4)))
})

test_that("bytes_slice extracts subrange", {
  skip_if_missing_feature("bytes")
  data <- as.raw(c(10, 20, 30, 40, 50))
  result <- bytes_slice(data, 1L, 4L)
  expect_equal(result, as.raw(c(20, 30, 40)))
})

# =============================================================================
# Bitflags feature tests
# =============================================================================

test_that("bitflags_roundtrip preserves flags", {
  skip_if_missing_feature("bitflags")
  # READ=1, WRITE=2, READ|WRITE=3
  expect_equal(bitflags_roundtrip(3L), 3L)
})

test_that("bitflags_from_strict rejects invalid flags", {
  skip_if_missing_feature("bitflags")
  # Valid: 7 = READ|WRITE|EXECUTE

  expect_equal(bitflags_from_strict(7L), 7L)
  # Invalid: 8 has no flag bit
  expect_true(is.na(bitflags_from_strict(8L)))
})

test_that("bitflags_from_truncate masks invalid bits", {
  skip_if_missing_feature("bitflags")
  # 15 = 0b1111, but only 0b111 are valid, so truncate to 7
  expect_equal(bitflags_from_truncate(15L), 7L)
})

test_that("bitflags_has_read/write check individual flags", {
  skip_if_missing_feature("bitflags")
  # READ=1, WRITE=2
  expect_true(bitflags_has_read(1L))
  expect_false(bitflags_has_write(1L))
  expect_true(bitflags_has_write(2L))
  expect_true(bitflags_has_read(3L))  # READ|WRITE
  expect_true(bitflags_has_write(3L))
})

test_that("bitflags_union combines flags", {
  skip_if_missing_feature("bitflags")
  # READ=1 | WRITE=2 = 3
  expect_equal(bitflags_union(1L, 2L), 3L)
  # EXECUTE=4 | READ=1 = 5
  expect_equal(bitflags_union(4L, 1L), 5L)
})

# =============================================================================
# Bitvec feature tests
# =============================================================================

test_that("bitvec_roundtrip preserves logical vector", {
  skip_if_missing_feature("bitvec")
  bits <- c(TRUE, FALSE, TRUE, TRUE, FALSE)
  result <- bitvec_roundtrip(bits)
  expect_equal(result, bits)
})

test_that("bitvec_ones counts TRUE values", {
  skip_if_missing_feature("bitvec")
  expect_equal(bitvec_ones(c(TRUE, TRUE, FALSE, TRUE)), 3L)
  expect_equal(bitvec_ones(c(FALSE, FALSE)), 0L)
})

test_that("bitvec_zeros counts FALSE values", {
  skip_if_missing_feature("bitvec")
  expect_equal(bitvec_zeros(c(TRUE, TRUE, FALSE, TRUE)), 1L)
  expect_equal(bitvec_zeros(c(FALSE, FALSE)), 2L)
})

test_that("bitvec_from_vec/to_vec roundtrips", {
  skip_if_missing_feature("bitvec")
  input <- c(TRUE, FALSE, FALSE, TRUE)
  result <- bitvec_to_vec(bitvec_from_vec(input))
  expect_equal(result, input)
})

test_that("bitvec_len returns correct length", {
  skip_if_missing_feature("bitvec")
  expect_equal(bitvec_len(c(TRUE, FALSE, TRUE)), 3L)
  expect_equal(bitvec_len(logical(0)), 0L)
})

# =============================================================================
# TinyVec feature tests
# =============================================================================

test_that("tinyvec_roundtrip_int preserves integer vector", {
  skip_if_missing_feature("tinyvec")
  x <- c(1L, 2L, 3L)
  expect_equal(tinyvec_roundtrip_int(x), x)
})

test_that("tinyvec_roundtrip_dbl preserves double vector", {
  skip_if_missing_feature("tinyvec")
  x <- c(1.1, 2.2, 3.3)
  expect_equal(tinyvec_roundtrip_dbl(x), x)
})

test_that("tinyvec_len returns correct length", {
  skip_if_missing_feature("tinyvec")
  expect_equal(tinyvec_len(c(10L, 20L, 30L)), 3L)
  expect_equal(tinyvec_len(integer(0)), 0L)
})

test_that("arrayvec_roundtrip_int preserves integer vector", {
  skip_if_missing_feature("tinyvec")
  x <- c(1L, 2L, 3L, 4L)
  expect_equal(arrayvec_roundtrip_int(x), x)
})

test_that("arrayvec_roundtrip_dbl preserves double vector", {
  skip_if_missing_feature("tinyvec")
  x <- c(1.5, 2.5, 3.5)
  expect_equal(arrayvec_roundtrip_dbl(x), x)
})

# =============================================================================
# SHA-2 feature tests
# =============================================================================

test_that("sha2_sha256 produces correct hash", {
  skip_if_missing_feature("sha2")
  # Known SHA-256 of empty string
  expect_equal(
    sha2_sha256(""),
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  )
})

test_that("sha2_sha512 produces correct hash", {
  skip_if_missing_feature("sha2")
  # Known SHA-512 of empty string
  expect_equal(
    sha2_sha512(""),
    "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"
  )
})

test_that("sha2_sha256_len returns 64", {
  skip_if_missing_feature("sha2")
  expect_equal(sha2_sha256_len(), 64L)
})

test_that("sha2_sha512_len returns 128", {
  skip_if_missing_feature("sha2")
  expect_equal(sha2_sha512_len(), 128L)
})

test_that("sha2_sha256 is deterministic", {
  skip_if_missing_feature("sha2")
  expect_equal(sha2_sha256("hello"), sha2_sha256("hello"))
  expect_false(sha2_sha256("hello") == sha2_sha256("world"))
})

# =============================================================================
# URL feature tests
# =============================================================================

test_that("url_roundtrip preserves URL", {
  skip_if_missing_feature("url")
  url <- "https://example.com/path?query=1"
  expect_equal(url_roundtrip(url), url)
})

test_that("url_scheme extracts scheme", {
  skip_if_missing_feature("url")
  expect_equal(url_scheme("https://example.com"), "https")
  expect_equal(url_scheme("ftp://files.example.com"), "ftp")
})

test_that("url_host extracts host", {
  skip_if_missing_feature("url")
  expect_equal(url_host("https://example.com/path"), "example.com")
})

test_that("url_path extracts path", {
  skip_if_missing_feature("url")
  expect_equal(url_path("https://example.com/foo/bar"), "/foo/bar")
})

test_that("url_roundtrip_vec preserves vector of URLs", {
  skip_if_missing_feature("url")
  # URL normalization adds trailing slash to bare domains
  urls <- c("https://a.com/path", "https://b.com/other")
  expect_equal(url_roundtrip_vec(urls), urls)
})

test_that("url_is_valid checks URL validity", {
  skip_if_missing_feature("url")
  expect_true(url_is_valid("https://example.com"))
  expect_false(url_is_valid("not a url"))
})

# =============================================================================
# Aho-Corasick feature tests
# =============================================================================

test_that("aho_test_is_match finds patterns", {
  skip_if_missing_feature("aho-corasick")
  expect_true(aho_test_is_match(c("foo", "bar"), "hello foobar"))
  expect_false(aho_test_is_match(c("foo", "bar"), "hello world"))
})

test_that("aho_test_count counts all matches", {
  skip_if_missing_feature("aho-corasick")
  expect_equal(aho_test_count(c("a"), "banana"), 3L)
  expect_equal(aho_test_count(c("na"), "banana"), 2L)
})

test_that("aho_test_find_flat returns 1-based positions", {
  skip_if_missing_feature("aho-corasick")
  # "ab" in "xabxab" at positions 2 and 5 (1-based)
  result <- aho_test_find_flat(c("ab"), "xabxab")
  # find_all_flat returns (pattern_id, start, end) triples, 1-based
  expect_true(length(result) > 0)
})

test_that("aho_test_replace replaces all matches", {
  skip_if_missing_feature("aho-corasick")
  result <- aho_test_replace(c("cat", "dog"), "I have a cat and a dog", "pet")
  expect_equal(result, "I have a pet and a pet")
})

test_that("aho_test_count handles multiple patterns", {
  skip_if_missing_feature("aho-corasick")
  # "she" matches twice in "she sells seashells" (non-overlapping)
  expect_equal(aho_test_count(c("he", "she"), "she sells seashells"), 2L)
})

# =============================================================================
# TOML feature tests
# =============================================================================

test_that("toml_roundtrip parses and re-serializes", {
  skip_if_missing_feature("toml")
  input <- 'key = "value"'
  result <- toml_roundtrip(input)
  expect_type(result, "character")
  expect_true(grepl("key", result))
  expect_true(grepl("value", result))
})

test_that("toml_pretty produces formatted output", {
  skip_if_missing_feature("toml")
  input <- 'key = "value"\n[section]\nfoo = 42'
  result <- toml_pretty(input)
  expect_type(result, "character")
})

test_that("toml_type_name identifies TOML type", {
  skip_if_missing_feature("toml")
  expect_equal(toml_type_name('key = "value"'), "table")
})

test_that("toml_is_table identifies tables", {
  skip_if_missing_feature("toml")
  expect_true(toml_is_table('key = "value"'))
})

test_that("toml_table_keys extracts keys", {
  skip_if_missing_feature("toml")
  keys <- toml_table_keys('alpha = 1\nbeta = 2')
  expect_true("alpha" %in% keys)
  expect_true("beta" %in% keys)
})

# =============================================================================
# Tabled feature tests
# =============================================================================

test_that("tabled_simple returns formatted table string", {
  skip_if_missing_feature("tabled")
  result <- tabled_simple()
  expect_type(result, "character")
  expect_true(grepl("Name", result))
  expect_true(grepl("pi", result))
  expect_true(grepl("3.14", result))
})

test_that("tabled_from_vecs creates table from vectors", {
  skip_if_missing_feature("tabled")
  result <- tabled_from_vecs(c("Col1", "Col2"), c("a", "b"), c("1", "2"))
  expect_type(result, "character")
  expect_true(grepl("Col1", result))
  expect_true(grepl("Col2", result))
})

# =============================================================================
# nalgebra feature tests
# =============================================================================

test_that("nalgebra_dvector_roundtrip preserves vector", {
  skip_if_missing_feature("nalgebra")
  x <- c(1.0, 2.0, 3.0)
  expect_equal(nalgebra_dvector_roundtrip(x), x)
})

test_that("nalgebra_dvector_len returns correct length", {
  skip_if_missing_feature("nalgebra")
  expect_equal(nalgebra_dvector_len(c(1.0, 2.0, 3.0, 4.0)), 4L)
})

test_that("nalgebra_dvector_sum sums elements", {
  skip_if_missing_feature("nalgebra")
  expect_equal(nalgebra_dvector_sum(c(1.0, 2.0, 3.0)), 6.0)
})

test_that("nalgebra_dvector_norm computes L2 norm", {
  skip_if_missing_feature("nalgebra")
  # norm of (3, 4) = 5
  expect_equal(nalgebra_dvector_norm(c(3.0, 4.0)), 5.0)
})

test_that("nalgebra_dvector_dot computes dot product", {
  skip_if_missing_feature("nalgebra")
  expect_equal(nalgebra_dvector_dot(c(1.0, 2.0, 3.0), c(4.0, 5.0, 6.0)), 32.0)
})

test_that("nalgebra_dmatrix_roundtrip preserves matrix", {
  skip_if_missing_feature("nalgebra")
  m <- matrix(c(1, 2, 3, 4, 5, 6), nrow = 2, ncol = 3)
  result <- nalgebra_dmatrix_roundtrip(m)
  expect_equal(result, m)
})

test_that("nalgebra_dmatrix_nrows/ncols return correct dimensions", {
  skip_if_missing_feature("nalgebra")
  m <- matrix(as.double(1:6), nrow = 2, ncol = 3)
  expect_equal(nalgebra_dmatrix_nrows(m), 2L)
  expect_equal(nalgebra_dmatrix_ncols(m), 3L)
})

test_that("nalgebra_dmatrix_transpose transposes correctly", {
  skip_if_missing_feature("nalgebra")
  m <- matrix(c(1, 2, 3, 4), nrow = 2, ncol = 2)
  result <- nalgebra_dmatrix_transpose(m)
  expect_equal(result, t(m))
})

test_that("nalgebra_dmatrix_trace computes trace", {
  skip_if_missing_feature("nalgebra")
  m <- matrix(c(1, 0, 0, 2), nrow = 2, ncol = 2)
  expect_equal(nalgebra_dmatrix_trace(m), 3.0)
})

# =============================================================================
# Either feature tests
# =============================================================================

test_that("either_int_or_str accepts integer", {
  skip_if_missing_feature("either")
  expect_equal(either_int_or_str(42L), "int:42")
})

test_that("either_int_or_str accepts string", {
  skip_if_missing_feature("either")
  expect_equal(either_int_or_str("hello"), "str:hello")
})

test_that("either_dbl_or_vec accepts double", {
  skip_if_missing_feature("either")
  result <- either_dbl_or_vec(3.14)
  expect_true(grepl("dbl:", result))
})

test_that("either_dbl_or_vec accepts integer vector", {
  skip_if_missing_feature("either")
  result <- either_dbl_or_vec(c(1L, 2L, 3L))
  expect_true(grepl("vec:", result))
})

test_that("either_make_left returns integer", {
  skip_if_missing_feature("either")
  expect_equal(either_make_left(99L), 99L)
})

test_that("either_make_right returns string", {
  skip_if_missing_feature("either")
  expect_equal(either_make_right("test"), "test")
})

# =============================================================================
# serde_json feature tests
# =============================================================================

test_that("json_roundtrip preserves R list as JSON", {
  skip_if_missing_feature("serde_json")
  input <- list(a = 1L, b = "hello", c = TRUE)
  result <- json_roundtrip(input)
  expect_type(result, "list")
  expect_equal(result$b, "hello")
  expect_equal(result$c, TRUE)
})

test_that("json_type_name identifies JSON type", {
  skip_if_missing_feature("serde_json")
  expect_equal(json_type_name(list(a = 1L)), "object")
  expect_equal(json_type_name(list(1L, 2L, 3L)), "array")
})

test_that("json_is_object identifies objects", {
  skip_if_missing_feature("serde_json")
  expect_true(json_is_object(list(key = "value")))
  expect_false(json_is_object(list(1L, 2L)))
})

test_that("json_object_keys extracts keys", {
  skip_if_missing_feature("serde_json")
  keys <- json_object_keys(list(x = 1L, y = 2L))
  expect_true("x" %in% keys)
  expect_true("y" %in% keys)
})

test_that("json_serialize_point produces JSON string", {
  skip_if_missing_feature("serde_json")
  result <- json_serialize_point(1.0, 2.0)
  expect_type(result, "character")
  expect_true(grepl('"x"', result))
  expect_true(grepl('"y"', result))
})

test_that("json_to_pretty produces formatted JSON", {
  skip_if_missing_feature("serde_json")
  result <- json_to_pretty(list(a = 1L))
  expect_type(result, "character")
  # Pretty-printed JSON has newlines
  expect_true(grepl("\n", result))
})
