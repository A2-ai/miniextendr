# Tests for #[derive(Vctrs)] proc macro
#
# These tests verify that the derive macro generates correct vctrs-compatible
# types. Compare with test-vctrs-class-example.R which tests the manual approach.

# Helper to skip if vctrs feature is not enabled
skip_if_vctrs_disabled <- function() {
  skip_if_not("vctrs" %in% miniextendr::rpkg_enabled_features(), "vctrs feature not enabled")
  skip_if_not_installed("vctrs")
}

# =============================================================================
# DerivedPercent tests (simple vctr backed by double)
# =============================================================================

test_that("new_derived_percent creates a vctrs vector", {
  skip_if_vctrs_disabled()

  x <- new_derived_percent(c(0.25, 0.5, 0.75))

  expect_true(vctrs::vec_is(x))
  expect_s3_class(x, "derived_percent")
  expect_equal(length(x), 3)
})

test_that("derived_percent has correct class structure", {
  skip_if_vctrs_disabled()

  x <- new_derived_percent(c(0.1, 0.2))

  # Should have vctrs_vctr in class hierarchy
  expect_true("vctrs_vctr" %in% class(x))
  # derived_percent should be first

  expect_equal(class(x)[1], "derived_percent")
})

test_that("format.derived_percent formats values", {
  skip_if_vctrs_disabled()

  x <- new_derived_percent(c(0.25, 0.5, 0.75))

  # format() should return formatted strings
  formatted <- format(x)
  expect_type(formatted, "character")
  expect_equal(length(formatted), 3)
})

test_that("vec_ptype_abbr.derived_percent returns '%'", {
  skip_if_vctrs_disabled()

  x <- new_derived_percent(c(0.25))

  expect_equal(vctrs::vec_ptype_abbr(x), "%")
})

test_that("vec_ptype_full.derived_percent returns class name", {
  skip_if_vctrs_disabled()

  x <- new_derived_percent(c(0.25))

  expect_equal(vctrs::vec_ptype_full(x), "derived_percent")
})

test_that("derived_percent subsetting preserves class", {
  skip_if_vctrs_disabled()

  x <- new_derived_percent(c(0.1, 0.2, 0.3, 0.4, 0.5))

  subset <- x[2:4]

  expect_s3_class(subset, "derived_percent")
  expect_equal(length(subset), 3)
  expect_equal(vctrs::vec_data(subset), c(0.2, 0.3, 0.4))
})

test_that("derived_percent vec_c combines vectors", {
  skip_if_vctrs_disabled()

  x <- new_derived_percent(c(0.1, 0.2))
  y <- new_derived_percent(c(0.3, 0.4))

  combined <- vctrs::vec_c(x, y)

  expect_s3_class(combined, "derived_percent")
  expect_equal(length(combined), 4)
  expect_equal(vctrs::vec_data(combined), c(0.1, 0.2, 0.3, 0.4))
})

test_that("derived_percent vec_ptype returns zero-length prototype", {
  skip_if_vctrs_disabled()

  x <- new_derived_percent(c(0.1, 0.2, 0.3))

  ptype <- vctrs::vec_ptype(x)

  expect_s3_class(ptype, "derived_percent")
  expect_equal(vctrs::vec_size(ptype), 0L)
})

test_that("derived_percent_class_info returns correct metadata", {
  skip_if_vctrs_disabled()

  info <- derived_percent_class_info()

  expect_type(info, "character")
  expect_true(any(grepl("CLASS_NAME: derived_percent", info)))
  expect_true(any(grepl("KIND: Vctr", info)))
  expect_true(any(grepl("ABBR: Some", info)))
})

# =============================================================================
# DerivedRational tests (record type with multiple fields)
# =============================================================================

test_that("new_derived_rational creates a vctrs record", {
  skip_if_vctrs_disabled()

  x <- new_derived_rational(c(1L, 2L, 3L), c(2L, 3L, 4L))

  expect_true(vctrs::vec_is(x))
  expect_s3_class(x, "derived_rational")
  expect_equal(vctrs::vec_size(x), 3)
})

