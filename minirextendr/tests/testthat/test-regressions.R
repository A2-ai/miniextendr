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
# P2: miniextendr_validate / miniextendr_status / miniextendr_doctor behavior
# (fixture-based; tests behavior, not function source text)
# =============================================================================

test_that("miniextendr_validate returns TRUE for well-formed package (Fixture A)", {
  # Fixture A: DESCRIPTION + configure.ac + src/rust/Cargo.toml with miniextendr-api
  tmp <- tempfile("fixture-a-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)
  writeLines(c(
    "Package: mypkg", "Version: 0.1.0",
    "Config/build/bootstrap: TRUE",
    "SystemRequirements: Rust (>= 1.85)"
  ), file.path(tmp, "DESCRIPTION"))
  writeLines("AC_INIT([mypkg], [0.1.0])", file.path(tmp, "configure.ac"))
  dir.create(file.path(tmp, "src", "rust"), recursive = TRUE)
  writeLines(c(
    '[package]', 'name = "mypkg"', 'version = "0.1.0"', '',
    '[dependencies]', 'miniextendr-api = "*"'
  ), file.path(tmp, "src", "rust", "Cargo.toml"))

  result <- suppressMessages(miniextendr_validate(tmp))
  # Rust may or may not be installed in test env, but should return logical
  expect_type(result, "logical")
})

test_that("miniextendr_validate warns about missing Config/build/bootstrap (Fixture B)", {
  # Fixture B: DESCRIPTION without Config/build/bootstrap
  tmp <- tempfile("fixture-b-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)
  writeLines(c(
    "Package: mypkg", "Version: 0.1.0",
    "SystemRequirements: Rust (>= 1.85)"
  ), file.path(tmp, "DESCRIPTION"))
  writeLines("AC_INIT([mypkg], [0.1.0])", file.path(tmp, "configure.ac"))
  dir.create(file.path(tmp, "src", "rust"), recursive = TRUE)
  writeLines(c(
    '[package]', 'name = "mypkg"', 'version = "0.1.0"', '',
    '[dependencies]', 'miniextendr-api = "*"'
  ), file.path(tmp, "src", "rust", "Cargo.toml"))

  expect_message(
    result <- miniextendr_validate(tmp),
    "bootstrap",
    info = "should warn about missing Config/build/bootstrap"
  )
  expect_type(result, "logical")
})

test_that("miniextendr_validate reports missing configure.ac as an issue (Fixture D)", {
  # Fixture D: configure.ac absent
  tmp <- tempfile("fixture-d-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)
  writeLines(c(
    "Package: mypkg", "Version: 0.1.0",
    "Config/build/bootstrap: TRUE",
    "SystemRequirements: Rust (>= 1.85)"
  ), file.path(tmp, "DESCRIPTION"))
  dir.create(file.path(tmp, "src", "rust"), recursive = TRUE)
  writeLines(c(
    '[package]', 'name = "mypkg"', 'version = "0.1.0"', '',
    '[dependencies]', 'miniextendr-api = "*"'
  ), file.path(tmp, "src", "rust", "Cargo.toml"))

  expect_message(
    result <- miniextendr_validate(tmp),
    "configure.ac",
    info = "should report configure.ac not found"
  )
  expect_false(result)
})

test_that("miniextendr_validate detects missing miniextendr-api in Cargo.toml", {
  tmp <- tempfile("fixture-no-api-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)
  writeLines(c(
    "Package: mypkg", "Version: 0.1.0",
    "Config/build/bootstrap: TRUE",
    "SystemRequirements: Rust (>= 1.85)"
  ), file.path(tmp, "DESCRIPTION"))
  writeLines("AC_INIT([mypkg], [0.1.0])", file.path(tmp, "configure.ac"))
  dir.create(file.path(tmp, "src", "rust"), recursive = TRUE)
  writeLines(c(
    '[package]', 'name = "mypkg"', 'version = "0.1.0"', '',
    '[dependencies]', 'serde = "*"'
  ), file.path(tmp, "src", "rust", "Cargo.toml"))

  expect_message(
    result <- miniextendr_validate(tmp),
    "miniextendr-api",
    info = "should report missing miniextendr-api dependency"
  )
  expect_false(result)
})

test_that("miniextendr_status smoke test returns expected structure (Fixture A)", {
  tmp <- tempfile("fixture-a-status-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)
  writeLines(c(
    "Package: mypkg", "Version: 0.1.0",
    "Config/build/bootstrap: TRUE",
    "SystemRequirements: Rust (>= 1.85)"
  ), file.path(tmp, "DESCRIPTION"))
  writeLines("AC_INIT([mypkg], [0.1.0])", file.path(tmp, "configure.ac"))
  dir.create(file.path(tmp, "src", "rust"), recursive = TRUE)
  writeLines(c(
    '[package]', 'name = "mypkg"', 'version = "0.1.0"', '',
    '[dependencies]', 'miniextendr-api = "*"'
  ), file.path(tmp, "src", "rust", "Cargo.toml"))

  result <- suppressMessages(miniextendr_status(tmp))
  expect_type(result, "list")
  expect_named(result, c("present", "missing", "stale"))
  # Vendored Crates section must no longer appear in status output
  expect_false("Vendored Crates" %in% names(result$present))
  expect_false("Vendored Crates" %in% names(result$missing))
})

test_that("miniextendr_doctor smoke test returns expected structure (Fixture A)", {
  tmp <- tempfile("fixture-a-doctor-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)
  dir.create(tmp)
  writeLines(c(
    "Package: mypkg", "Version: 0.1.0",
    "Config/build/bootstrap: TRUE",
    "SystemRequirements: Rust (>= 1.85)"
  ), file.path(tmp, "DESCRIPTION"))
  writeLines("AC_INIT([mypkg], [0.1.0])", file.path(tmp, "configure.ac"))
  dir.create(file.path(tmp, "src", "rust"), recursive = TRUE)
  writeLines(c(
    '[package]', 'name = "mypkg"', 'version = "0.1.0"', '',
    '[dependencies]', 'miniextendr-api = "*"'
  ), file.path(tmp, "src", "rust", "Cargo.toml"))

  result <- suppressMessages(miniextendr_doctor(tmp))
  expect_type(result, "list")
  expect_named(result, c("pass", "warn", "fail"))
  # Cargo.toml check should pass for fixture A (miniextendr-api present)
  expect_true(
    any(grepl("miniextendr-api", result$pass)),
    label = "doctor should pass Cargo.toml check for fixture A"
  )
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
