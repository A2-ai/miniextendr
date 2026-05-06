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
