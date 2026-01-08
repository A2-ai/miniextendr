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
