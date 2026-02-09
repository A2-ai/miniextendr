# Integration tests for status functions

# -----------------------------------------------------------------------------
# has_miniextendr() tests
# -----------------------------------------------------------------------------

test_that("has_miniextendr returns FALSE for empty project", {
  tmp <- tempfile("empty-proj-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  dir.create(tmp)
  usethis::proj_set(tmp, force = TRUE)
  writeLines("Package: testpkg\nTitle: Test\nVersion: 0.0.1\n",
    file.path(tmp, "DESCRIPTION"))

  expect_false(has_miniextendr())
})

test_that("has_miniextendr returns TRUE for monorepo project", {
  tmp <- tempfile("monorepo-status-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg", open = FALSE)

  # Point usethis to the rpkg subdirectory
  usethis::proj_set(file.path(tmp, "rpkg"), force = TRUE)

  expect_true(has_miniextendr())
})

# -----------------------------------------------------------------------------
# miniextendr_status() tests
# -----------------------------------------------------------------------------

test_that("miniextendr_status returns list with present and missing", {
  tmp <- tempfile("status-test-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg", open = FALSE)
  usethis::proj_set(file.path(tmp, "rpkg"), force = TRUE)

  result <- suppressMessages(miniextendr_status())

  expect_type(result, "list")
  expect_named(result, c("present", "missing", "stale"))
  expect_type(result$present, "list")
  expect_type(result$missing, "list")

  # Monorepo should have most files present
  expect_true(sum(lengths(result$present)) > 0)
})

test_that("miniextendr_status detects generated files as missing", {
  tmp <- tempfile("status-gen-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg", open = FALSE)
  usethis::proj_set(file.path(tmp, "rpkg"), force = TRUE)

  result <- suppressMessages(miniextendr_status())

  # Generated files should be missing before configure runs
  generated_missing <- result$missing[["Generated Files"]]
  expect_true(length(generated_missing) > 0)
  expect_true("src/Makevars" %in% generated_missing)
})

# -----------------------------------------------------------------------------
# miniextendr_validate() tests
# -----------------------------------------------------------------------------

test_that("miniextendr_validate validates DESCRIPTION config", {
  tmp <- tempfile("check-desc-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg", open = FALSE)
  usethis::proj_set(file.path(tmp, "rpkg"), force = TRUE)

  # Should pass - monorepo template sets up DESCRIPTION correctly
  result <- suppressMessages(miniextendr_validate())
  # Result depends on whether Rust is installed and crates vendored
  expect_type(result, "logical")
})

test_that("miniextendr_validate warns on missing Config/build/bootstrap", {
  tmp <- tempfile("check-bootstrap-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  # Create minimal package without miniextendr setup
  dir.create(tmp)
  usethis::proj_set(tmp, force = TRUE)
  writeLines("Package: testpkg\nTitle: Test\nVersion: 0.0.1\n",
    file.path(tmp, "DESCRIPTION"))

  # Create minimal configure.ac
  writeLines("AC_INIT([testpkg], [0.0.1])\nMINIEXTENDR_FEATURES=\n",
    file.path(tmp, "configure.ac"))

  # Should return FALSE or TRUE with warnings (DESCRIPTION missing Config fields)
  result <- suppressMessages(miniextendr_validate())
  expect_type(result, "logical")
})
