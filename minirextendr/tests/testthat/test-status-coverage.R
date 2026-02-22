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

  # Should return TRUE with warnings (DESCRIPTION missing Config fields)
  expect_message(
    result <- miniextendr_validate(tmp),
    "Config/build/bootstrap"
  )
  expect_type(result, "logical")
})

# -----------------------------------------------------------------------------
# miniextendr_status() branch coverage
# -----------------------------------------------------------------------------

test_that("miniextendr_status derives wrapper filename from package name", {
  tmp <- tempfile("status-wrapper-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  writeLines("Package: mypkg\nTitle: Test\nVersion: 0.0.1\n",
    file.path(tmp, "DESCRIPTION"))
  # Create minimal miniextendr-like structure
  dir.create(file.path(tmp, "src", "rust"), recursive = TRUE)
  writeLines("AC_INIT([mypkg])\nMYPKG_FEATURES=\n",
    file.path(tmp, "configure.ac"))
  writeLines('[package]\nname = "mypkg"',
    file.path(tmp, "src", "rust", "Cargo.toml"))

  result <- suppressMessages(miniextendr_status(tmp))

  # Wrapper file should reference package name, not hardcoded "miniextendr"
  all_files <- unlist(c(result$present, result$missing))
  expect_true(any(grepl("mypkg-wrappers\\.R", all_files)))
  expect_false(any(grepl("miniextendr_wrappers\\.R", all_files)))
})

test_that("miniextendr_status detects stale generated files", {
  tmp <- tempfile("status-stale-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  writeLines("Package: testpkg\nTitle: Test\nVersion: 0.0.1\n",
    file.path(tmp, "DESCRIPTION"))

  # Create template and generated files
  dir.create(file.path(tmp, "src"), recursive = TRUE)
  writeLines("template content", file.path(tmp, "src", "Makevars.in"))

  # Wait briefly and create generated file (older timestamp)
  Sys.sleep(0.1)
  writeLines("generated content", file.path(tmp, "src", "Makevars"))

  # Touch template to be newer
  Sys.sleep(0.1)
  writeLines("template content v2", file.path(tmp, "src", "Makevars.in"))

  result <- suppressMessages(miniextendr_status(tmp))
  expect_true("src/Makevars" %in% result$stale)
})

test_that("miniextendr_status reports no staleness when up to date", {
  tmp <- tempfile("status-fresh-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  writeLines("Package: testpkg\nTitle: Test\nVersion: 0.0.1\n",
    file.path(tmp, "DESCRIPTION"))

  # Create template first, then generated file (newer)
  dir.create(file.path(tmp, "src"), recursive = TRUE)
  writeLines("template content", file.path(tmp, "src", "Makevars.in"))
  Sys.sleep(0.1)
  writeLines("generated content", file.path(tmp, "src", "Makevars"))

  result <- suppressMessages(miniextendr_status(tmp))
  expect_equal(length(result$stale), 0)
})

# -----------------------------------------------------------------------------
# miniextendr_validate() branch coverage
# -----------------------------------------------------------------------------

test_that("miniextendr_validate returns FALSE when DESCRIPTION is missing", {
  tmp <- tempfile("validate-nodesc-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  # No DESCRIPTION at all
  expect_message(
    result <- miniextendr_validate(tmp),
    "DESCRIPTION not found"
  )
  expect_false(result)
})

test_that("miniextendr_validate returns FALSE when configure.ac is missing", {
  tmp <- tempfile("validate-noconf-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  writeLines(paste0(
    "Package: testpkg\nTitle: Test\nVersion: 0.0.1\n",
    "Config/build/bootstrap: TRUE\n",
    "SystemRequirements: Rust (>= 1.85)\n"
  ), file.path(tmp, "DESCRIPTION"))

  expect_message(
    result <- miniextendr_validate(tmp),
    "configure.ac not found"
  )
  expect_false(result)
})

test_that("miniextendr_validate warns when SystemRequirements lacks Rust", {
  tmp <- tempfile("validate-nosysreq-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  writeLines(paste0(
    "Package: testpkg\nTitle: Test\nVersion: 0.0.1\n",
    "Config/build/bootstrap: TRUE\n"
  ), file.path(tmp, "DESCRIPTION"))

  writeLines("AC_INIT([testpkg], [0.0.1])\nTESTPKG_FEATURES=\n",
    file.path(tmp, "configure.ac"))

  expect_message(
    result <- miniextendr_validate(tmp),
    "SystemRequirements"
  )
  # Should still return TRUE (warning, not error)
  expect_true(result)
})

test_that("miniextendr_validate warns on AC_INIT mismatch", {
  tmp <- tempfile("validate-acinit-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  writeLines(paste0(
    "Package: testpkg\nTitle: Test\nVersion: 0.0.1\n",
    "Config/build/bootstrap: TRUE\n",
    "SystemRequirements: Rust (>= 1.85)\n"
  ), file.path(tmp, "DESCRIPTION"))

  # AC_INIT with wrong package name
  writeLines("AC_INIT([wrongpkg], [0.0.1])\nWRONGPKG_FEATURES=\n",
    file.path(tmp, "configure.ac"))

  expect_message(
    result <- miniextendr_validate(tmp),
    "AC_INIT"
  )
  expect_type(result, "logical")
})

test_that("miniextendr_validate warns on missing vendored crates", {
  tmp <- tempfile("validate-novendor-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)

  writeLines(paste0(
    "Package: testpkg\nTitle: Test\nVersion: 0.0.1\n",
    "Config/build/bootstrap: TRUE\n",
    "SystemRequirements: Rust (>= 1.85)\n"
  ), file.path(tmp, "DESCRIPTION"))

  writeLines("AC_INIT([testpkg], [0.0.1])\nTESTPKG_FEATURES=\n",
    file.path(tmp, "configure.ac"))

  # No vendor/ directory at all
  expect_message(
    result <- miniextendr_validate(tmp),
    "Missing vendored crates"
  )
  expect_type(result, "logical")
})
