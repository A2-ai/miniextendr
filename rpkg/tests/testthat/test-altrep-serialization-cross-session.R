# Cross-session ALTREP serialization round-trips (saveRDS → callr::r → readRDS)
#
# These tests verify that ALTREP objects serialized in one R session can be
# deserialized in a fresh session where miniextendr is loaded (ALTREP classes
# re-registered at init) AND in sessions without the package (falls back to
# the serialized state, which is a plain R vector).
#
# Every ALTREP type with serialization support should have a test here.

# Skip on Windows: callr/processx on Windows leaves orphan Rterm processes that
# hold stdout pipe handles open, preventing R CMD check from detecting test
# completion. This causes R CMD check to hang until the 90-minute CI timeout.
# Serialization behavior is platform-independent and tested on Linux/macOS.
skip_on_os("windows")

# Helper: cross-session round-trip with package loaded
cross_session_with_pkg <- function(value) {
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))
  saveRDS(value, tmp)
  callr::r(function(path) {
    library(miniextendr)
    readRDS(path)
  }, args = list(path = tmp))
}

# Helper: cross-session round-trip WITHOUT package (plain R fallback)
cross_session_without_pkg <- function(value) {
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))
  saveRDS(value, tmp)
  callr::r(function(path) {
    readRDS(path)
  }, args = list(path = tmp))
}

# region: Vec<T> types

test_that("Vec<i32> cross-session round-trip", {
  original <- iter_int_range(1L, 6L)  # [1,2,3,4,5]
  loaded <- cross_session_with_pkg(original)
  expect_equal(loaded, 1:5)
})

test_that("Vec<i32> cross-session without package", {
  original <- iter_int_range(1L, 6L)
  loaded <- cross_session_without_pkg(original)
  expect_equal(loaded, 1:5)
})

test_that("Vec<f64> cross-session round-trip", {
  original <- iter_real_squares(5L)  # [0,1,4,9,16]
  loaded <- cross_session_with_pkg(original)
  expect_equal(loaded, c(0, 1, 4, 9, 16))
})

test_that("Vec<f64> cross-session without package", {
  original <- iter_real_squares(5L)
  loaded <- cross_session_without_pkg(original)
  expect_equal(loaded, c(0, 1, 4, 9, 16))
})

test_that("Vec<bool> cross-session round-trip", {
  original <- iter_logical_alternating(4L)  # [T,F,T,F]
  loaded <- cross_session_with_pkg(original)
  # Element-by-element: callr may reconstruct ALTREP in parent process,
  # and DATAPTR materialization may not work across process boundaries on Windows
  expect_equal(length(loaded), 4L)
  expect_true(loaded[1])
  expect_false(loaded[2])
})

test_that("Vec<bool> cross-session without package", {
  original <- iter_logical_alternating(4L)
  loaded <- cross_session_without_pkg(original)
  expect_equal(length(loaded), 4L)
  expect_true(loaded[1])
  expect_false(loaded[2])
})

test_that("Vec<u8> cross-session round-trip", {
  original <- iter_raw_bytes(4L)  # [0,1,2,3]
  loaded <- cross_session_with_pkg(original)
  expect_equal(length(loaded), 4L)
  expect_equal(loaded[1], as.raw(0))
  expect_equal(loaded[4], as.raw(3))
})

test_that("Vec<u8> cross-session without package", {
  original <- iter_raw_bytes(4L)
  loaded <- cross_session_without_pkg(original)
  expect_equal(length(loaded), 4L)
  expect_equal(loaded[1], as.raw(0))
  expect_equal(loaded[4], as.raw(3))
})

test_that("Vec<String> cross-session round-trip", {
  original <- iter_string_items(3L)  # ["item_0","item_1","item_2"]
  loaded <- cross_session_with_pkg(original)
  expect_equal(loaded, c("item_0", "item_1", "item_2"))
})

test_that("Vec<String> cross-session without package", {
  original <- iter_string_items(3L)
  loaded <- cross_session_without_pkg(original)
  expect_equal(loaded, c("item_0", "item_1", "item_2"))
})

test_that("Vec<Rcomplex> cross-session round-trip", {
  original <- vec_complex_altrep(3L)  # [0-0i, 1-1i, 2-2i]
  loaded <- cross_session_with_pkg(original)
  expect_equal(length(loaded), 3L)
  expect_equal(Re(loaded[1]), 0)
  expect_equal(Im(loaded[1]), 0)
  expect_equal(Re(loaded[3]), 2)
  expect_equal(Im(loaded[3]), -2)
})

