# Scaffold smoke tests for CI
#
# These tests verify CRAN prep workflows, path-dep detection, and
# auto-vendor sync round-trips.

# =============================================================================
# Path-dep detection tests (run everywhere, no cargo/autoconf needed)
# =============================================================================

test_that("check_path_deps detects path deps in [dependencies]", {
  tmp <- tempfile("path-deps-")
  dir.create(tmp)
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  cargo_dir <- file.path(tmp, "src", "rust")
  dir.create(cargo_dir, recursive = TRUE)
  writeLines(c(
    "[package]",
    'name = "testpkg"',
    'version = "0.1.0"',
    "",
    "[dependencies]",
    'miniextendr-api = { git = "https://github.com/A2-ai/miniextendr" }',
    'dvs = { path = "../../../dvs" }',
    'other-crate = { version = "1.0", path = "../other" }',
    "",
    "[build-dependencies]",
    'build-helper = { path = "../build-helper" }',
    "",
    "[patch.crates-io]",
    'miniextendr-api = { path = "../../vendor/miniextendr-api" }'
  ), file.path(cargo_dir, "Cargo.toml"))

  result <- minirextendr:::check_path_deps(tmp)

  expect_s3_class(result, "data.frame")
  expect_equal(nrow(result), 3)
  expect_true("dvs" %in% result$crate)
  expect_true("other-crate" %in% result$crate)
  expect_true("build-helper" %in% result$crate)

  # [patch.crates-io] entries should NOT be flagged

  expect_false("miniextendr-api" %in% result$crate)

  # Check paths are correct
  expect_equal(result$path[result$crate == "dvs"], "../../../dvs")
  expect_equal(result$path[result$crate == "build-helper"], "../build-helper")
})

test_that("check_path_deps returns empty data frame when no path deps", {
  tmp <- tempfile("no-path-deps-")
  dir.create(tmp)
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  cargo_dir <- file.path(tmp, "src", "rust")
  dir.create(cargo_dir, recursive = TRUE)
  writeLines(c(
    "[package]",
    'name = "testpkg"',
    'version = "0.1.0"',
    "",
    "[dependencies]",
    'miniextendr-api = { git = "https://github.com/A2-ai/miniextendr" }',
    'serde = "1.0"'
  ), file.path(cargo_dir, "Cargo.toml"))

  result <- minirextendr:::check_path_deps(tmp)
  expect_equal(nrow(result), 0)
  expect_equal(names(result), c("crate", "path"))
})

test_that("check_path_deps returns empty data frame when no Cargo.toml", {
  tmp <- tempfile("no-cargo-")
  dir.create(tmp)
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  result <- minirextendr:::check_path_deps(tmp)
  expect_equal(nrow(result), 0)
})

test_that("check_path_deps ignores path deps in [patch.*] sections", {
  tmp <- tempfile("patch-path-")
  dir.create(tmp)
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  cargo_dir <- file.path(tmp, "src", "rust")
  dir.create(cargo_dir, recursive = TRUE)
  writeLines(c(
    "[package]",
    'name = "testpkg"',
    'version = "0.1.0"',
    "",
    "[dependencies]",
    'serde = "1.0"',
    "",
    "[patch.crates-io]",
    'miniextendr-api = { path = "../../vendor/miniextendr-api" }',
    'miniextendr-macros = { path = "../../vendor/miniextendr-macros" }',
    "",
    '[patch."https://github.com/A2-ai/miniextendr"]',
    'miniextendr-api = { path = "../../vendor/miniextendr-api" }'
  ), file.path(cargo_dir, "Cargo.toml"))

  result <- minirextendr:::check_path_deps(tmp)
  expect_equal(nrow(result), 0)
})

# =============================================================================
# Auto-vendor sync detection tests (run everywhere)
# =============================================================================