test_that("derived_rational has correct class structure", {
  skip_if_vctrs_disabled()

  x <- new_derived_rational(c(1L, 2L), c(3L, 4L))

  # Should have vctrs_rcrd in class hierarchy (record type)
  expect_true("vctrs_rcrd" %in% class(x))
  expect_true("vctrs_vctr" %in% class(x))
  # derived_rational should be first
  expect_equal(class(x)[1], "derived_rational")
})

test_that("derived_rational fields are accessible", {
  skip_if_vctrs_disabled()

  x <- new_derived_rational(c(1L, 2L, 3L), c(4L, 5L, 6L))

  # Access fields using vctrs::field()
  expect_equal(vctrs::field(x, "n"), c(1L, 2L, 3L))
  expect_equal(vctrs::field(x, "d"), c(4L, 5L, 6L))
})

test_that("format.derived_rational shows n/d format", {
  skip_if_vctrs_disabled()

  x <- new_derived_rational(c(1L, 2L, 3L), c(2L, 3L, 4L))

  formatted <- format(x)

  expect_equal(formatted, c("1/2", "2/3", "3/4"))
})

test_that("vec_ptype_full.derived_rational returns class name", {
  skip_if_vctrs_disabled()

  x <- new_derived_rational(c(1L), c(2L))

  expect_equal(vctrs::vec_ptype_full(x), "derived_rational")
})

test_that("derived_rational subsetting preserves class and slices fields", {
  skip_if_vctrs_disabled()

  x <- new_derived_rational(c(1L, 2L, 3L, 4L, 5L), c(10L, 20L, 30L, 40L, 50L))

  subset <- x[2:4]

  expect_s3_class(subset, "derived_rational")
  expect_equal(vctrs::vec_size(subset), 3)
  expect_equal(vctrs::field(subset, "n"), c(2L, 3L, 4L))
  expect_equal(vctrs::field(subset, "d"), c(20L, 30L, 40L))
})

test_that("derived_rational vec_c combines records", {
  skip_if_vctrs_disabled()

  x <- new_derived_rational(c(1L, 2L), c(3L, 4L))
  y <- new_derived_rational(c(5L, 6L), c(7L, 8L))

  combined <- vctrs::vec_c(x, y)

  expect_s3_class(combined, "derived_rational")
  expect_equal(vctrs::vec_size(combined), 4)
  expect_equal(vctrs::field(combined, "n"), c(1L, 2L, 5L, 6L))
  expect_equal(vctrs::field(combined, "d"), c(3L, 4L, 7L, 8L))
})

test_that("derived_rational vec_ptype returns zero-length prototype", {
  skip_if_vctrs_disabled()

  x <- new_derived_rational(c(1L, 2L, 3L), c(4L, 5L, 6L))

  ptype <- vctrs::vec_ptype(x)

  expect_s3_class(ptype, "derived_rational")
  expect_equal(vctrs::vec_size(ptype), 0L)
  # Fields should also be empty
  expect_equal(length(vctrs::field(ptype, "n")), 0)
  expect_equal(length(vctrs::field(ptype, "d")), 0)
})

test_that("derived_rational_class_info returns correct metadata", {
  skip_if_vctrs_disabled()

  info <- derived_rational_class_info()

  expect_type(info, "character")
  expect_true(any(grepl("CLASS_NAME: derived_rational", info)))
  # KIND for record types is Rcrd
  expect_true(any(grepl("KIND: Rcrd", info)))
})

test_that("derived_rational errors on mismatched lengths", {
  skip_if_vctrs_disabled()

  expect_error(
    new_derived_rational(c(1L, 2L, 3L), c(4L, 5L)),
    "same length"
  )
})

# =============================================================================
# Comparison: derive vs manual approach
# =============================================================================

test_that("derived_percent behaves like manual percent", {

  skip_if_vctrs_disabled()

  # Create both types
  derived <- new_derived_percent(c(0.25, 0.5))
  manual <- new_percent(c(0.25, 0.5))

  # Both should be vctrs vectors

  expect_true(vctrs::vec_is(derived))
  expect_true(vctrs::vec_is(manual))


  # Both should have same underlying data (use vec_data to access)
  expect_equal(vctrs::vec_data(derived), vctrs::vec_data(manual))

  # Both should support subsetting
  expect_equal(vctrs::vec_data(derived[1]), vctrs::vec_data(manual[1]))
})