test_that("Vec<Rcomplex> cross-session without package", {
  original <- vec_complex_altrep(3L)
  loaded <- cross_session_without_pkg(original)
  expect_equal(length(loaded), 3L)
  expect_equal(Re(loaded[3]), 2)
  expect_equal(Im(loaded[3]), -2)
})

# endregion

# region: Vec<T> with NAs

test_that("Vec<i32> with NA cross-session round-trip", {
  original <- altrep_from_integers(c(1L, NA_integer_, 3L))
  loaded <- cross_session_with_pkg(original)
  expect_equal(loaded[1], 1L)
  expect_true(is.na(loaded[2]))
  expect_equal(loaded[3], 3L)
})

test_that("Vec<String> with NA cross-session round-trip", {
  original <- altrep_from_strings(c("a", NA_character_, "c"))
  loaded <- cross_session_with_pkg(original)
  expect_equal(loaded[1], "a")
  expect_true(is.na(loaded[2]))
  expect_equal(loaded[3], "c")
})

test_that("Vec<f64> with special values cross-session round-trip", {
  original <- altrep_from_doubles(c(1.0, NA_real_, Inf, -Inf, NaN))
  loaded <- cross_session_with_pkg(original)
  expect_equal(loaded[1], 1.0)
  expect_true(is.na(loaded[2]))
  expect_true(is.infinite(loaded[3]) && loaded[3] > 0)
  expect_true(is.infinite(loaded[4]) && loaded[4] < 0)
  expect_true(is.nan(loaded[5]))
})

# endregion

# region: Box<[T]> types

test_that("Box<[i32]> cross-session round-trip", {
  original <- boxed_ints(5L)  # [1,2,3,4,5]
  loaded <- cross_session_with_pkg(original)
  expect_equal(loaded, 1:5)
})

test_that("Box<[i32]> cross-session without package", {
  original <- boxed_ints(5L)
  loaded <- cross_session_without_pkg(original)
  expect_equal(loaded, 1:5)
})

test_that("Box<[f64]> cross-session round-trip", {
  original <- boxed_reals(4L)  # [1.5,3.0,4.5,6.0]
  loaded <- cross_session_with_pkg(original)
  expect_equal(length(loaded), 4L)
  expect_equal(loaded[1], 1.5)
  expect_equal(loaded[4], 6.0)
})

test_that("Box<[bool]> cross-session round-trip", {
  original <- boxed_logicals(4L)  # [T,F,T,F]
  loaded <- cross_session_with_pkg(original)
  expect_equal(length(loaded), 4L)
  expect_true(loaded[1])
  expect_false(loaded[2])
})

test_that("Box<[u8]> cross-session round-trip", {
  original <- boxed_raw(4L)  # [0,1,2,3]
  loaded <- cross_session_with_pkg(original)
  expect_equal(length(loaded), 4L)
  expect_equal(loaded[1], as.raw(0))
  expect_equal(loaded[4], as.raw(3))
})

test_that("Box<[String]> cross-session round-trip", {
  original <- boxed_strings(3L)  # ["boxed_0","boxed_1","boxed_2"]
  loaded <- cross_session_with_pkg(original)
  expect_equal(loaded, c("boxed_0", "boxed_1", "boxed_2"))
})

test_that("Box<[Rcomplex]> cross-session round-trip", {
  original <- boxed_complex(3L)
  loaded <- cross_session_with_pkg(original)
  expect_equal(length(loaded), 3L)
  expect_equal(Re(loaded[1]), 0.25)
  expect_equal(Im(loaded[1]), 0.75)
})

# endregion

# region: Range<T> types

test_that("Range<i32> cross-session round-trip", {
  original <- range_int_altrep(1L, 6L)  # [1,2,3,4,5]
  loaded <- cross_session_with_pkg(original)
  expect_equal(length(loaded), 5L)
  expect_equal(loaded[1], 1L)
  expect_equal(loaded[5], 5L)
})

test_that("Range<i32> cross-session without package", {
  original <- range_int_altrep(1L, 6L)
  loaded <- cross_session_without_pkg(original)
  expect_equal(length(loaded), 5L)
  expect_equal(loaded[1], 1L)
  expect_equal(loaded[5], 5L)
})