test_that("detect_miniextendr_local finds repo via .vendor-source", {
  tmp <- tempfile("detect-source-")
  dir.create(tmp)
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  # Create a fake miniextendr repo
  fake_repo <- file.path(tmp, "miniextendr-repo")
  dir.create(fake_repo)
  api_dir <- file.path(fake_repo, "miniextendr-api")
  dir.create(api_dir)
  writeLines("", file.path(api_dir, "Cargo.toml"))

  # Create vendor dir with .vendor-source marker
  vendor_dir <- file.path(tmp, "project", "vendor")
  dir.create(vendor_dir, recursive = TRUE)
  writeLines(fake_repo, file.path(vendor_dir, ".vendor-source"))

  result <- minirextendr:::detect_miniextendr_local(vendor_dir)
  expect_equal(result, normalizePath(fake_repo, mustWork = TRUE))
})

test_that("detect_miniextendr_local returns NULL when no source found", {
  tmp <- tempfile("detect-none-")
  dir.create(tmp)
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  vendor_dir <- file.path(tmp, "vendor")
  dir.create(vendor_dir)

  result <- minirextendr:::detect_miniextendr_local(vendor_dir)
  expect_null(result)
})

# =============================================================================
# Standalone CRAN prep smoke test (requires cargo + autoconf)
# =============================================================================

test_that("standalone scaffold can vendor for CRAN prep", {
  skip_on_ci()
  skip_if_not(nzchar(Sys.which("autoconf")), "autoconf not available")
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
  skip_if_no_local_repo()

  miniextendr_path <- find_miniextendr_repo()

  tmp <- tempfile("cran-standalone-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  pkg_path <- file.path(tmp, "testpkg")

  # Create package
  suppressWarnings(suppressMessages({
    usethis::create_package(pkg_path, open = FALSE)
    use_miniextendr(path = pkg_path, local_path = miniextendr_path)
  }))

  # Autoconf + configure
  suppressMessages({
    miniextendr_autoconf(path = pkg_path)
    miniextendr_configure(path = pkg_path)
  })

  # Vendor for CRAN
  suppressMessages({
    miniextendr_vendor(path = pkg_path)
  })

  # Verify vendor tarball was created
  tarball <- file.path(pkg_path, "inst", "vendor.tar.xz")
  expect_true(file.exists(tarball), info = "vendor.tar.xz should exist")
  expect_true(file.size(tarball) > 0, info = "vendor.tar.xz should be non-empty")

  # Configure in tarball mode (inst/vendor.tar.xz present — configure auto-detects)
  suppressMessages({
    miniextendr_configure(path = pkg_path)
  })

  # cargo check --offline should work with vendored deps
  result <- withr::with_dir(file.path(pkg_path, "src", "rust"), {
    system2("cargo", c("check", "--offline"), stdout = TRUE, stderr = TRUE)
  })
  status <- attr(result, "status")
  expect_true(is.null(status) || status == 0,
              info = paste("cargo check --offline failed:", paste(result, collapse = "\n")))
})

# =============================================================================
# Monorepo CRAN prep smoke test (requires cargo + autoconf)
# =============================================================================

test_that("monorepo scaffold can vendor for CRAN prep", {
  skip_on_ci()
  skip_if_not(nzchar(Sys.which("autoconf")), "autoconf not available")
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
  skip_if_no_local_repo()

  miniextendr_path <- find_miniextendr_repo()

  tmp <- tempfile("cran-monorepo-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  # Create monorepo
  suppressMessages({
    create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg-rs",
                                local_path = miniextendr_path, open = FALSE)
  })

  rpkg_path <- file.path(tmp, "testpkg")

  # Autoconf + configure
  suppressMessages({
    miniextendr_autoconf(path = rpkg_path)
    miniextendr_configure(path = rpkg_path)
  })

  # Vendor for CRAN
  suppressMessages({
    miniextendr_vendor(path = rpkg_path)
  })

  # Verify vendor tarball was created
  tarball <- file.path(rpkg_path, "inst", "vendor.tar.xz")
  expect_true(file.exists(tarball), info = "vendor.tar.xz should exist")
  expect_true(file.size(tarball) > 0, info = "vendor.tar.xz should be non-empty")
})
