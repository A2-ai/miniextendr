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

test_that("cleanup restores a frozen manifest byte for byte from the pre-freeze snapshot", {
  root <- make_minimal_project()
  withr::defer(unlink(root, recursive = TRUE))
  rust <- file.path(root, "src", "rust")
  dir.create(rust, recursive = TRUE)
  manifest <- file.path(rust, "Cargo.toml")
  sidecar <- file.path(rust, ".Cargo.toml.prefreeze")
  # CRLF, odd spacing and a comment: the restore must not normalise anything.
  original <- charToRaw(paste0(
    '[package]\r\nname = "fixture"\r\n\r\n',
    '[dependencies]\r\ncore   = { path = "../../../core" }  # sibling\r\n'
  ))
  writeBin(original, sidecar)
  writeLines(c(
    '[package]', 'name = "fixture"', '',
    '[dependencies]', 'core = { path = "../../vendor/core", version = "*" }', '',
    '[patch.crates-io]', 'core = { path = "../../vendor/core" }'
  ), manifest)
  expect_length(minirextendr:::frozen_manifest_entries(root), 2L)

  expect_true(miniextendr_clean_vendor_leak(root))
  expect_identical(readBin(manifest, "raw", n = 1e6), original)
  expect_false(file.exists(sidecar))
  expect_length(minirextendr:::frozen_manifest_entries(root), 0L)

  # Nothing left to clean.
  expect_false(miniextendr_clean_vendor_leak(root))
})

test_that("a stale pre-freeze snapshot next to a restored manifest is removed", {
  root <- make_minimal_project()
  withr::defer(unlink(root, recursive = TRUE))
  rust <- file.path(root, "src", "rust")
  dir.create(rust, recursive = TRUE)
  manifest <- file.path(rust, "Cargo.toml")
  sidecar <- file.path(rust, ".Cargo.toml.prefreeze")
  original <- c('[dependencies]', 'core = { path = "../../../core" }')
  writeLines(original, manifest)
  writeLines(c('[dependencies]', 'core = { path = "../../../elsewhere" }'), sidecar)

  expect_false(miniextendr_clean_vendor_leak(root))
  expect_false(file.exists(sidecar))
  expect_identical(readLines(manifest), original)
})

test_that("the frozen-manifest report points at the snapshot when one exists", {
  entries <- list(list(section = "dependencies", crate = "core", path = "../../vendor/core"))
  report <- function(snapshot) {
    # One string; cli wraps alerts at the console width.
    paste(cli::cli_fmt(minirextendr:::report_frozen_manifest(entries, snapshot = snapshot)), collapse = " ")
  }
  expect_match(report(TRUE), "miniextendr_clean_vendor_leak()", fixed = TRUE)
  expect_no_match(report(TRUE), "no\\s+pre-freeze\\s+snapshot")
  expect_match(report(FALSE), "no\\s+pre-freeze\\s+snapshot")
})

# region: monorepo workspace root (#1786)

make_monorepo_leak <- function() {
  ws <- tempfile("leak-ws-")
  pkg <- file.path(ws, "rpkg")
  dir.create(file.path(pkg, "inst"), recursive = TRUE)
  dir.create(file.path(pkg, "src", "rust"), recursive = TRUE)
  writeLines("[workspace]\nmembers = [\"core\"]", file.path(ws, "Cargo.toml"))
  writeLines("Package: demo\nVersion: 0.0.0.9000", file.path(pkg, "DESCRIPTION"))
  writeLines("CARGO_FEATURES=\"\"", file.path(pkg, "configure.ac"))
  writeLines("fake", file.path(pkg, "inst", "vendor.tar.xz"))
  writeLines(
    c("[dependencies]", "core = { path = \"../../../core\" }"),
    file.path(pkg, "src", "rust", ".Cargo.toml.prefreeze")
  )
  writeLines(
    c("[dependencies]", "core = { version = \"*\" }", "",
      "[patch.crates-io]", "core = { path = \"../../vendor/core\" }"),
    file.path(pkg, "src", "rust", "Cargo.toml")
  )
  ws
}

test_that("miniextendr_clean_vendor_leak cleans the package subdirectory from a monorepo root", {
  ws <- make_monorepo_leak()
  on.exit(unlink(ws, recursive = TRUE), add = TRUE)
  pkg <- file.path(ws, "rpkg")

  msgs <- capture_messages(result <- miniextendr_clean_vendor_leak(ws))

  expect_true(result)
  expect_true(any(grepl("Monorepo layout detected", msgs)))
  expect_false(file.exists(file.path(pkg, "inst", "vendor.tar.xz")))
  expect_false(file.exists(file.path(pkg, "src", "rust", ".Cargo.toml.prefreeze")))
  expect_true(any(grepl("../../../core", readLines(file.path(pkg, "src", "rust", "Cargo.toml")), fixed = TRUE)))
})

test_that("miniextendr_doctor reports the leak of the package subdirectory from a monorepo root", {
  ws <- make_monorepo_leak()
  on.exit(unlink(ws, recursive = TRUE), add = TRUE)

  msgs <- capture_messages(result <- miniextendr_doctor(ws))

  expect_true(any(grepl("Monorepo layout detected", msgs)))
  expect_false(any(grepl("No vendor tarball leak", result$pass, fixed = TRUE)))
})

test_that("monorepo helpers abort for a directory that is not an R package", {
  empty <- tempfile("not-a-package-")
  dir.create(empty)
  on.exit(unlink(empty, recursive = TRUE), add = TRUE)

  expect_error(miniextendr_clean_vendor_leak(empty), "not an R package")
  expect_error(miniextendr_doctor(empty), "not an R package")

  # A directory with exactly one package subdirectory names that directory.
  dir.create(file.path(empty, "pkgdir"))
  writeLines("Package: x", file.path(empty, "pkgdir", "DESCRIPTION"))
  expect_error(miniextendr_clean_vendor_leak(empty), "pkgdir")
})

# endregion
