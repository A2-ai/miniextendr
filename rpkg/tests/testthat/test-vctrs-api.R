# Tests for vctrs C API integration
#
# These tests verify that the vctrs support correctly interfaces with vctrs'
# maturing C API (obj_is_vector, short_vec_size, short_vec_recycle).
#
# Note: Test functions are internal (not exported), so we use `:::` to access them.

# Helper to skip if vctrs feature is not enabled
skip_if_vctrs_disabled <- function() {
  skip_if_not("vctrs" %in% miniextendr::rpkg_enabled_features(), "vctrs feature not enabled")
  skip_if_not_installed("vctrs")
}

test_that("vctrs support is initialized", {
  skip_if_vctrs_disabled()

  # vctrs should be initialized when the package loads
  expect_true(miniextendr:::test_vctrs_is_initialized())
})

test_that("obj_is_vector returns TRUE for vectors", {
  skip_if_vctrs_disabled()

  # Atomic vectors
  expect_true(miniextendr:::test_vctrs_obj_is_vector(logical(0)))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(TRUE))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(integer(0)))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(1L))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(double(0)))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(1.5))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(character(0)))
  expect_true(miniextendr:::test_vctrs_obj_is_vector("hello"))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(raw(0)))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(as.raw(1)))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(complex(0)))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(1+2i))

  # Lists
  expect_true(miniextendr:::test_vctrs_obj_is_vector(list()))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(list(1, 2, 3)))

  # Data frames
  expect_true(miniextendr:::test_vctrs_obj_is_vector(data.frame()))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(data.frame(x = 1:3, y = letters[1:3])))
})

test_that("obj_is_vector returns FALSE for non-vectors", {
  skip_if_vctrs_disabled()

  # NULL
  expect_false(miniextendr:::test_vctrs_obj_is_vector(NULL))

  # Language objects
  expect_false(miniextendr:::test_vctrs_obj_is_vector(quote(x + y)))
  expect_false(miniextendr:::test_vctrs_obj_is_vector(call("sum")))

  # Environments
  expect_false(miniextendr:::test_vctrs_obj_is_vector(new.env()))
  expect_false(miniextendr:::test_vctrs_obj_is_vector(globalenv()))

  # Symbols
  expect_false(miniextendr:::test_vctrs_obj_is_vector(as.symbol("x")))

  # Functions
  expect_false(miniextendr:::test_vctrs_obj_is_vector(function() {}))
  expect_false(miniextendr:::test_vctrs_obj_is_vector(mean))
})

test_that("short_vec_size returns correct sizes", {
  skip_if_vctrs_disabled()

  # Atomic vectors
  expect_equal(miniextendr:::test_vctrs_short_vec_size(integer(0)), 0L)
  expect_equal(miniextendr:::test_vctrs_short_vec_size(1:10), 10L)
  expect_equal(miniextendr:::test_vctrs_short_vec_size(c(1.5, 2.5, 3.5)), 3L)
  expect_equal(miniextendr:::test_vctrs_short_vec_size(c("a", "b")), 2L)

  # Lists
  expect_equal(miniextendr:::test_vctrs_short_vec_size(list()), 0L)
  expect_equal(miniextendr:::test_vctrs_short_vec_size(list(1, 2, 3, 4)), 4L)

  # Data frames - size is number of rows
  expect_equal(miniextendr:::test_vctrs_short_vec_size(data.frame()), 0L)
  expect_equal(miniextendr:::test_vctrs_short_vec_size(data.frame(x = 1:5)), 5L)
  expect_equal(miniextendr:::test_vctrs_short_vec_size(mtcars), 32L)
})

test_that("short_vec_recycle recycles vectors", {
  skip_if_vctrs_disabled()

  # Recycle length-1 to length-5
  result <- miniextendr:::test_vctrs_short_vec_recycle(42L, 5L)
  expect_equal(result, rep(42L, 5))

  # Recycle length-1 character
  result <- miniextendr:::test_vctrs_short_vec_recycle("x", 3L)
  expect_equal(result, c("x", "x", "x"))

  # No recycling needed when sizes match
  result <- miniextendr:::test_vctrs_short_vec_recycle(1:3, 3L)
  expect_equal(result, 1:3)

  # Empty vector stays empty
  result <- miniextendr:::test_vctrs_short_vec_recycle(integer(0), 0L)
  expect_equal(result, integer(0))
})

