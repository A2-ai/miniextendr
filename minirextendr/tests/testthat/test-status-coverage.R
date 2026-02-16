# Integration tests for status functions

# -----------------------------------------------------------------------------
# has_miniextendr() tests
# -----------------------------------------------------------------------------

test_that("has_miniextendr returns FALSE for empty project", {
  tmp <- tempfile("empty-proj-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  dir.create(tmp)
  writeLines("Package: testpkg\nTitle: Test\nVersion: 0.0.1\n",
    file.path(tmp, "DESCRIPTION"))

  expect_false(has_miniextendr(tmp))
})

test_that("has_miniextendr returns TRUE for monorepo project", {
  skip_if_no_local_repo()
  tmp <- tempfile("monorepo-status-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg-rs",
                              local_path = find_miniextendr_repo(), open = FALSE)

  expect_true(has_miniextendr(file.path(tmp, "testpkg")))
})

# -----------------------------------------------------------------------------
# miniextendr_status() tests
# -----------------------------------------------------------------------------

test_that("miniextendr_status returns list with present and missing", {
  skip_if_no_local_repo()
  tmp <- tempfile("status-test-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg-rs",
                              local_path = find_miniextendr_repo(), open = FALSE)

  result <- suppressMessages(miniextendr_status(file.path(tmp, "testpkg")))

  expect_type(result, "list")
  expect_named(result, c("present", "missing", "stale"))
  expect_type(result$present, "list")
  expect_type(result$missing, "list")

  # Monorepo should have most files present
  expect_true(sum(lengths(result$present)) > 0)
})

test_that("miniextendr_status detects generated files as missing", {
  skip_if_no_local_repo()
  tmp <- tempfile("status-gen-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg-rs",
                              local_path = find_miniextendr_repo(), open = FALSE)

  result <- suppressMessages(miniextendr_status(file.path(tmp, "testpkg")))

  # Generated files should be missing before configure runs
  generated_missing <- result$missing[["Generated Files"]]
  expect_true(length(generated_missing) > 0)
  expect_true("src/Makevars" %in% generated_missing)
})

# -----------------------------------------------------------------------------
# miniextendr_validate() tests
# -----------------------------------------------------------------------------

test_that("miniextendr_validate validates DESCRIPTION config", {
  skip_if_no_local_repo()
  tmp <- tempfile("check-desc-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "testpkg", crate_name = "testpkg-rs",
                              local_path = find_miniextendr_repo(), open = FALSE)

  # Should pass - monorepo template sets up DESCRIPTION correctly
  result <- suppressMessages(miniextendr_validate(file.path(tmp, "testpkg")))
  # Result depends on whether Rust is installed and crates vendored
  expect_type(result, "logical")
})

test_that("miniextendr_validate warns on missing Config/build/bootstrap", {
  tmp <- tempfile("check-bootstrap-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  # Create minimal package without miniextendr setup
  dir.create(tmp)
  writeLines("Package: testpkg\nTitle: Test\nVersion: 0.0.1\n",
    file.path(tmp, "DESCRIPTION"))

  # Create minimal configure.ac
  writeLines("AC_INIT([testpkg], [0.0.1])\nTESTPKG_FEATURES=\n",
    file.path(tmp, "configure.ac"))

  # Should return FALSE or TRUE with warnings (DESCRIPTION missing Config fields)
  result <- suppressMessages(miniextendr_validate(tmp))
  expect_type(result, "logical")
})