# =============================================================================
# DerivedIntLists tests (list_of type)
# =============================================================================

test_that("new_derived_int_lists creates a vctrs list_of", {
  skip_if_vctrs_disabled()

  x <- new_derived_int_lists(list(1:3, 4:6, 7:10))

  expect_true(vctrs::vec_is(x))
  expect_s3_class(x, "derived_int_lists")
  expect_s3_class(x, "vctrs_list_of")
  expect_equal(vctrs::vec_size(x), 3)
})

test_that("derived_int_lists elements are integer vectors", {
  skip_if_vctrs_disabled()

  x <- new_derived_int_lists(list(1:3, 4:5))

  expect_equal(x[[1]], 1:3)
  expect_equal(x[[2]], 4:5)
})

test_that("derived_int_lists subsetting preserves class", {
  skip_if_vctrs_disabled()

  x <- new_derived_int_lists(list(1:2, 3:4, 5:6))

  subset <- x[1:2]

  expect_s3_class(subset, "derived_int_lists")
  expect_equal(vctrs::vec_size(subset), 2)
})

# =============================================================================
# DerivedPoint tests (record with proxy_equal/compare/order)
# =============================================================================

test_that("new_derived_point creates a vctrs record", {
  skip_if_vctrs_disabled()

  x <- new_derived_point(c(1.0, 2.0, 3.0), c(4.0, 5.0, 6.0))

  expect_true(vctrs::vec_is(x))
  expect_s3_class(x, "derived_point")
  expect_s3_class(x, "vctrs_rcrd")
  expect_equal(vctrs::vec_size(x), 3)
})

test_that("derived_point fields are accessible", {
  skip_if_vctrs_disabled()

  x <- new_derived_point(c(1.0, 2.0), c(3.0, 4.0))

  expect_equal(vctrs::field(x, "x"), c(1.0, 2.0))
  expect_equal(vctrs::field(x, "y"), c(3.0, 4.0))
})

test_that("derived_point vec_proxy_equal works", {
  skip_if_vctrs_disabled()

  x <- new_derived_point(c(1.0, 2.0, 1.0), c(2.0, 3.0, 2.0))

  # vec_equal should work via proxy_equal
  result <- vctrs::vec_equal(x[1], x[3])
  expect_true(result)

  result2 <- vctrs::vec_equal(x[1], x[2])
  expect_false(result2)
})

# =============================================================================
# DerivedTemp tests (arith/math support)
# =============================================================================

test_that("new_derived_temp creates a vctrs vector", {
  skip_if_vctrs_disabled()

  x <- new_derived_temp(c(20.0, 25.0, 30.0))

  expect_true(vctrs::vec_is(x))
  expect_s3_class(x, "derived_temp")
  expect_equal(vctrs::vec_size(x), 3)
})

test_that("vec_ptype_abbr.derived_temp returns '°'", {
  skip_if_vctrs_disabled()

  x <- new_derived_temp(c(20.0))

  expect_equal(vctrs::vec_ptype_abbr(x), "°")
})

test_that("derived_temp arithmetic works", {
  skip_if_vctrs_disabled()

  x <- new_derived_temp(c(20.0, 25.0))
  y <- new_derived_temp(c(5.0, 10.0))

  # Addition
  result <- vctrs::vec_arith("+", x, y)
  expect_s3_class(result, "derived_temp")
  expect_equal(vctrs::vec_data(result), c(25.0, 35.0))

  # Subtraction
  result2 <- vctrs::vec_arith("-", x, y)
  expect_s3_class(result2, "derived_temp")
  expect_equal(vctrs::vec_data(result2), c(15.0, 15.0))
})

test_that("derived_temp math works", {
  skip_if_vctrs_disabled()

  x <- new_derived_temp(c(4.0, 9.0, 16.0))

  # sqrt
  result <- vctrs::vec_math("sqrt", x)
  expect_s3_class(result, "derived_temp")
  expect_equal(vctrs::vec_data(result), c(2.0, 3.0, 4.0))

  # abs
  y <- new_derived_temp(c(-5.0, 10.0, -15.0))
  result2 <- vctrs::vec_math("abs", y)
  expect_s3_class(result2, "derived_temp")
  expect_equal(vctrs::vec_data(result2), c(5.0, 10.0, 15.0))
})
