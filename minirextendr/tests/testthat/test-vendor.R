# Tests for vendor and cache functions

# -----------------------------------------------------------------------------
# read_workspace_package_values (#253)
# -----------------------------------------------------------------------------

test_that("read_workspace_package_values returns defaults when workspace_root is NULL", {
  vals <- minirextendr:::read_workspace_package_values(NULL)
  expect_equal(vals$edition, "2024")
  expect_equal(vals$version, "0.1.0")
  expect_equal(vals$license, "MIT")
})

test_that("read_workspace_package_values reads string fields from workspace Cargo.toml", {
  tmp <- withr::local_tempdir()
  writeLines(c(
    "[workspace.package]",
    'edition = "2021"',
    'version = "9.9.9"',
    'license = "Apache-2.0"',
    'repository = "https://example.com/repo"',
    'homepage = "https://example.com/home"',
    'keywords = ["a", "b"]',
    'categories = ["x"]',
    "",
    "[workspace]",
    "members = []"
  ), fs::path(tmp, "Cargo.toml"))

  vals <- minirextendr:::read_workspace_package_values(tmp)
  expect_equal(vals$edition, "2021")
  expect_equal(vals$version, "9.9.9")
  expect_equal(vals$license, "Apache-2.0")
  expect_equal(vals$repository, "https://example.com/repo")
  expect_equal(vals$homepage, "https://example.com/home")
  expect_equal(vals$keywords, '["a", "b"]')
  expect_equal(vals$categories, '["x"]')
})

test_that("read_workspace_package_values falls back per-field when fields are missing", {
  tmp <- withr::local_tempdir()
  # Only declare `version` — everything else must fall back to defaults.
  writeLines(c(
    "[workspace.package]",
    'version = "7.7.7"',
    "",
    "[workspace]",
    "members = []"
  ), fs::path(tmp, "Cargo.toml"))

  vals <- minirextendr:::read_workspace_package_values(tmp)
  expect_equal(vals$version, "7.7.7")       # from file
  expect_equal(vals$edition, "2024")        # default
  expect_equal(vals$license, "MIT")         # default
  expect_equal(vals$keywords, '["r", "ffi", "bindings"]')  # default
})

test_that("read_workspace_package_values warns when workspace Cargo.toml is missing", {
  tmp <- withr::local_tempdir()
  expect_warning(
    vals <- minirextendr:::read_workspace_package_values(tmp),
    "Workspace Cargo.toml not found"
  )
  expect_equal(vals$edition, "2024")  # defaults used
})

test_that("read_workspace_package_values handles no [workspace.package] section", {
  tmp <- withr::local_tempdir()
  writeLines(c(
    "[package]",
    'name = "x"',
    'version = "1.0.0"'
  ), fs::path(tmp, "Cargo.toml"))

  vals <- minirextendr:::read_workspace_package_values(tmp)
  # No [workspace.package] → defaults returned, no warning.
  expect_equal(vals$edition, "2024")
})

# Helper to skip tests that require network
skip_if_offline <- function() {
  tryCatch(
    {
      con <- url("https://api.github.com", open = "r")
      close(con)
    },
    error = function(e) {
      skip("No internet connection available")
    }
  )
}

# -----------------------------------------------------------------------------
# Cache management tests
# -----------------------------------------------------------------------------

test_that("miniextendr_cache_info returns data frame", {
  # Should not error even if cache is empty
  result <- suppressMessages(miniextendr_cache_info())
  expect_true(is.data.frame(result) || is.null(dim(result)))
})

test_that("miniextendr_clear_cache handles empty cache", {
  # Should not error if cache doesn't exist
  expect_invisible(suppressMessages(miniextendr_clear_cache()))
})

test_that("miniextendr_clear_cache with version handles missing version",
{
    # Should not error for non-existent version
    expect_invisible(suppressMessages(miniextendr_clear_cache("nonexistent-version-xyz")))
  })

# -----------------------------------------------------------------------------
# Version listing tests
# -----------------------------------------------------------------------------

test_that("miniextendr_available_versions returns character vector", {
  skip_if_offline()

  result <- suppressMessages(miniextendr_available_versions())
  expect_type(result, "character")
  expect_true(length(result) >= 1)
})

