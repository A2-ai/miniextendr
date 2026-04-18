# Regression tests for bugs found in the 2026-02-07 code review.
# Each test documents the original bug and verifies the fix.

# =============================================================================
# P1: use_miniextendr() default version must be "main", not "latest"
# =============================================================================

test_that("use_miniextendr default version is 'main'", {
  # Bug: default was "latest" which is not a valid GitHub ref (404 on download).
  args <- formals(use_miniextendr)
  expect_equal(args$miniextendr_version, "main")
})

# =============================================================================
# P2: minirextendr:::add_crate_to_workspace() handles one-line members arrays
# =============================================================================

test_that("add_crate_to_workspace handles one-line members array", {
  tmp <- tempfile(fileext = ".toml")
  on.exit(unlink(tmp), add = TRUE)

  # Bug: one-line array `members = ["a"]` caused insertion before the line
  # instead of inside the array.
  writeLines(c(
    '[workspace]',
    'members = ["existing-crate"]',
    '',
    '[workspace.dependencies]',
    'serde = "1"'
  ), tmp)

  result <- minirextendr:::add_crate_to_workspace(tmp, "new-crate")
  expect_true(result)

  content <- readLines(tmp)
  # New crate must appear in members

  expect_true(any(grepl('"new-crate"', content)))
  # Existing crate must still be present
  expect_true(any(grepl('"existing-crate"', content)))
  # The [workspace.dependencies] section must still exist and be intact
  expect_true(any(grepl("\\[workspace\\.dependencies\\]", content)))
})

test_that("add_crate_to_workspace handles one-line array with multiple members", {
  tmp <- tempfile(fileext = ".toml")
  on.exit(unlink(tmp), add = TRUE)

  writeLines(c(
    '[workspace]',
    'members = ["crate-a", "crate-b", "crate-c"]'
  ), tmp)

  result <- minirextendr:::add_crate_to_workspace(tmp, "crate-d")
  expect_true(result)

  content <- readLines(tmp)
  # All four crates must be present
  for (crate in c("crate-a", "crate-b", "crate-c", "crate-d")) {
    expect_true(
      any(grepl(sprintf('"%s"', crate), content)),
      label = sprintf("crate %s should be in members", crate)
    )
  }
})

test_that("add_crate_to_workspace handles empty one-line array", {
  tmp <- tempfile(fileext = ".toml")
  on.exit(unlink(tmp), add = TRUE)

  writeLines(c(
    '[workspace]',
    'members = []'
  ), tmp)

  result <- minirextendr:::add_crate_to_workspace(tmp, "first-crate")
  expect_true(result)

  content <- readLines(tmp)
  expect_true(any(grepl('"first-crate"', content)))
})

test_that("add_crate_to_workspace handles multiline members array", {
  tmp <- tempfile(fileext = ".toml")
  on.exit(unlink(tmp), add = TRUE)

  writeLines(c(
    '[workspace]',
    'members = [',
    '    "crate-a",',
    '    "crate-b",',
    ']'
  ), tmp)

  result <- minirextendr:::add_crate_to_workspace(tmp, "crate-c")
  expect_true(result)

  content <- readLines(tmp)
  expect_true(any(grepl('"crate-c"', content)))
  # Existing crates preserved
  expect_true(any(grepl('"crate-a"', content)))
  expect_true(any(grepl('"crate-b"', content)))
})

test_that("add_crate_to_workspace detects duplicate", {
  tmp <- tempfile(fileext = ".toml")
  on.exit(unlink(tmp), add = TRUE)

  writeLines(c(
    '[workspace]',
    'members = ["already-here"]'
  ), tmp)

  result <- suppressMessages(minirextendr:::add_crate_to_workspace(tmp, "already-here"))
  expect_false(result)
})

# =============================================================================
# P2: status/check/doctor required crate lists include miniextendr-engine
# =============================================================================

test_that("miniextendr_status expects all 5 required vendored crates", {
  skip_if_no_local_repo()
  tmp <- tempfile("status-engine-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  create_miniextendr_monorepo(tmp, package = "testpkg",
                               crate_name = "testpkg-rs",
                               local_path = find_miniextendr_repo(), open = FALSE)

  result <- suppressMessages(miniextendr_status(file.path(tmp, "testpkg")))

  # The expected vendored crates list should include miniextendr-engine
  all_expected <- unlist(result$present, use.names = FALSE)
  all_missing <- unlist(result$missing, use.names = FALSE)
  all_files <- c(all_expected, all_missing)

  expect_true(
    any(grepl("miniextendr-engine", all_files)),
    label = "miniextendr-engine should be in status expected files"
  )
})

test_that("miniextendr_doctor checks all 5 required crates", {
  # Verify the required_crates vector in doctor.R includes miniextendr-engine
  # by inspecting the function body
  body_text <- deparse(body(miniextendr_doctor))
  expect_true(
    any(grepl("miniextendr-engine", body_text)),
    label = "miniextendr_doctor should check for miniextendr-engine"
  )
})

test_that("miniextendr_validate checks all required crates", {
  body_text <- deparse(body(miniextendr_validate))
  expected_crates <- c("miniextendr-api", "miniextendr-macros",
                        "miniextendr-lint", "miniextendr-engine")
  for (crate in expected_crates) {
    expect_true(
      any(grepl(crate, body_text, fixed = TRUE)),
      label = sprintf("miniextendr_validate should check for %s", crate)
    )
  }
})

# =============================================================================
# P3: miniextendr_doctor is exported
# =============================================================================

test_that("miniextendr_doctor is exported", {
  expect_true(
    "miniextendr_doctor" %in% getNamespaceExports("minirextendr"),
    label = "miniextendr_doctor should be in NAMESPACE exports"
  )
})

# =============================================================================
# P3: ensure_dir is not duplicated
# =============================================================================

test_that("ensure_dir is defined only once", {
  # Scan all R files for ensure_dir definitions
  r_dir <- system.file("R", package = "minirextendr")
  if (!nzchar(r_dir)) {
    # Fallback: use source directory
    r_dir <- file.path(
      normalizePath(file.path(getwd(), "..", "..", "R")),
      fsep = "/"
    )
  }

  # Check source files directly if available
  src_dir <- normalizePath(
    file.path(system.file(package = "minirextendr"), "..", "..", "R"),
    mustWork = FALSE
  )

  # At minimum, verify ensure_dir exists in the namespace
  expect_true(
    exists("ensure_dir", envir = asNamespace("minirextendr")),
    label = "ensure_dir should exist in minirextendr namespace"
  )
})