test_that("short_vec_recycle matches vctrs behavior", {
  skip_if_vctrs_disabled()

  # Test that our wrapper matches vctrs::vec_recycle
  x <- 1:1
  size <- 5L
  our_result <- miniextendr:::test_vctrs_short_vec_recycle(x, size)
  vctrs_result <- vctrs::vec_recycle(x, size)
  expect_equal(our_result, vctrs_result)

  x <- list(a = 1)
  size <- 3L
  our_result <- miniextendr:::test_vctrs_short_vec_recycle(x, size)
  vctrs_result <- vctrs::vec_recycle(x, size)
  expect_equal(our_result, vctrs_result)
})

test_that("vctrs error messages are informative", {
  # Test error message formatting (doesn't require vctrs)
  expect_match(
    miniextendr:::test_vctrs_error_message(0L),
    "not initialized"
  )
  expect_match(
    miniextendr:::test_vctrs_error_message(1L),
    "not found"
  )
  expect_match(
    miniextendr:::test_vctrs_error_message(2L),
    "already initialized"
  )
  expect_match(
    miniextendr:::test_vctrs_error_message(3L),
    "main thread"
  )
})

# =============================================================================
# Construction helper tests
# =============================================================================

test_that("new_vctr creates basic vctr from double", {
  skip_if_vctrs_disabled()

  data <- c(0.1, 0.2, 0.3)
  result <- miniextendr:::test_new_vctr(data, "vctrs_percent")

  # Check class structure: c("vctrs_percent", "vctrs_vctr")
  # (inherit_base_type defaults to FALSE for non-list)
  expect_equal(class(result), c("vctrs_percent", "vctrs_vctr"))

  # Data should be preserved (use vec_data to access underlying data)
  expect_equal(vctrs::vec_data(result), c(0.1, 0.2, 0.3))

  # vctrs should recognize it as a vector
  expect_true(vctrs::obj_is_vector(result))
  expect_equal(vctrs::vec_size(result), 3L)
})

test_that("new_vctr with inherit_base_type=TRUE includes type", {
  skip_if_vctrs_disabled()

  data <- c(1.5, 2.5)
  result <- miniextendr:::test_new_vctr_inherit(data, "my_class", TRUE)

  # Class should include "double" at the end
  expect_equal(class(result), c("my_class", "vctrs_vctr", "double"))
})

test_that("new_vctr with inherit_base_type=FALSE excludes type", {
  skip_if_vctrs_disabled()

  data <- 1:3
  result <- miniextendr:::test_new_vctr_inherit(data, "my_class", FALSE)

  # Class should NOT include base type
  expect_equal(class(result), c("my_class", "vctrs_vctr"))
})

test_that("new_vctr on list defaults to inherit_base_type=TRUE", {
  skip_if_vctrs_disabled()

  data <- list(a = 1, b = 2)
  result <- miniextendr:::test_new_vctr(data, "my_list_class")

  # Class should include "list" at the end
  expect_equal(class(result), c("my_list_class", "vctrs_vctr", "list"))

  # vctrs should recognize it
  expect_true(vctrs::obj_is_vector(result))
})

test_that("new_vctr on list with inherit_base_type=FALSE errors", {
  skip_if_vctrs_disabled()

  data <- list(1, 2, 3)
  expect_error(
    miniextendr:::test_new_vctr_inherit(data, "my_class", FALSE),
    "list data requires inherit_base_type"
  )
})

test_that("new_vctr with multiple class names", {
  skip_if_vctrs_disabled()

  data <- c(1, 2, 3)
  result <- miniextendr:::test_new_vctr(data, c("subclass", "superclass"))

  expect_equal(class(result), c("subclass", "superclass", "vctrs_vctr"))
})

