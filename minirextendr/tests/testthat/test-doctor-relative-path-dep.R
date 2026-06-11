# Tests for the relative-path-dep check in miniextendr_doctor() (#894).
#
# The check warns when [dependencies] in src/rust/Cargo.toml contains a
# `path = "..."` value that is relative (does not start with "/").  It must
# NOT warn for [patch.crates-io] entries.

# ---------------------------------------------------------------------------
# Unit tests for the parse_relative_path_deps() helper
# ---------------------------------------------------------------------------

test_that("parse_relative_path_deps detects inline relative path dep", {
  lines <- c(
    '[package]',
    'name = "mypkg"',
    '',
    '[dependencies]',
    'my-crate = { path = "../my-crate" }'
  )
  result <- minirextendr:::parse_relative_path_deps(lines)
  expect_length(result, 1L)
  expect_equal(result[[1L]]$crate, "my-crate")
  expect_equal(result[[1L]]$path,  "../my-crate")
})

test_that("parse_relative_path_deps detects multi-line relative path dep", {
  lines <- c(
    '[package]',
    'name = "mypkg"',
    '',
    '[dependencies.my-crate]',
    'path = "../my-crate"'
  )
  result <- minirextendr:::parse_relative_path_deps(lines)
  expect_length(result, 1L)
  expect_equal(result[[1L]]$crate, "my-crate")
  expect_equal(result[[1L]]$path,  "../my-crate")
})

test_that("parse_relative_path_deps does NOT flag absolute paths", {
  lines <- c(
    '[package]',
    'name = "mypkg"',
    '',
    '[dependencies]',
    'my-crate = { path = "/home/user/my-crate" }'
  )
  result <- minirextendr:::parse_relative_path_deps(lines)
  expect_length(result, 0L)
})

test_that("parse_relative_path_deps does NOT flag [patch.crates-io] entries", {
  lines <- c(
    '[package]',
    'name = "mypkg"',
    '',
    '[dependencies]',
    'miniextendr-api = "*"',
    '',
    '[patch.crates-io]',
    'my-crate = { path = "../my-crate" }'
  )
  result <- minirextendr:::parse_relative_path_deps(lines)
  expect_length(result, 0L)
})

test_that("parse_relative_path_deps handles mixed absolute + relative deps", {
  lines <- c(
    '[package]',
    'name = "mypkg"',
    '',
    '[dependencies]',
    'abs-crate = { path = "/usr/local/abs-crate" }',
    'rel-crate = { path = "../rel-crate" }'
  )
  result <- minirextendr:::parse_relative_path_deps(lines)
  expect_length(result, 1L)
  expect_equal(result[[1L]]$crate, "rel-crate")
})

test_that("parse_relative_path_deps returns empty list for no path deps", {
  lines <- c(
    '[package]',
    'name = "mypkg"',
    '',
    '[dependencies]',
    'serde = { version = "1.0", features = ["derive"] }',
    'miniextendr-api = "*"'
  )
  result <- minirextendr:::parse_relative_path_deps(lines)
  expect_length(result, 0L)
})

# ---------------------------------------------------------------------------
# Integration tests via miniextendr_doctor()
# ---------------------------------------------------------------------------

# Build a minimal fake R package directory suitable for doctor().
make_doctor_pkg <- function(cargo_lines) {
  tmp <- tempfile("doctor-rel-path-")
  dir.create(tmp)
  writeLines(c(
    "Package: mypkg",
    "Version: 0.1.0",
    "Config/build/bootstrap: TRUE",
    "SystemRequirements: Rust (>= 1.85)"
  ), file.path(tmp, "DESCRIPTION"))
  writeLines("AC_INIT([mypkg], [0.1.0])", file.path(tmp, "configure.ac"))
  rust_dir <- file.path(tmp, "src", "rust")
  dir.create(rust_dir, recursive = TRUE)
  writeLines(cargo_lines, file.path(rust_dir, "Cargo.toml"))
  tmp
}

test_that("miniextendr_doctor warns on relative path dep in [dependencies]", {
  tmp <- make_doctor_pkg(c(
    '[package]',
    'name = "mypkg"',
    'version = "0.1.0"',
    '',
    '[dependencies]',
    'miniextendr-api = "*"',
    'my-local = { path = "../my-local" }'
  ))
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  result <- suppressMessages(miniextendr_doctor(tmp))

  expect_true(
    any(grepl("relative path dep in \\[dependencies\\]", result$warn)),
    label = "doctor should warn about relative [dependencies] path dep"
  )
  expect_true(
    any(grepl("my-local", result$warn)),
    label = "warning should name the offending crate"
  )
})

test_that("miniextendr_doctor does NOT warn on relative path in [patch.crates-io]", {
  tmp <- make_doctor_pkg(c(
    '[package]',
    'name = "mypkg"',
    'version = "0.1.0"',
    '',
    '[dependencies]',
    'miniextendr-api = "*"',
    '',
    '[patch.crates-io]',
    'miniextendr-api = { path = "../miniextendr-api" }'
  ))
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  result <- suppressMessages(miniextendr_doctor(tmp))

  expect_false(
    any(grepl("relative path dep in \\[dependencies\\]", result$warn)),
    label = "doctor must NOT warn for [patch.crates-io] relative paths"
  )
})

test_that("miniextendr_doctor passes cleanly when no relative path deps exist", {
  tmp <- make_doctor_pkg(c(
    '[package]',
    'name = "mypkg"',
    'version = "0.1.0"',
    '',
    '[dependencies]',
    'miniextendr-api = "*"',
    'serde = { version = "1.0", features = ["derive"] }'
  ))
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  result <- suppressMessages(miniextendr_doctor(tmp))

  expect_true(
    any(grepl("no relative path deps in \\[dependencies\\]", result$pass)),
    label = "doctor should pass the relative-path check when no relative deps"
  )
})