test_that("Range<i64> cross-session round-trip", {
  original <- range_i64_altrep(1, 6)  # [1,2,3,4,5]
  loaded <- cross_session_with_pkg(original)
  expect_equal(length(loaded), 5L)
  expect_equal(loaded[1], 1L)
  expect_equal(loaded[5], 5L)
})

test_that("Range<f64> cross-session round-trip", {
  original <- range_real_altrep(0, 5)  # [0,1,2,3,4]
  loaded <- cross_session_with_pkg(original)
  expect_equal(length(loaded), 5L)
  expect_equal(loaded[1], 0.0)
  expect_equal(loaded[5], 4.0)
})

test_that("Range<f64> cross-session without package", {
  original <- range_real_altrep(0, 5)
  loaded <- cross_session_without_pkg(original)
  expect_equal(length(loaded), 5L)
  expect_equal(loaded[1], 0.0)
  expect_equal(loaded[5], 4.0)
})

# endregion

# region: Proc-macro-derived ALTREP types (1-field struct via #[miniextendr])
#
# These test the proc-macro codegen path (explicit base= in altrep.rs) which
# previously passed null DllInfo to R, breaking cross-session readRDS (#62).

test_that("Proc-macro ALTREP (ConstantInt) cross-session round-trip", {
  original <- constant_int()  # 10 elements, all 42
  loaded <- cross_session_with_pkg(original)
  expect_equal(length(loaded), 10L)
  expect_equal(loaded[1], 42L)
  expect_equal(loaded[10], 42L)
  expect_equal(sum(loaded), 420L)
})

test_that("Proc-macro ALTREP (ConstantInt) cross-session without package", {
  original <- constant_int()
  loaded <- cross_session_without_pkg(original)
  expect_equal(length(loaded), 10L)
  expect_equal(loaded[1], 42L)
  expect_equal(sum(loaded), 420L)
})

# endregion

# region: Arrow arrays (feature-gated)

test_that("Float64Array cross-session round-trip", {
  original <- zero_copy_arrow_f64_altrep(c(1.0, 2.0, 3.0))
  loaded <- cross_session_with_pkg(original)
  expect_equal(loaded, c(10.0, 20.0, 30.0))
})

test_that("Float64Array with NAs cross-session round-trip", {
  original <- zero_copy_arrow_f64_altrep(c(1.0, NA, 3.0))
  loaded <- cross_session_with_pkg(original)
  expect_equal(loaded[1], 10.0)
  expect_true(is.na(loaded[2]))
  expect_equal(loaded[3], 30.0)
})

test_that("Int32Array cross-session round-trip", {
  original <- zero_copy_arrow_i32_altrep(c(1L, 2L, 3L))
  loaded <- cross_session_with_pkg(original)
  expect_equal(loaded, c(101L, 102L, 103L))
})

test_that("Float64Array cross-session without package", {
  original <- zero_copy_arrow_f64_altrep(c(1.0, 2.0, 3.0))
  loaded <- cross_session_without_pkg(original)
  expect_equal(loaded, c(10.0, 20.0, 30.0))
})

# endregion

# region: Edge cases

test_that("Empty ALTREP cross-session round-trip", {
  original <- altrep_from_integers(integer(0))
  loaded <- cross_session_with_pkg(original)
  expect_equal(length(loaded), 0L)
  expect_equal(loaded, integer(0))
})

test_that("Single-element ALTREP cross-session round-trip", {
  original <- altrep_from_integers(42L)
  loaded <- cross_session_with_pkg(original)
  expect_equal(loaded, 42L)
})

test_that("Double round-trip: saveRDS → readRDS → saveRDS → readRDS cross-session", {
  original <- iter_int_range(1L, 6L)

  tmp1 <- tempfile(fileext = ".rds")
  tmp2 <- tempfile(fileext = ".rds")
  on.exit(unlink(c(tmp1, tmp2)))
  saveRDS(original, tmp1)

  # First cross-session: load and re-save
  loaded1 <- callr::r(function(path_in, path_out) {
    library(miniextendr)
    obj <- readRDS(path_in)
    saveRDS(obj, path_out)
    obj
  }, args = list(path_in = tmp1, path_out = tmp2))

  expect_equal(loaded1, 1:5)

  # Second cross-session: load the re-saved file
  loaded2 <- callr::r(function(path) {
    readRDS(path)
  }, args = list(path = tmp2))

  expect_equal(loaded2, 1:5)
})

# endregion