test_that("new_rcrd creates basic record", {
  skip_if_vctrs_disabled()

  fields <- list(x = 1:3, y = c("a", "b", "c"))
  result <- miniextendr:::test_new_rcrd(fields, "vctrs_rational")

  # Check class structure
  expect_equal(class(result), c("vctrs_rational", "vctrs_rcrd", "vctrs_vctr"))

  # Fields should be preserved (use vctrs::field() to access record fields)
  expect_equal(vctrs::field(result, "x"), 1:3)
  expect_equal(vctrs::field(result, "y"), c("a", "b", "c"))

  # vctrs should recognize it
  expect_true(vctrs::obj_is_vector(result))
  expect_equal(vctrs::vec_size(result), 3L)
})

test_that("new_rcrd errors on empty fields", {
  skip_if_vctrs_disabled()

  expect_error(
    miniextendr:::test_new_rcrd(list(), "my_record"),
    "record must have at least one field"
  )
})

test_that("new_rcrd errors on unnamed fields", {
  skip_if_vctrs_disabled()

  expect_error(
    miniextendr:::test_new_rcrd(list(1:3, 4:6), "my_record"),
    "record fields must be named"
  )
})

test_that("new_rcrd errors on mismatched field lengths", {
  skip_if_vctrs_disabled()

  expect_error(
    miniextendr:::test_new_rcrd(list(x = 1:3, y = 1:5), "my_record"),
    "field 'y' has length 5 but expected 3"
  )
})

test_that("new_rcrd errors on duplicate field names", {
  skip_if_vctrs_disabled()

  expect_error(
    miniextendr:::test_new_rcrd(list(x = 1:3, x = 4:6), "my_record"),
    "duplicate field name"
  )
})

test_that("new_list_of creates list_of with ptype", {
  skip_if_vctrs_disabled()

  x <- list(1:2, 3:4, 5:6)
  ptype <- integer(0)
  result <- miniextendr:::test_new_list_of_ptype(x, ptype, character(0))

  # Check class structure
  expect_equal(class(result), c("vctrs_list_of", "vctrs_vctr", "list"))

  # Check ptype attribute
  expect_equal(attr(result, "ptype"), integer(0))

  # vctrs should recognize it
  expect_true(vctrs::obj_is_vector(result))
  expect_equal(vctrs::vec_size(result), 3L)
})

test_that("new_list_of creates list_of with size", {
  skip_if_vctrs_disabled()

  x <- list(1:2, 3:4)
  result <- miniextendr:::test_new_list_of_size(x, 2L, character(0))

  # Check class structure
  expect_equal(class(result), c("vctrs_list_of", "vctrs_vctr", "list"))

  # Check size attribute
  expect_equal(attr(result, "size"), 2L)
})

test_that("new_list_of with custom class", {
  skip_if_vctrs_disabled()

  x <- list(c("a", "b"), c("c", "d"))
  ptype <- character(0)
  result <- miniextendr:::test_new_list_of_ptype(x, ptype, "my_list_of")

  # Custom class should come first
  expect_equal(class(result), c("my_list_of", "vctrs_list_of", "vctrs_vctr", "list"))
})

test_that("VctrsBuildError messages are informative", {
  skip_if_vctrs_disabled()

  expect_match(
    miniextendr:::test_vctrs_build_error_message("not_initialized"),
    "vctrs not initialized"
  )
  expect_match(
    miniextendr:::test_vctrs_build_error_message("not_a_vector"),
    "not a vector"
  )
  expect_match(
    miniextendr:::test_vctrs_build_error_message("list_requires_inherit"),
    "inherit_base_type"
  )
  expect_match(
    miniextendr:::test_vctrs_build_error_message("field_length_mismatch"),
    "field 'x' has length 5"
  )
  expect_match(
    miniextendr:::test_vctrs_build_error_message("empty_record"),
    "at least one field"
  )
  expect_match(
    miniextendr:::test_vctrs_build_error_message("duplicate_field"),
    "duplicate field name"
  )
  expect_match(
    miniextendr:::test_vctrs_build_error_message("unnamed_fields"),
    "must be named"
  )
  expect_match(
    miniextendr:::test_vctrs_build_error_message("missing_ptype_or_size"),
    "requires at least one"
  )
  expect_match(
    miniextendr:::test_vctrs_build_error_message("invalid_size"),
    "invalid size"
  )
})
