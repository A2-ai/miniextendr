# Tests for the Cargo.lock-shape check in inst/hooks/pre-commit.

# Run the bundled pre-commit hook against a fake repo + staged file set.
# Returns the system2 exit status. The hook reads its staged-file list from
# `git diff --cached --name-only --diff-filter=ACM`, so we shell out to a
# real `git` in a tempdir.
run_hook_in_repo <- function(repo, staged) {
  hook_path <- system.file("hooks", "pre-commit", package = "minirextendr", mustWork = TRUE)
  withr::with_dir(repo, {
    system2("git", c("init", "-q"))
    # Quiet identity so commits don't fail in CI sandboxes.
    system2("git", c("config", "user.email", "test@example.com"))
    system2("git", c("config", "user.name", "Test"))
    system2("git", c("add", staged), stdout = FALSE, stderr = FALSE)
    # Hook returns non-zero on intentional block; suppress R's warning for
    # that case so testthat doesn't flag every blocked-commit case.
    suppressWarnings(system2("bash", hook_path, stdout = TRUE, stderr = TRUE))
  })
}

# Build a fake R-package layout: DESCRIPTION + src/rust/Cargo.toml +
# src/rust/Cargo.lock with the requested content.
# NOTE: must use plain tempfile + dir.create so the dir survives past this
# helper's frame (withr::local_tempdir defaults scope cleanup here, deleting
# the dir before the test uses it).
make_lock_repo <- function(lock_lines) {
  repo <- tempfile("lock-shape-hook-")
  dir.create(repo)
  writeLines("Package: testpkg\nVersion: 0.1.0\n", file.path(repo, "DESCRIPTION"))
  rust <- file.path(repo, "src", "rust")
  dir.create(rust, recursive = TRUE)
  writeLines(c(
    '[package]', 'name = "testpkg"', 'version = "0.1.0"', 'edition = "2021"'
  ), file.path(rust, "Cargo.toml"))
  writeLines(lock_lines, file.path(rust, "Cargo.lock"))
  repo
}

skip_if_no_git_or_bash <- function() {
  if (Sys.which("git") == "") testthat::skip("git not available")
  if (Sys.which("bash") == "") testthat::skip("bash not available")
}

test_that("pre-commit hook accepts a tarball-shape Cargo.lock", {
  skip_if_no_git_or_bash()
  repo <- make_lock_repo(c(
    'version = 3',
    '',
    '[[package]]',
    'name = "miniextendr-api"',
    'version = "0.1.0"',
    'source = "git+https://github.com/A2-ai/miniextendr#abc123"'
  ))
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)

  out <- run_hook_in_repo(repo, "src/rust/Cargo.lock")
  status <- attr(out, "status")
  # Hook returns 0 (and prints "all miniextendr checks passed.") on success.
  expect_true(is.null(status) || status == 0L,
              info = paste(out, collapse = "\n"))
})

test_that("pre-commit hook blocks a path+ source entry", {
  skip_if_no_git_or_bash()
  repo <- make_lock_repo(c(
    'version = 3',
    '',
    '[[package]]',
    'name = "miniextendr-api"',
    'version = "0.1.0"',
    'source = "path+file:///home/dev/miniextendr/miniextendr-api"'
  ))
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)

  out <- run_hook_in_repo(repo, "src/rust/Cargo.lock")
  status <- attr(out, "status")
  expect_true(!is.null(status) && status == 1L,
              info = paste(out, collapse = "\n"))
  expect_true(any(grepl("source-shape|path\\+", out)),
              info = paste(out, collapse = "\n"))
  expect_true(any(grepl("miniextendr_repair_lock", out)),
              info = paste(out, collapse = "\n"))
})

test_that("pre-commit hook accepts a checksum line", {
  # Post-#408: cargo-revendor recomputes valid .cargo-checksum.json files, so
  # `checksum = "..."` lines are canonical and the hook no longer blocks them.
  skip_if_no_git_or_bash()
  repo <- make_lock_repo(c(
    'version = 3',
    '',
    '[[package]]',
    'name = "libc"',
    'version = "0.2.150"',
    'source = "registry+https://github.com/rust-lang/crates.io-index"',
    'checksum = "89d92a4743f9a61002fae18374ed11e7973f530cb3b3e0b4b63760b6d924afb5"'
  ))
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)

  out <- run_hook_in_repo(repo, "src/rust/Cargo.lock")
  status <- attr(out, "status")
  expect_true(is.null(status) || status == 0L,
              info = paste(out, collapse = "\n"))
})

test_that("pre-commit hook blocks a [[patch.unused]] block", {
  skip_if_no_git_or_bash()
  repo <- make_lock_repo(c(
    'version = 3',
    '',
    '[[package]]',
    'name = "miniextendr-api"',
    'version = "0.1.0"',
    'source = "git+https://github.com/A2-ai/miniextendr#abc123"',
    '',
    '[[patch.unused]]',
    'name = "miniextendr-api"',
    'version = "0.1.0"',
    'source = "git+https://github.com/A2-ai/miniextendr#abc123"'
  ))
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)

  out <- run_hook_in_repo(repo, "src/rust/Cargo.lock")
  status <- attr(out, "status")
  expect_true(!is.null(status) && status == 1L,
              info = paste(out, collapse = "\n"))
  expect_true(any(grepl("patch.unused|narrow", out)),
              info = paste(out, collapse = "\n"))
})

test_that("pre-commit hook does not check Cargo.lock when it is not staged", {
  skip_if_no_git_or_bash()
  # Lock has path+ but is not staged — hook should pass (it inspects only
  # what's in the diff).
  repo <- make_lock_repo(c(
    'version = 3',
    '',
    '[[package]]',
    'name = "miniextendr-api"',
    'version = "0.1.0"',
    'source = "path+file:///home/dev/miniextendr/miniextendr-api"'
  ))
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  # Stage DESCRIPTION instead, leaving Cargo.lock unstaged.
  out <- run_hook_in_repo(repo, "DESCRIPTION")
  status <- attr(out, "status")
  expect_true(is.null(status) || status == 0L,
              info = paste(out, collapse = "\n"))
})
