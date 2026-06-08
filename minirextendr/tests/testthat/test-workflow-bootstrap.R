# Tests for miniextendr_build()'s fresh-package bootstrap helpers (#822).
# These exercise the pure-R guards without invoking a compile/build.

make_pkg_root <- function() {
  tmp <- tempfile("workflow-bootstrap-")
  dir.create(tmp)
  writeLines(
    "Package: testpkg\nTitle: Test\nVersion: 0.1.0\n",
    file.path(tmp, "DESCRIPTION")
  )
  writeLines("", file.path(tmp, "NAMESPACE"))
  dir.create(file.path(tmp, "R"))
  tmp
}

test_that("wrappers_file_exists detects a generated *-wrappers.R", {
  tmp <- make_pkg_root()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  # Fresh package: no wrappers file yet.
  expect_false(minirextendr:::wrappers_file_exists(tmp))

  # Once a *-wrappers.R lands, it is detected regardless of the package stem.
  writeLines("# generated", file.path(tmp, "R", "testpkg-wrappers.R"))
  expect_true(minirextendr:::wrappers_file_exists(tmp))
})

test_that("wrappers_file_exists is FALSE when the R/ directory is missing", {
  tmp <- tempfile("workflow-bootstrap-nodir-")
  dir.create(tmp)
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  expect_false(minirextendr:::wrappers_file_exists(tmp))
})

test_that("clear_install_mode_latch removes tarball, vendor/, and .cargo/", {
  tmp <- make_pkg_root()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  inst_dir <- file.path(tmp, "inst")
  dir.create(inst_dir)
  tarball <- file.path(inst_dir, "vendor.tar.xz")
  writeLines("fake tarball", tarball)

  vendor_dir <- file.path(tmp, "vendor")
  dir.create(vendor_dir)
  writeLines("x", file.path(vendor_dir, "marker"))

  cargo_dir <- file.path(tmp, "src", "rust", ".cargo")
  dir.create(cargo_dir, recursive = TRUE)
  writeLines("[build]", file.path(cargo_dir, "config.toml"))

  expect_true(minirextendr:::clear_install_mode_latch(tmp))

  expect_false(file.exists(tarball))
  expect_false(dir.exists(vendor_dir))
  expect_false(dir.exists(cargo_dir))
})

test_that("clear_install_mode_latch is a no-op when nothing is present", {
  tmp <- make_pkg_root()
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  expect_true(minirextendr:::clear_install_mode_latch(tmp))
})
