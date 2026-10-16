# Tests for the rv.lock / Cargo.lock miniextendr pin check in
# inst/hooks/pre-commit.
#
# A package that follows miniextendr from git on both sides (minirextendr via
# rv, the miniextendr-* crates via cargo) pins the framework in two lock
# files that move independently. The hook blocks a commit whose staged
# rv.lock and src/rust/Cargo.lock name different commits.

skip_if_no_git_or_bash <- function() {
  if (Sys.which("git") == "") testthat::skip("git not available")
  if (Sys.which("bash") == "") testthat::skip("bash not available")
}

# Setup-only git call: a failure here would let a test pass for the wrong
# reason, so stop instead of returning the status.
git <- function(repo, ...) {
  args <- c("-C", repo, ...)
  status <- system2("git", shQuote(args), stdout = FALSE, stderr = FALSE)
  if (status != 0L) stop("git ", paste(args, collapse = " "), " failed")
  invisible(status)
}

sha_r <- "0b2c83d7de47b1ead4ed95dddf238b1c480caeb7"
sha_c <- "43695ea760fec3c7d94fbc390237aba07212e284"

# rv.lock `source` value for minirextendr taken from git.
rv_git_source <- function(sha) {
  sprintf(
    '{ git = "https://github.com/A2-ai/miniextendr.git", sha = "%s", directory = "minirextendr", branch = "main" }',
    sha
  )
}

# An rv.lock with cli and, when `mx_source` is given, a minirextendr entry
# whose `source = ` line is `mx_source`.
rv_lock_lines <- function(mx_source = NULL) {
  lines <- c(
    "version = 2",
    'r_version = "4.6"',
    "",
    "[[packages]]",
    'name = "cli"',
    'version = "3.6.5"',
    'source = { repository = "https://cran.r-project.org/" }',
    "force_source = false"
  )
  if (!is.null(mx_source)) {
    lines <- c(lines,
      "",
      "[[packages]]",
      'name = "minirextendr"',
      'version = "0.2.0"',
      paste("source =", mx_source),
      "force_source = true",
      "dependencies = [",
      '    "cli",',
      '    { name = "usethis", requirement = "(>= 2.0.0)" },',
      "]"
    )
  }
  lines
}

# Cargo.lock `source` value for a miniextendr crate taken from git.
cargo_git_source <- function(sha, ref = "branch=main") {
  sprintf('"git+https://github.com/A2-ai/miniextendr?%s#%s"', ref, sha)
}

# A Cargo.lock with miniextendr-api, miniextendr-lint and the package crate.
# A NULL source omits the line (how cargo records a path dependency).
cargo_lock_lines <- function(api_source, lint_source = api_source) {
  crate <- function(name, source) {
    c("", "[[package]]", sprintf('name = "%s"', name), 'version = "0.1.0"',
      if (!is.null(source)) paste("source =", source))
  }
  c(
    "version = 4",
    crate("miniextendr-api", api_source),
    "dependencies = [",
    ' "miniextendr-macros",',
    "]",
    crate("miniextendr-lint", lint_source),
    crate("testpkg", NULL)
  )
}

# Fake miniextendr package under `pkg_dir` in a fresh git repo, with
# `cargo_lock` as src/rust/Cargo.lock and, when given, `rv_lock` as
# `<rv_dir>/rv.lock`. `pkg_dir = "."` is a standalone package, `"rpkg"` a
# monorepo R package subdirectory. Plain tempfile + dir.create so the dir
# outlives this helper's frame.
make_pin_repo <- function(cargo_lock, rv_lock = NULL, pkg_dir = ".", rv_dir = ".") {
  repo <- tempfile("pin-match-hook-")
  pkg <- file.path(repo, pkg_dir)
  rust <- file.path(pkg, "src", "rust")
  dir.create(rust, recursive = TRUE)
  writeLines(c("Package: testpkg", "Version: 0.1.0"), file.path(pkg, "DESCRIPTION"))
  writeLines(c(
    "[package]", 'name = "testpkg"', 'version = "0.1.0"', 'edition = "2021"'
  ), file.path(rust, "Cargo.toml"))
  writeLines(cargo_lock, file.path(rust, "Cargo.lock"))
  if (!is.null(rv_lock)) {
    writeLines(rv_lock, file.path(repo, rv_dir, "rv.lock"))
  }
  git(repo, "init", "-q")
  # Quiet identity so commits don't fail in CI sandboxes.
  git(repo, "config", "user.email", "test@example.com")
  git(repo, "config", "user.name", "Test")
  repo
}

commit_all <- function(repo) {
  git(repo, "add", "-A")
  git(repo, "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null",
      "commit", "-q", "-m", "init")
}

# Run the bundled hook in `repo` with whatever is currently staged.
run_hook <- function(repo, bash = "bash") {
  hook_path <- system.file("hooks", "pre-commit", package = "minirextendr", mustWork = TRUE)
  withr::with_dir(repo, {
    # The hook exits non-zero on an intentional block; suppress R's warning.
    suppressWarnings(system2(bash, hook_path, stdout = TRUE, stderr = TRUE))
  })
}

