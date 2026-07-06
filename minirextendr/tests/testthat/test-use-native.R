# Tests for native R package integration (use-native.R)

test_that("warn_known_bad_package errors on Rcpp ecosystem", {
  expect_error(minirextendr:::warn_known_bad_package("Rcpp"), "Rcpp/cpp11 ecosystem")
  expect_error(minirextendr:::warn_known_bad_package("RcppArmadillo"), "Rcpp/cpp11 ecosystem")
  expect_error(minirextendr:::warn_known_bad_package("cpp11"), "Rcpp/cpp11 ecosystem")
})

test_that("warn_known_bad_package errors on no-header packages", {

  expect_error(minirextendr:::warn_known_bad_package("noweb"), "no header files")
  expect_error(minirextendr:::warn_known_bad_package("zigg"), "no header files")
})

test_that("warn_known_bad_package warns on system-lib packages", {
  expect_warning(minirextendr:::warn_known_bad_package("sf"), "system libraries")
  expect_warning(minirextendr:::warn_known_bad_package("HighFive"), "system libraries")
})

test_that("warn_known_bad_package is silent for good packages", {
  expect_silent(minirextendr:::warn_known_bad_package("cli"))
  expect_silent(minirextendr:::warn_known_bad_package("nanoarrow"))
  expect_silent(minirextendr:::warn_known_bad_package("vctrs"))
})

test_that("discover_native_package finds installed packages", {
  skip_if_not_installed("cli")
  info <- minirextendr:::discover_native_package("cli")
  expect_true(info$has_include)
  expect_true(dir.exists(info$include_path))
})

test_that("discover_native_package returns empty for missing packages", {
  info <- minirextendr:::discover_native_package("nonexistent_pkg_1234567890")
  expect_false(info$has_include)
})

test_that("detect_header_mode returns c for pure C packages", {
  skip_if_not_installed("cli")
  mode <- minirextendr:::detect_header_mode(system.file("include", package = "cli"))
  expect_equal(mode, "c")
})

test_that("resolve_blocklist_files adds boost for BH-dependent packages", {
  skip_if_not_installed("svines")
  bl <- resolve_blocklist_files("svines")
  expect_true(".*/boost/.*" %in% bl)
})

test_that("assert_bindgen_installed errors when bindgen missing", {
  withr::local_envvar(PATH = "")
  expect_error(minirextendr:::assert_bindgen_installed(), "bindgen.*not installed")
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

test_that("C→C++ fallback works for packages with C++ includes in .h files", {
  skip_if_not_installed("Countr")
  skip_if(nchar(Sys.which("bindgen")) == 0, "bindgen not installed")
  # Countr has .h files that #include <cmath> — detected as C initially,
  # falls back to C++ mode
  result <- check_native_package("Countr")
  expect_true(result$success)
  expect_equal(result$mode, "cpp")
})

test_that("resolve_include_paths is recursive", {
  skip_if_not_installed("svines")
  skip_if_not_installed("RcppEigen")
  info <- minirextendr:::discover_native_package("svines")
  paths <- resolve_include_paths("svines", info$include_path)
  # svines → rvinecopulib → Eigen. RcppEigen should be in the resolved paths.
  eigen_found <- any(grepl("RcppEigen", paths))
  expect_true(eigen_found)
})

# =============================================================================
# anchor precondition (#1171)
# =============================================================================

test_that("abort_if_missing_native_anchor accepts any of the three anchors", {
  expect_true(minirextendr:::abort_if_missing_native_anchor(
    "AC_CONFIG_SRCDIR([src/rust/lib.rs])"
  ))
  expect_true(minirextendr:::abort_if_missing_native_anchor(
    "dnl MINIREXTENDR: native-pkg-cppflags insertion marker"
  ))
  expect_true(minirextendr:::abort_if_missing_native_anchor(
    "AC_SUBST([NATIVE_PKG_CPPFLAGS])"
  ))
  expect_error(
    minirextendr:::abort_if_missing_native_anchor(
      c("AC_INIT([testpkg], [1.0])", "AC_OUTPUT")
    ),
    "AC_CONFIG_SRCDIR"
  )
})

test_that("use_native_package aborts before mutating DESCRIPTION when configure.ac has no anchor (#1171)", {
  skip_if_not_installed("cli")
  skip_if(nchar(Sys.which("bindgen")) == 0, "bindgen not installed")
  tmp <- withr::local_tempdir()
  # Hand-mangled configure.ac: no AC_CONFIG_SRCDIR, no NATIVE_PKG_CPPFLAGS
  # section, no insertion marker -- nothing to hang include detection off.
  writeLines(c(
    "AC_INIT([testpkg], [1.0])",
    "AC_OUTPUT"
  ), file.path(tmp, "configure.ac"))
  writeLines(c(
    "Package: testpkg",
    "Title: Test",
    "Version: 0.1.0"
  ), file.path(tmp, "DESCRIPTION"))
  desc_before <- readLines(file.path(tmp, "DESCRIPTION"), warn = FALSE)
  withr::local_dir(tmp)
  usethis::local_project(tmp, quiet = TRUE, force = TRUE, setwd = FALSE)

  expect_error(
    use_native_package("cli", path = tmp),
    "AC_CONFIG_SRCDIR"
  )

  # The whole point: DESCRIPTION untouched, no half-configured package.
  expect_identical(
    readLines(file.path(tmp, "DESCRIPTION"), warn = FALSE),
    desc_before
  )
})

test_that("add_native_to_configure_ac aborts instead of warning on missing anchor (#1171)", {
  tmp <- withr::local_tempdir()
  writeLines(c(
    "AC_INIT([testpkg], [1.0])",
    "AC_OUTPUT"
  ), file.path(tmp, "configure.ac"))
  writeLines("Package: testpkg", file.path(tmp, "DESCRIPTION"))
  conf_before <- readLines(file.path(tmp, "configure.ac"), warn = FALSE)
  withr::local_dir(tmp)
  usethis::local_project(tmp, quiet = TRUE, force = TRUE, setwd = FALSE)

  expect_error(
    minirextendr:::add_native_to_configure_ac("cli"),
    "AC_CONFIG_SRCDIR"
  )
  expect_identical(
    readLines(file.path(tmp, "configure.ac"), warn = FALSE),
    conf_before
  )
})

test_that("add_native_to_configure_ac appends detection block", {
  skip_if_not_installed("cli")
  tmp <- withr::local_tempdir()
  # Create a minimal configure.ac
  writeLines(c(
    'NATIVE_PKG_CPPFLAGS=""',
    'dnl MINIREXTENDR: native-pkg-cppflags insertion marker',
    'AC_SUBST([NATIVE_PKG_CPPFLAGS])',
    'AC_CONFIG_SRCDIR([src/rust/lib.rs])'
  ), file.path(tmp, "configure.ac"))
  writeLines("Package: testpkg", file.path(tmp, "DESCRIPTION"))
  withr::local_dir(tmp)
  usethis::local_project(tmp, quiet = TRUE, force = TRUE, setwd = FALSE)
  minirextendr:::add_native_to_configure_ac("cli")
  lines <- readLines(file.path(tmp, "configure.ac"))
  expect_true(any(grepl("dnl native: cli", lines)))
  expect_true(any(grepl("CLI_INCLUDE", lines)))
  # Second call should be idempotent
  minirextendr:::add_native_to_configure_ac("cli")
  lines2 <- readLines(file.path(tmp, "configure.ac"))
  expect_equal(sum(grepl("dnl native: cli", lines2)), 1)
})
