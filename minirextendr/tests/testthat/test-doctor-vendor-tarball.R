# Tests for the single, severity-gated vendor tarball check in
# miniextendr_doctor() (BUG6, audit/_worklist-2026-07-03.md).
#
# Before the fix, inst/vendor.tar.xz was checked twice, unconditionally: once
# as a hard "fail" in the "Install-mode signal" section, and again as a
# "warn" in the "Vendor tarball" section -- double-counting the same file
# with contradictory guidance (one message said remove it, the other said
# it's fine mid-CRAN-prep). The fix collapses this to ONE check, with
# severity gated on whether a `.git` ancestor is present (dev source tree
# => fail; otherwise => warn).

# Build a minimal fake R-package directory suitable for doctor().
make_doctor_tarball_pkg <- function(with_git = FALSE) {
  tmp <- tempfile("doctor-tarball-")
  dir.create(tmp)
  writeLines(
    c("Package: testpkg", "Title: Test", "Version: 0.1.0", ""),
    file.path(tmp, "DESCRIPTION")
  )
  writeLines("", file.path(tmp, "NAMESPACE"))

  rust_dir <- file.path(tmp, "src", "rust")
  dir.create(rust_dir, recursive = TRUE)
  writeLines(
    c("[package]", 'name = "testpkg"', "", "[dependencies]", 'miniextendr-api = "*"'),
    file.path(rust_dir, "Cargo.toml")
  )

  if (with_git) {
    dir.create(file.path(tmp, ".git"))
  }

  tmp
}

plant_tarball <- function(pkg) {
  inst_dir <- file.path(pkg, "inst")
  dir.create(inst_dir, showWarnings = FALSE)
  writeLines("fake", file.path(inst_dir, "vendor.tar.xz"))
}

test_that("miniextendr_doctor reports no vendor tarball leak when absent", {
  pkg <- make_doctor_tarball_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)

  result <- suppressMessages(miniextendr_doctor(pkg))

  expect_true(any(grepl("No vendor tarball leak", result$pass, fixed = TRUE)))
  expect_false(any(grepl("vendor.tar.xz", result$warn, fixed = TRUE)))
  expect_false(any(grepl("vendor.tar.xz", result$fail, fixed = TRUE)))
})

test_that("miniextendr_doctor warns (not fails) on a tarball present outside a git tree", {
  pkg <- make_doctor_tarball_pkg(with_git = FALSE)
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  plant_tarball(pkg)

  result <- suppressMessages(miniextendr_doctor(pkg))

  expect_true(any(grepl("vendor.tar.xz", result$warn, fixed = TRUE)))
  expect_false(any(grepl("vendor.tar.xz", result$fail, fixed = TRUE)))
})

test_that("miniextendr_doctor fails (not warns) on a tarball present inside a dev git tree", {
  pkg <- make_doctor_tarball_pkg(with_git = TRUE)
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  plant_tarball(pkg)

  result <- suppressMessages(miniextendr_doctor(pkg))

  expect_true(any(grepl("vendor.tar.xz", result$fail, fixed = TRUE)))
  expect_false(any(grepl("vendor.tar.xz", result$warn, fixed = TRUE)))
})

test_that("miniextendr_doctor reports the vendor tarball exactly once, in a dev git tree", {
  pkg <- make_doctor_tarball_pkg(with_git = TRUE)
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  plant_tarball(pkg)

  result <- suppressMessages(miniextendr_doctor(pkg))

  n_mentions <- sum(grepl("vendor.tar.xz", c(result$warn, result$fail), fixed = TRUE))
  expect_equal(n_mentions, 1L)
})

test_that("miniextendr_doctor reports the vendor tarball exactly once, outside a git tree", {
  pkg <- make_doctor_tarball_pkg(with_git = FALSE)
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  plant_tarball(pkg)

  result <- suppressMessages(miniextendr_doctor(pkg))

  n_mentions <- sum(grepl("vendor.tar.xz", c(result$warn, result$fail), fixed = TRUE))
  expect_equal(n_mentions, 1L)
})
