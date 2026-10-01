# Tests for the staged-vendor-tarball block in inst/hooks/pre-commit.
#
# inst/vendor.tar.xz is a build artifact (miniextendr_build_tarball() vendors
# it into the release tarball). The hook used to print a "was not updated" reminder
# whenever Rust sources changed without it, which nudged downstream packages
# into committing a fresh multi-MiB tarball on every Rust edit. It now blocks
# any staged vendor.tar.xz path and prints the `git rm --cached` fix.

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

# Fake miniextendr package (DESCRIPTION + src/rust/Cargo.toml) under
# `pkg_dir` inside a fresh git repo. `pkg_dir = "."` is a standalone package,
# `"rpkg"` a monorepo R package subdirectory. Plain tempfile + dir.create so
# the dir outlives this helper's frame.
make_vendor_repo <- function(pkg_dir = ".") {
  repo <- tempfile("vendor-hook-")
  pkg <- file.path(repo, pkg_dir)
  dir.create(file.path(pkg, "src", "rust"), recursive = TRUE)
  dir.create(file.path(pkg, "inst"))
  writeLines(c("Package: testpkg", "Version: 0.1.0"), file.path(pkg, "DESCRIPTION"))
  writeLines(c(
    '[package]', 'name = "testpkg"', 'version = "0.1.0"', 'edition = "2021"'
  ), file.path(pkg, "src", "rust", "Cargo.toml"))
  git(repo, "init", "-q")
  # Quiet identity so commits don't fail in CI sandboxes.
  git(repo, "config", "user.email", "test@example.com")
  git(repo, "config", "user.name", "Test")
  repo
}

# Run the bundled hook in `repo` with whatever is currently staged.
run_hook <- function(repo) {
  hook_path <- system.file("hooks", "pre-commit", package = "minirextendr", mustWork = TRUE)
  withr::with_dir(repo, {
    # The hook exits non-zero on an intentional block; suppress R's warning.
    suppressWarnings(system2("bash", hook_path, stdout = TRUE, stderr = TRUE))
  })
}

hook_status <- function(out) {
  status <- attr(out, "status")
  if (is.null(status)) 0L else status
}

test_that("pre-commit hook blocks a staged inst/vendor.tar.xz in a standalone package", {
  skip_if_no_git_or_bash()
  repo <- make_vendor_repo()
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  writeBin(as.raw(1:64), file.path(repo, "inst", "vendor.tar.xz"))
  # -f: the scaffold .gitignore would normally refuse the path.
  git(repo, "add", "-f", "DESCRIPTION", "inst/vendor.tar.xz")

  out <- run_hook(repo)
  expect_identical(hook_status(out), 1L, info = paste(out, collapse = "\n"))
  expect_true(any(grepl("BLOCKED: vendor tarball staged", out, fixed = TRUE)),
              info = paste(out, collapse = "\n"))
  expect_true(any(grepl("git rm --cached inst/vendor.tar.xz", out, fixed = TRUE)),
              info = paste(out, collapse = "\n"))
})

test_that("pre-commit hook blocks a staged vendor tarball under a monorepo prefix", {
  skip_if_no_git_or_bash()
  repo <- make_vendor_repo("rpkg")
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  writeBin(as.raw(1:64), file.path(repo, "rpkg", "inst", "vendor.tar.xz"))
  git(repo, "add", "-f", "rpkg/inst/vendor.tar.xz")

  out <- run_hook(repo)
  expect_identical(hook_status(out), 1L, info = paste(out, collapse = "\n"))
  expect_true(any(grepl("git rm --cached rpkg/inst/vendor.tar.xz", out, fixed = TRUE)),
              info = paste(out, collapse = "\n"))
})

test_that("pre-commit hook blocks a vendor tarball outside inst/", {
  # Older scaffolds wrote src/rust/vendor.tar.xz; any vendor.tar.xz path blocks.
  skip_if_no_git_or_bash()
  repo <- make_vendor_repo()
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  writeBin(as.raw(1:64), file.path(repo, "src", "rust", "vendor.tar.xz"))
  git(repo, "add", "-f", "src/rust/vendor.tar.xz")

  out <- run_hook(repo)
  expect_identical(hook_status(out), 1L, info = paste(out, collapse = "\n"))
  expect_true(any(grepl("git rm --cached src/rust/vendor.tar.xz", out, fixed = TRUE)),
              info = paste(out, collapse = "\n"))
})

test_that("pre-commit hook lets the commit that untracks the tarball through", {
  skip_if_no_git_or_bash()
  repo <- make_vendor_repo()
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  writeBin(as.raw(1:64), file.path(repo, "inst", "vendor.tar.xz"))
  git(repo, "add", "-f", "DESCRIPTION", "inst/vendor.tar.xz")
  git(repo, "commit", "-q", "-m", "tracked tarball (pre-.gitignore history)")

  # The fix the block message prints: stage the removal, keep the file on disk.
  git(repo, "rm", "-q", "--cached", "inst/vendor.tar.xz")
  writeLines(c("Package: testpkg", "Version: 0.1.1"), file.path(repo, "DESCRIPTION"))
  git(repo, "add", "DESCRIPTION")

  out <- run_hook(repo)
  expect_identical(hook_status(out), 0L, info = paste(out, collapse = "\n"))
  expect_true(file.exists(file.path(repo, "inst", "vendor.tar.xz")))
})

test_that("pre-commit hook no longer nudges to refresh an unstaged tarball", {
  skip_if_no_git_or_bash()
  repo <- make_vendor_repo()
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  # Tarball on disk (untracked), Rust manifest staged: the old hook printed
  # "inst/vendor.tar.xz was not updated" here.
  writeBin(as.raw(1:64), file.path(repo, "inst", "vendor.tar.xz"))
  git(repo, "add", "src/rust/Cargo.toml")

  out <- run_hook(repo)
  expect_identical(hook_status(out), 0L, info = paste(out, collapse = "\n"))
  expect_false(any(grepl("was not updated", out, fixed = TRUE)),
               info = paste(out, collapse = "\n"))
})
