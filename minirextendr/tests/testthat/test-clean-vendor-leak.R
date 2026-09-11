# Tests for miniextendr_clean_vendor_leak()

make_minimal_project <- function() {
  tmp <- tempfile("clean-vendor-leak-")
  dir.create(tmp)
  usethis::proj_set(tmp, force = TRUE)
  writeLines("Package: testpkg\nTitle: Test\nVersion: 0.1.0\n", file.path(tmp, "DESCRIPTION"))
  writeLines("", file.path(tmp, "NAMESPACE"))
  tmp
}

test_that("miniextendr_clean_vendor_leak removes inst/vendor.tar.xz when present", {
  tmp <- make_minimal_project()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  inst_dir <- file.path(tmp, "inst")
  dir.create(inst_dir, showWarnings = FALSE)
  tarball <- file.path(inst_dir, "vendor.tar.xz")
  writeLines("fake tarball", tarball)
  expect_true(file.exists(tarball))

  result <- miniextendr_clean_vendor_leak(tmp)

  expect_true(isTRUE(result))
  expect_false(file.exists(tarball))
})

test_that("miniextendr_clean_vendor_leak returns FALSE when tarball is absent", {
  tmp <- make_minimal_project()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  # No inst/ directory — tarball definitely absent
  result <- miniextendr_clean_vendor_leak(tmp)

  expect_true(isFALSE(result))
})

test_that("miniextendr_clean_vendor_leak is idempotent", {
  tmp <- make_minimal_project()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  inst_dir <- file.path(tmp, "inst")
  dir.create(inst_dir, showWarnings = FALSE)
  tarball <- file.path(inst_dir, "vendor.tar.xz")
  writeLines("fake tarball", tarball)

  result1 <- miniextendr_clean_vendor_leak(tmp)
  result2 <- miniextendr_clean_vendor_leak(tmp)

  expect_true(isTRUE(result1))
  expect_true(isFALSE(result2))
  expect_false(file.exists(tarball))
})

test_that("frozen manifest detection reports dependency and patch paths together", {
  root <- withr::local_tempdir()
  rust <- file.path(root, "src", "rust")
  dir.create(rust, recursive = TRUE)
  writeLines(c(
    '[package]', 'name = "fixture"',
    '[dependencies]', 'core = { path = "../../vendor/core", version = "*" }',
    'live = { path = "../../../live" }',
    '[build-dependencies.helper]', 'path = "../../vendor/helper"',
    '[patch.crates-io]', 'core = { path = "../../vendor/core" }',
    'separate = { path = "../../vendor-other/separate" }'
  ), file.path(rust, "Cargo.toml"))
  entries <- minirextendr:::frozen_manifest_entries(root)
  expect_identical(vapply(entries, `[[`, character(1), "crate"), c("core", "helper", "core"))
  expect_identical(vapply(entries, `[[`, character(1), "section"),
                   c("dependencies", "build-dependencies", "patch.crates-io"))
})

test_that("cleanup reports frozen entries even when the tarball is already absent", {
  root <- make_minimal_project()
  withr::defer(unlink(root, recursive = TRUE))
  dir.create(file.path(root, "src", "rust"), recursive = TRUE)
  manifest <- file.path(root, "src", "rust", "Cargo.toml")
  original <- c('[dependencies]', 'core = { path = "../../vendor/core" }',
                '[patch.crates-io]', 'core = { path = "../../vendor/core" }')
  writeLines(original, manifest)
  reported <- NULL
  testthat::local_mocked_bindings(
    report_frozen_manifest = function(entries) reported <<- entries,
    .package = "minirextendr"
  )
  expect_false(miniextendr_clean_vendor_leak(root))
  expect_length(reported, 2L)
  expect_identical(readLines(manifest), original)
})
