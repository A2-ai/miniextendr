# Tests for native R package integration (use-native.R)

test_that("warn_known_bad_package errors on Rcpp ecosystem", {
  expect_error(warn_known_bad_package("Rcpp"), "Rcpp/cpp11 ecosystem")
  expect_error(warn_known_bad_package("RcppArmadillo"), "Rcpp/cpp11 ecosystem")
  expect_error(warn_known_bad_package("cpp11"), "Rcpp/cpp11 ecosystem")
})

test_that("warn_known_bad_package errors on no-header packages", {

  expect_error(warn_known_bad_package("noweb"), "no header files")
  expect_error(warn_known_bad_package("zigg"), "no header files")
})

test_that("warn_known_bad_package warns on system-lib packages", {
  expect_warning(warn_known_bad_package("sf"), "system libraries")
  expect_warning(warn_known_bad_package("HighFive"), "system libraries")
})

test_that("warn_known_bad_package is silent for good packages", {
  expect_silent(warn_known_bad_package("cli"))
  expect_silent(warn_known_bad_package("nanoarrow"))
  expect_silent(warn_known_bad_package("vctrs"))
})

test_that("discover_native_package finds installed packages", {
  skip_if_not_installed("cli")
  info <- discover_native_package("cli")
  expect_true(info$has_include)
  expect_true(dir.exists(info$include_path))
})

test_that("discover_native_package returns empty for missing packages", {
  info <- discover_native_package("nonexistent_pkg_1234567890")
  expect_false(info$has_include)
})

test_that("detect_header_mode returns c for pure C packages", {
  skip_if_not_installed("cli")
  mode <- detect_header_mode(system.file("include", package = "cli"))
  expect_equal(mode, "c")
})

test_that("resolve_blocklist_files adds boost for BH-dependent packages", {
  skip_if_not_installed("svines")
  bl <- resolve_blocklist_files("svines")
  expect_true(".*/boost/.*" %in% bl)
})

test_that("check_native_package succeeds for cli", {
  skip_if_not_installed("cli")
  skip_if(nchar(Sys.which("bindgen")) == 0, "bindgen not installed")
  result <- check_native_package("cli")
  expect_true(result$success)
  expect_equal(result$mode, "c")
  expect_true(result$has_static_fns)
  expect_gt(result$n_lines, 0)
})
