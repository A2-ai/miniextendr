# Tests for vendor and cache functions

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
    download_miniextendr_archive("definitely-not-a-real-version-12345", tempfile()),
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
  expect_silent(patch_cargo_toml(tmp, "test-crate"))

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
  expect_silent(patch_cargo_toml(tmp, "test-crate"))

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
  patch_cargo_toml(tmp, "miniextendr-api")

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
  expect_warning(patch_cargo_toml(tmp, "test-crate"), "Unhandled workspace")
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
    suppressWarnings(suppressMessages(vendor_miniextendr_local(tmp_src, tmp_dest))),
    "Failed to vendor"
  )
})