hook_status <- function(out) {
  status <- attr(out, "status")
  if (is.null(status)) 0L else status
}

expect_hook_pass <- function(out) {
  expect_identical(hook_status(out), 0L, info = paste(out, collapse = "\n"))
  expect_false(any(grepl("BLOCKED", out, fixed = TRUE)), info = paste(out, collapse = "\n"))
}

expect_pin_mismatch <- function(out, rv_sha = sha_r, cargo_sha = sha_c) {
  text <- paste(out, collapse = "\n")
  expect_identical(hook_status(out), 1L, info = text)
  expect_match(text, "pin different miniextendr commits", fixed = TRUE)
  expect_match(text, rv_sha, fixed = TRUE)
  expect_match(text, cargo_sha, fixed = TRUE)
  expect_match(text, "cargo update -p miniextendr-api", fixed = TRUE)
  expect_match(text, paste("--commit", cargo_sha), fixed = TRUE)
}

test_that("pin check passes without an rv.lock", {
  skip_if_no_git_or_bash()
  repo <- make_pin_repo(cargo_lock_lines(cargo_git_source(sha_c)))
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  git(repo, "add", "-A")

  expect_hook_pass(run_hook(repo))
})

test_that("pin check passes when rv.lock has no minirextendr", {
  skip_if_no_git_or_bash()
  repo <- make_pin_repo(cargo_lock_lines(cargo_git_source(sha_c)), rv_lock_lines())
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  git(repo, "add", "-A")

  expect_hook_pass(run_hook(repo))
})

test_that("pin check passes when minirextendr has a non-git source", {
  skip_if_no_git_or_bash()
  repo <- make_pin_repo(
    cargo_lock_lines(cargo_git_source(sha_c)),
    rv_lock_lines('{ path = "../miniextendr/minirextendr" }')
  )
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  git(repo, "add", "-A")

  expect_hook_pass(run_hook(repo))
})

test_that("pin check passes when both locks name the same commit", {
  skip_if_no_git_or_bash()
  repo <- make_pin_repo(
    cargo_lock_lines(cargo_git_source(sha_r)),
    rv_lock_lines(rv_git_source(sha_r))
  )
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  git(repo, "add", "-A")

  out <- run_hook(repo)
  expect_hook_pass(out)
  expect_true(any(grepl("all miniextendr checks passed", out, fixed = TRUE)))
})

test_that("pin check blocks rv.lock and Cargo.lock naming different commits", {
  skip_if_no_git_or_bash()
  repo <- make_pin_repo(
    cargo_lock_lines(cargo_git_source(sha_c)),
    rv_lock_lines(rv_git_source(sha_r))
  )
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  git(repo, "add", "-A")

  out <- run_hook(repo)
  expect_pin_mismatch(out)
  text <- paste(out, collapse = "\n")
  expect_match(text, sprintf("(cd src/rust && cargo update -p miniextendr-api --precise %s)", sha_r),
               fixed = TRUE)
  expect_match(text, paste0(
    "rv remove minirextendr --no-sync && rv add minirextendr",
    " --git https://github.com/A2-ai/miniextendr.git --commit ", sha_c,
    " --directory minirextendr"
  ), fixed = TRUE)
})

test_that("pin check runs under the system bash", {
  # macOS ships bash 3.2 as /bin/bash; the hook must not need bash 4 features.
  skip_if_no_git_or_bash()
  skip_if_not(file.exists("/bin/bash"), "no /bin/bash")
  mismatch <- make_pin_repo(
    cargo_lock_lines(cargo_git_source(sha_c)),
    rv_lock_lines(rv_git_source(sha_r))
  )
  split <- make_pin_repo(
    cargo_lock_lines(cargo_git_source(sha_r), cargo_git_source(sha_c)),
    rv_lock_lines(rv_git_source(sha_r))
  )
  on.exit(unlink(c(mismatch, split), recursive = TRUE), add = TRUE)
  git(mismatch, "add", "-A")
  git(split, "add", "-A")

  expect_pin_mismatch(run_hook(mismatch, bash = "/bin/bash"))
  out <- run_hook(split, bash = "/bin/bash")
  expect_identical(hook_status(out), 1L, info = paste(out, collapse = "\n"))
  expect_true(any(grepl("pins the miniextendr crates to different commits", out, fixed = TRUE)),
              info = paste(out, collapse = "\n"))
})

test_that("pin check blocks when only rv.lock is staged", {
  # The usual way in: `rv sync` re-resolves a branch pin and rewrites rv.lock
  # while the committed Cargo.lock stays put.
  skip_if_no_git_or_bash()
  repo <- make_pin_repo(
    cargo_lock_lines(cargo_git_source(sha_c)),
    rv_lock_lines(rv_git_source(sha_c))
  )
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  commit_all(repo)
  writeLines(rv_lock_lines(rv_git_source(sha_r)), file.path(repo, "rv.lock"))
  git(repo, "add", "rv.lock")

  expect_pin_mismatch(run_hook(repo))
})

