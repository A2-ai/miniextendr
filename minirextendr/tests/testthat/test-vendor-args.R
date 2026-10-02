# The `cargo revendor` argument vector vendor_crates_io() builds, and the
# `revendor_args` passthrough from miniextendr_vendor() /
# miniextendr_build_tarball() / miniextendr_check() (#1714).
#
# Behavioural through mocks: run_with_logging() records the arguments instead
# of running cargo. The real vendor + offline build is covered by
# test-scaffold-smoke.R and test-vendor-cache.R.

make_vendor_pkg_root <- function() {
  tmp <- withr::local_tempdir(.local_envir = parent.frame())
  writeLines("Package: testpkg\nTitle: Test\nVersion: 0.1.0\n", file.path(tmp, "DESCRIPTION"))
  dir.create(file.path(tmp, "src", "rust"), recursive = TRUE)
  writeLines(c("[package]", 'name = "testpkg"'), file.path(tmp, "src", "rust", "Cargo.toml"))
  tmp
}

# Mock the toolchain probes and the cargo call; return an environment whose
# `args` field holds the arguments of the last `cargo` invocation.
mock_cargo_revendor <- function(env = parent.frame()) {
  state <- new.env(parent = emptyenv())
  testthat::local_mocked_bindings(
    check_rust = function() invisible(TRUE),
    check_cargo_revendor = function() invisible(TRUE),
    run_with_logging = function(command, args = character(), ...) {
      state$command <- command
      state$args <- args
      list(status = 0L, output = character(), log_file = "cargo-revendor.log", success = TRUE)
    },
    .package = "minirextendr",
    .env = env
  )
  state
}

strip_flags <- function(args) grep("^--strip-", args, value = TRUE)

test_that("vendor_crates_io(): trims with --strip-all and freezes by default", {
  pkg <- make_vendor_pkg_root()
  state <- mock_cargo_revendor()

  suppressMessages(vendor_crates_io(pkg))

  expect_identical(state$command, "cargo")
  expect_identical(state$args[[1L]], "revendor")
  expect_identical(strip_flags(state$args), "--strip-all")
  expect_true("--freeze" %in% state$args)
  expect_false("--compress" %in% state$args)
})

test_that("vendor_crates_io(): an archive adds the compression flags", {
  pkg <- make_vendor_pkg_root()
  state <- mock_cargo_revendor()
  tarball <- file.path(pkg, "inst", "vendor.tar.xz")

  suppressMessages(vendor_crates_io(pkg, tarball = tarball))

  at <- match("--compress", state$args)
  expect_identical(state$args[at + 1L], tarball)
  expect_true(all(c("--blank-md", "--source-marker", "--force") %in% state$args))
  expect_identical(strip_flags(state$args), "--strip-all")
})

test_that("vendor_crates_io(): revendor_args are appended after the defaults", {
  pkg <- make_vendor_pkg_root()
  state <- mock_cargo_revendor()
  tarball <- file.path(pkg, "inst", "vendor.tar.xz")
  extra <- c("--compression-level", "9")

  suppressMessages(vendor_crates_io(pkg, tarball = tarball, revendor_args = extra))

  n <- length(state$args)
  expect_identical(state$args[(n - 1L):n], extra)
  expect_identical(strip_flags(state$args), "--strip-all")
})

test_that("vendor_crates_io(): a caller's --strip-* flag replaces the default --strip-all", {
  # cargo-revendor rejects --strip-toml-sections next to any other strip flag,
  # so the default must drop out rather than be appended to.
  for (flag in c("--strip-toml-sections", "--strip-tests", "--strip-benches",
                 "--strip-examples", "--strip-bins", "--strip-all")) {
    pkg <- make_vendor_pkg_root()
    state <- mock_cargo_revendor()

    suppressMessages(vendor_crates_io(pkg, revendor_args = flag))

    expect_identical(strip_flags(state$args), flag, info = flag)
  }
})

test_that("vendor_crates_io(): revendor_args must be a character vector", {
  pkg <- make_vendor_pkg_root()
  state <- mock_cargo_revendor()

  expect_error(vendor_crates_io(pkg, revendor_args = 9), "character vector")
  expect_null(state$args)
})

test_that("miniextendr_vendor() passes revendor_args to vendor_crates_io()", {
  pkg <- make_vendor_pkg_root()
  seen <- new.env(parent = emptyenv())
  testthat::local_mocked_bindings(
    vendor_crates_io = function(path = ".", tarball = NULL, revendor_args = character()) {
      seen$revendor_args <- revendor_args
      writeLines("fake", tarball)
      invisible(TRUE)
    }
  )

  suppressMessages(miniextendr_vendor(pkg, revendor_args = "--strip-toml-sections"))
  expect_identical(seen$revendor_args, "--strip-toml-sections")

  suppressMessages(miniextendr_vendor(pkg))
  expect_identical(seen$revendor_args, character())
})

test_that("miniextendr_build_tarball() passes revendor_args to miniextendr_vendor()", {
  pkg <- make_vendor_pkg_root()
  seen <- new.env(parent = emptyenv())
  testthat::local_mocked_bindings(
    miniextendr_build = function(...) invisible(TRUE),
    miniextendr_vendor = function(path = ".", revendor_args = character()) {
      seen$revendor_args <- revendor_args
      invisible(NULL)
    }
  )
  testthat::local_mocked_bindings(
    build = function(path, dest_path = NULL, ...) file.path(dest_path, "testpkg_0.1.0.tar.gz"),
    .package = "pkgbuild"
  )

  suppressMessages(miniextendr_build_tarball(pkg, dest_path = pkg,
                                             revendor_args = c("--compression-level", "9")))
  expect_identical(seen$revendor_args, c("--compression-level", "9"))
})

test_that("miniextendr_check() passes revendor_args to miniextendr_build_tarball()", {
  skip_if_not_installed("rcmdcheck")
  pkg <- make_vendor_pkg_root()
  seen <- new.env(parent = emptyenv())
  testthat::local_mocked_bindings(
    miniextendr_build_tarball = function(path = ".", dest_path = NULL, args = character(),
                                         revendor_args = character()) {
      seen$revendor_args <- revendor_args
      file.path(dest_path, "testpkg_0.1.0.tar.gz")
    }
  )
  testthat::local_mocked_bindings(
    rcmdcheck = function(path, ...) invisible(list(path = path)),
    .package = "rcmdcheck"
  )

  suppressMessages(miniextendr_check(pkg, revendor_args = "--strip-toml-sections"))
  expect_identical(seen$revendor_args, "--strip-toml-sections")
})
