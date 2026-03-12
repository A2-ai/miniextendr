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
# resolve_workspace_cargo_toml tests
# -----------------------------------------------------------------------------

# Helper: create a mock workspace root with Cargo.toml
setup_mock_workspace <- function() {
  ws_root <- tempfile("ws-root-")
  dir.create(ws_root)
  writeLines(c(
    "[workspace]",
    'members = ["my-api", "my-macros"]',
    "",
    "[workspace.package]",
    'version = "0.1.0"',
    'edition = "2024"',
    'license = "MIT"',
    'repository = "https://github.com/example/repo"',
    "",
    "[workspace.dependencies]",
    'my-macros = { version = "*", path = "my-macros" }',
    'linkme = "0.3"',
    'serde = { version = "1.0", features = ["derive"] }'
  ), file.path(ws_root, "Cargo.toml"))
  ws_root
}

test_that("resolve_workspace_cargo_toml handles empty file", {
  ws_root <- setup_mock_workspace()
  tmp <- tempfile(fileext = ".toml")
  on.exit(unlink(c(tmp, ws_root), recursive = TRUE), add = TRUE)

  writeLines(character(), tmp)
  expect_silent(resolve_workspace_cargo_toml(tmp, ws_root))
  expect_equal(readLines(tmp), character())
})

test_that("resolve_workspace_cargo_toml handles file with no workspace entries", {
  ws_root <- setup_mock_workspace()
  tmp <- tempfile(fileext = ".toml")
  on.exit(unlink(c(tmp, ws_root), recursive = TRUE), add = TRUE)

  content <- c(
    '[package]',
    'name = "test-crate"',
    'version = "1.0.0"',
    'edition = "2024"',
    '',
    '[dependencies]',
    'tokio = "1.0"'
  )
  writeLines(content, tmp)
  expect_silent(resolve_workspace_cargo_toml(tmp, ws_root))

  result <- readLines(tmp)
  expect_true(any(grepl('version = "1.0.0"', result)))
  expect_true(any(grepl('tokio = "1.0"', result)))
})

test_that("resolve_workspace_cargo_toml resolves package fields", {
  ws_root <- setup_mock_workspace()
  tmp <- tempfile(fileext = ".toml")
  on.exit(unlink(c(tmp, ws_root), recursive = TRUE), add = TRUE)

  content <- c(
    '[package]',
    'name = "my-api"',
    'edition.workspace = true',
    'version.workspace = true',
    'license.workspace = true'
  )
  writeLines(content, tmp)
  resolve_workspace_cargo_toml(tmp, ws_root)

  result <- readLines(tmp)
  expect_true(any(grepl('edition = "2024"', result)))
  expect_true(any(grepl('version = "0.1.0"', result)))
  expect_true(any(grepl('license = "MIT"', result)))
  expect_false(any(grepl("workspace = true", result)))
})

test_that("resolve_workspace_cargo_toml resolves dependency fields", {
  ws_root <- setup_mock_workspace()
  tmp <- tempfile(fileext = ".toml")
  on.exit(unlink(c(tmp, ws_root), recursive = TRUE), add = TRUE)

  content <- c(
    '[package]',
    'name = "my-api"',
    'version.workspace = true',
    '',
    '[dependencies]',
    'my-macros = { workspace = true }',
    'linkme = { workspace = true }',
    'serde = { workspace = true }'
  )
  writeLines(content, tmp)
  resolve_workspace_cargo_toml(tmp, ws_root)

  result <- readLines(tmp)
  # Workspace member dep gets relative path
  expect_true(any(grepl('my-macros = \\{ version = "\\*", path = "../my-macros" \\}', result)))
  # External dep gets plain version
  expect_true(any(grepl('linkme = "0.3"', result)))
  # External dep with features preserved
  expect_true(any(grepl('serde = \\{ version = "1.0", features', result)))
  expect_false(any(grepl("workspace = true", result)))
})

test_that("resolve_workspace_cargo_toml warns on unresolved entries", {
  ws_root <- setup_mock_workspace()
  tmp <- tempfile(fileext = ".toml")
  on.exit(unlink(c(tmp, ws_root), recursive = TRUE), add = TRUE)

  content <- c(
    '[package]',
    'name = "test-crate"',
    '',
    '[dependencies]',
    'unknown-crate = { workspace = true }'
  )
  writeLines(content, tmp)

  # unknown-crate is not in workspace.dependencies
  expect_warning(resolve_workspace_cargo_toml(tmp, ws_root), "Unhandled workspace")
})

test_that("resolve_workspace_cargo_toml strips dev-deps and bench sections", {
  ws_root <- setup_mock_workspace()
  tmp <- tempfile(fileext = ".toml")
  on.exit(unlink(c(tmp, ws_root), recursive = TRUE), add = TRUE)

  content <- c(
    '[package]',
    'name = "my-api"',
    '',
    '[dependencies]',
    'linkme = { workspace = true }',
    '',
    '[dev-dependencies]',
    'proptest = "1.0"',
    '',
    '[[bench]]',
    'name = "my_bench"',
    'harness = false',
    '',
    '[[test]]',
    'name = "integration"'
  )
  writeLines(content, tmp)
  resolve_workspace_cargo_toml(tmp, ws_root)

  result <- readLines(tmp)
  expect_false(any(grepl("dev-dependencies", result)))
  expect_false(any(grepl("proptest", result)))
  expect_false(any(grepl("\\[\\[bench\\]\\]", result)))
  expect_false(any(grepl("\\[\\[test\\]\\]", result)))
  # Regular deps should still be there
  expect_true(any(grepl('linkme = "0.3"', result)))
})

# -----------------------------------------------------------------------------
# vendor_miniextendr_local failure tests
# -----------------------------------------------------------------------------

test_that("vendor_miniextendr_local fails on missing crate directory", {
  tmp_src <- tempfile("vendor-src-")
  tmp_dest <- tempfile("vendor-dest-")
  on.exit(unlink(c(tmp_src, tmp_dest), recursive = TRUE), add = TRUE)

  # Create source dir with workspace Cargo.toml and only one crate
  dir.create(tmp_src)
  writeLines(c(
    "[workspace]",
    'members = ["miniextendr-api"]',
    "[workspace.package]",
    'version = "0.1.0"',
    'edition = "2024"'
  ), file.path(tmp_src, "Cargo.toml"))
  dir.create(file.path(tmp_src, "miniextendr-api"))
  writeLines('[package]\nname = "miniextendr-api"', file.path(tmp_src, "miniextendr-api", "Cargo.toml"))

  # Should fail because other required crates are missing
  # suppressWarnings: rlang::warn fires for each missing crate before the final error
  expect_error(
    suppressWarnings(suppressMessages(vendor_miniextendr_local(tmp_src, tmp_dest))),
    "Failed to vendor"
  )
})