test_that("pin check blocks miniextendr crates pinned to different commits", {
  skip_if_no_git_or_bash()
  repo <- make_pin_repo(
    cargo_lock_lines(cargo_git_source(sha_r), cargo_git_source(sha_c)),
    rv_lock_lines(rv_git_source(sha_r))
  )
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  git(repo, "add", "-A")

  out <- run_hook(repo)
  text <- paste(out, collapse = "\n")
  expect_identical(hook_status(out), 1L, info = text)
  expect_match(text, "pins the miniextendr crates to different commits", fixed = TRUE)
  expect_match(text, paste0("miniextendr-api +", sha_r))
  expect_match(text, paste0("miniextendr-lint +", sha_c))
})

test_that("pin check finds rv.lock at the root of a monorepo", {
  skip_if_no_git_or_bash()
  repo <- make_pin_repo(
    cargo_lock_lines(cargo_git_source(sha_c)),
    rv_lock_lines(rv_git_source(sha_r)),
    pkg_dir = "rpkg"
  )
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  git(repo, "add", "-A")

  out <- run_hook(repo)
  expect_pin_mismatch(out)
  text <- paste(out, collapse = "\n")
  expect_match(text, "rv.lock and rpkg/src/rust/Cargo.lock", fixed = TRUE)
  expect_match(text, "(cd rpkg/src/rust && cargo update", fixed = TRUE)
  # rv.lock is at the root, so the rv command runs there.
  expect_false(grepl("(cd rpkg && rv", text, fixed = TRUE))
})

test_that("pin check passes a monorepo whose locks agree", {
  skip_if_no_git_or_bash()
  repo <- make_pin_repo(
    cargo_lock_lines(cargo_git_source(sha_r)),
    rv_lock_lines(rv_git_source(sha_r)),
    pkg_dir = "rpkg"
  )
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  git(repo, "add", "-A")

  expect_hook_pass(run_hook(repo))
})

test_that("pin check falls back to rv.lock in the monorepo package directory", {
  skip_if_no_git_or_bash()
  repo <- make_pin_repo(
    cargo_lock_lines(cargo_git_source(sha_c)),
    rv_lock_lines(rv_git_source(sha_r)),
    pkg_dir = "rpkg", rv_dir = "rpkg"
  )
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  git(repo, "add", "-A")

  out <- run_hook(repo)
  expect_pin_mismatch(out)
  text <- paste(out, collapse = "\n")
  expect_match(text, "rpkg/rv.lock and rpkg/src/rust/Cargo.lock", fixed = TRUE)
  expect_match(text, "(cd rpkg && rv remove minirextendr", fixed = TRUE)
})

test_that("pin check reads a ?rev= Cargo.lock source and advises editing the rev", {
  skip_if_no_git_or_bash()
  repo <- make_pin_repo(
    cargo_lock_lines(cargo_git_source(sha_c, paste0("rev=", sha_c))),
    rv_lock_lines(rv_git_source(sha_r))
  )
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  git(repo, "add", "-A")

  out <- run_hook(repo)
  expect_pin_mismatch(out)
  text <- paste(out, collapse = "\n")
  # --precise against a rev pin leaves a lock that contradicts Cargo.toml.
  expect_match(text, sprintf('set rev = "%s"', sha_r), fixed = TRUE)
  expect_false(grepl("--precise", text, fixed = TRUE))
})

test_that("pin check passes a ?rev= Cargo.lock at the rv.lock commit", {
  skip_if_no_git_or_bash()
  repo <- make_pin_repo(
    cargo_lock_lines(cargo_git_source(sha_r, paste0("rev=", sha_r))),
    rv_lock_lines(rv_git_source(sha_r))
  )
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  git(repo, "add", "-A")

  expect_hook_pass(run_hook(repo))
})

test_that("pin check passes path- and registry-sourced miniextendr crates", {
  skip_if_no_git_or_bash()
  repo <- make_pin_repo(
    cargo_lock_lines(
      NULL,
      '"registry+https://github.com/rust-lang/crates.io-index"'
    ),
    rv_lock_lines(rv_git_source(sha_r))
  )
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  git(repo, "add", "-A")

  expect_hook_pass(run_hook(repo))
})

test_that("pin check does not run when neither lock is staged", {
  skip_if_no_git_or_bash()
  repo <- make_pin_repo(
    cargo_lock_lines(cargo_git_source(sha_c)),
    rv_lock_lines(rv_git_source(sha_r))
  )
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  # Both mismatched locks are committed (in the index), but the commit being
  # made only touches DESCRIPTION.
  commit_all(repo)
  write("Title: Test", file.path(repo, "DESCRIPTION"), append = TRUE)
  git(repo, "add", "DESCRIPTION")

  expect_hook_pass(run_hook(repo))
})

test_that("pin check reads the staged locks, not the working tree", {
  skip_if_no_git_or_bash()
  repo <- make_pin_repo(
    cargo_lock_lines(cargo_git_source(sha_c)),
    rv_lock_lines(rv_git_source(sha_r))
  )
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  git(repo, "add", "-A")
  # Fixed on disk but not re-staged: the commit would still carry the mismatch.
  writeLines(rv_lock_lines(rv_git_source(sha_c)), file.path(repo, "rv.lock"))

  expect_pin_mismatch(run_hook(repo))
})
