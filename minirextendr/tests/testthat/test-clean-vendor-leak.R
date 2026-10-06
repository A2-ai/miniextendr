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

# A workspace root (Cargo.toml, a git repository as in a developer checkout)
# whose package subdirectories each carry a leak: the archive, a frozen
# manifest and its pre-freeze snapshot. `marker` controls the configure.ac
# CARGO_FEATURES marker that rpkg_subdir_candidates() keys on.
make_monorepo_leak <- function(pkgs = "rpkg", marker = TRUE) {
  ws <- tempfile("leak-ws-")
  dir.create(ws)
  writeLines("[workspace]\nmembers = [\"core\"]", file.path(ws, "Cargo.toml"))
  if (nzchar(Sys.which("git"))) {
    suppressWarnings(system2(
      "git", c("-C", shQuote(ws), "init", "--quiet"), stdout = TRUE, stderr = TRUE
    ))
  } else {
    dir.create(file.path(ws, ".git"))
  }
  for (name in pkgs) {
    pkg <- file.path(ws, name)
    dir.create(file.path(pkg, "inst"), recursive = TRUE)
    dir.create(file.path(pkg, "src", "rust"), recursive = TRUE)
    writeLines("Package: demo\nVersion: 0.0.0.9000", file.path(pkg, "DESCRIPTION"))
    if (marker) writeLines("CARGO_FEATURES=\"\"", file.path(pkg, "configure.ac"))
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
  }
  ws
}

# cli wraps messages at the console width; compare them as one line.
one_line <- function(x) gsub("\\s+", " ", paste(x, collapse = " "))

test_that("miniextendr_clean_vendor_leak cleans the package subdirectory from a monorepo root", {
  ws <- make_monorepo_leak()
  on.exit(unlink(ws, recursive = TRUE), add = TRUE)
  pkg <- normalizePath(file.path(ws, "rpkg"))

  msgs <- capture_messages(result <- miniextendr_clean_vendor_leak(ws))

  expect_true(result)
  # The message names the caller and the resolved package directory in full.
  expect_match(one_line(msgs), "Monorepo layout detected", fixed = TRUE)
  expect_match(one_line(msgs), "miniextendr_clean_vendor_leak()", fixed = TRUE)
  expect_match(one_line(msgs), pkg, fixed = TRUE)
  expect_false(file.exists(file.path(pkg, "inst", "vendor.tar.xz")))
  expect_false(file.exists(file.path(pkg, "src", "rust", ".Cargo.toml.prefreeze")))
  expect_true(any(grepl("../../../core", readLines(file.path(pkg, "src", "rust", "Cargo.toml")), fixed = TRUE)))
})

test_that("miniextendr_doctor reports the leak of the package subdirectory from a monorepo root", {
  ws <- make_monorepo_leak()
  on.exit(unlink(ws, recursive = TRUE), add = TRUE)
  pkg <- normalizePath(file.path(ws, "rpkg"))

  msgs <- capture_messages(result <- miniextendr_doctor(ws))

  expect_match(one_line(msgs), "miniextendr_doctor()", fixed = TRUE)
  expect_match(one_line(msgs), pkg, fixed = TRUE)
  # Under a .git ancestor the archive is a failure, as in a developer checkout,
  # and the frozen manifest is reported next to it.
  expect_true("stale inst/vendor.tar.xz in source tree" %in% result$fail)
  expect_true(any(grepl("vendor-bound Cargo.toml", result$warn, fixed = TRUE)))
  expect_false("No vendor tarball leak" %in% result$pass)
})

test_that("the abort for a directory that is not an R package names the caller and the full path", {
  empty <- tempfile("not-a-package-")
  dir.create(empty)
  on.exit(unlink(empty, recursive = TRUE), add = TRUE)

  expect_error(miniextendr_clean_vendor_leak(empty), "not an R package")
  err <- expect_error(miniextendr_doctor(empty), "not an R package")
  # The example names the function that was called, not a fixed one.
  expect_match(one_line(conditionMessage(err)), 'miniextendr_doctor("', fixed = TRUE)
  expect_no_match(conditionMessage(err), "miniextendr_clean_vendor_leak")

  # One package subdirectory: the example is its full path, so it pastes from
  # any working directory.
  dir.create(file.path(empty, "pkgdir"))
  writeLines("Package: x", file.path(empty, "pkgdir", "DESCRIPTION"))
  err <- expect_error(miniextendr_clean_vendor_leak(empty), "not an R package")
  expect_match(
    one_line(conditionMessage(err)),
    sprintf('miniextendr_clean_vendor_leak("%s")', normalizePath(file.path(empty, "pkgdir"))),
    fixed = TRUE
  )
})

test_that("two package subdirectories give a hint these helpers can act on", {
  ws <- make_monorepo_leak(pkgs = c("rpkg", "rpkg2"))
  on.exit(unlink(ws, recursive = TRUE), add = TRUE)

  err <- expect_error(miniextendr_clean_vendor_leak(ws), "2 miniextendr packages")
  msg <- one_line(conditionMessage(err))
  # Neither helper has an `rpkg_subdir` argument; `path` is the way in.
  expect_no_match(msg, "rpkg_subdir")
  expect_match(msg, "Pass the package directory as `path`", fixed = TRUE)
  for (name in c("rpkg", "rpkg2")) {
    expect_match(
      msg,
      sprintf('miniextendr_clean_vendor_leak("%s")', normalizePath(file.path(ws, name))),
      fixed = TRUE
    )
  }
  # Nothing was cleaned.
  expect_true(file.exists(file.path(ws, "rpkg", "inst", "vendor.tar.xz")))
  expect_true(file.exists(file.path(ws, "rpkg2", "inst", "vendor.tar.xz")))
})

test_that("a package under a Rust workspace is resolved by DESCRIPTION, not the configure.ac marker", {
  # The #1786 stand-in: a package subdirectory without the marker. Given the
  # root, the helper names the package; given the package, it cleans it.
  ws <- make_monorepo_leak(marker = FALSE)
  on.exit(unlink(ws, recursive = TRUE), add = TRUE)
  pkg <- normalizePath(file.path(ws, "rpkg"))

  err <- expect_error(miniextendr_clean_vendor_leak(ws), "not an R package")
  expect_match(
    one_line(conditionMessage(err)),
    sprintf('miniextendr_clean_vendor_leak("%s")', pkg),
    fixed = TRUE
  )

  capture_messages(result <- miniextendr_clean_vendor_leak(pkg))
  expect_true(result)
  expect_false(file.exists(file.path(pkg, "inst", "vendor.tar.xz")))
})

# endregion