# -----------------------------------------------------------------------------
# Helper function tests
# -----------------------------------------------------------------------------

test_that("download_miniextendr_archive validates version parameter", {
  skip_if_offline()

  # Invalid version should error
  expect_error(
    minirextendr:::download_miniextendr_archive("definitely-not-a-real-version-12345", tempfile()),
    "Failed to download"
  )
})

# -----------------------------------------------------------------------------
# patch_cargo_toml tests
# -----------------------------------------------------------------------------

test_that("patch_cargo_toml handles empty file", {
  tmp <- tempfile(fileext = ".toml")
  on.exit(unlink(tmp), add = TRUE)

  writeLines(character(), tmp)
  # Should not error on empty file
  expect_silent(minirextendr:::patch_cargo_toml(tmp, "test-crate"))

  # File should still be empty
  expect_equal(readLines(tmp), character())
})

test_that("patch_cargo_toml handles file with no workspace entries", {
  tmp <- tempfile(fileext = ".toml")
  on.exit(unlink(tmp), add = TRUE)

  content <- c(
    '[package]',
    'name = "test-crate"',
    'version = "1.0.0"',
    'edition = "2024"',
    '',
    '[dependencies]',
    'serde = "1.0"'
  )
  writeLines(content, tmp)
  expect_silent(minirextendr:::patch_cargo_toml(tmp, "test-crate"))

  result <- readLines(tmp)
  # Content should be unchanged
  expect_true(any(grepl('version = "1.0.0"', result)))
  expect_true(any(grepl('serde = "1.0"', result)))
})

test_that("patch_cargo_toml replaces known workspace entries", {
  tmp <- tempfile(fileext = ".toml")
  on.exit(unlink(tmp), add = TRUE)

  content <- c(
    '[package]',
    'name = "miniextendr-api"',
    'edition.workspace = true',
    'version.workspace = true',
    'license.workspace = true',
    '',
    '[dependencies]',
    'miniextendr-macros = { workspace = true }'
  )
  writeLines(content, tmp)
  minirextendr:::patch_cargo_toml(tmp, "miniextendr-api")

  result <- readLines(tmp)
  expect_true(any(grepl('edition = "2024"', result)))
  expect_true(any(grepl('version = "0.1.0"', result)))
  expect_true(any(grepl('license = "MIT"', result)))
  expect_true(any(grepl('miniextendr-macros = \\{ version = "0.1.0"', result)))
  # No workspace = true should remain
  expect_false(any(grepl("workspace = true", result)))
})

test_that("patch_cargo_toml warns on unhandled workspace entries", {
  tmp <- tempfile(fileext = ".toml")
  on.exit(unlink(tmp), add = TRUE)

  content <- c(
    '[package]',
    'name = "test-crate"',
    'edition.workspace = true',
    '',
    '[dependencies]',
    'unknown-crate = { workspace = true }'
  )
  writeLines(content, tmp)

  # edition.workspace is handled, but unknown-crate workspace = true is not
  expect_warning(minirextendr:::patch_cargo_toml(tmp, "test-crate"), "Unhandled workspace")
})

# -----------------------------------------------------------------------------
# vendor_miniextendr_local failure tests
# -----------------------------------------------------------------------------

test_that("vendor_miniextendr_local fails on missing crate directory", {
  tmp_src <- tempfile("vendor-src-")
  tmp_dest <- tempfile("vendor-dest-")
  on.exit(unlink(c(tmp_src, tmp_dest), recursive = TRUE), add = TRUE)

  # Create source dir with only one crate
  dir.create(tmp_src)
  dir.create(file.path(tmp_src, "miniextendr-api"))
  writeLines('[package]\nname = "miniextendr-api"', file.path(tmp_src, "miniextendr-api", "Cargo.toml"))

  # Should fail because other required crates are missing
  # suppressWarnings: rlang::warn fires for each missing crate before the final error
  expect_error(
    suppressWarnings(suppressMessages(minirextendr:::vendor_miniextendr_local(tmp_src, tmp_dest))),
    "Failed to vendor"
  )
})
