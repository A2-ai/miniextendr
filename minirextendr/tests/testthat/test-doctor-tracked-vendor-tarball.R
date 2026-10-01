# Tests for the tracked-vendor-tarball check in miniextendr_doctor().
#
# The on-disk check only sees whether inst/vendor.tar.xz exists. A *tracked*
# tarball is worse: every clone gets it, configure's leaked-tarball guard then
# stops the build there, and every committed refresh stays in the history.
# doctor must name each tracked tarball with `git rm --cached <path>`, never
# touch the index itself, and skip quietly without git or outside a repo.

git_in <- function(dir, ...) {
  out <- suppressWarnings(
    system2("git", c("-C", shQuote(dir), ...), stdout = TRUE, stderr = TRUE)
  )
  status <- attr(out, "status")
  if (!is.null(status) && status != 0L) {
    stop("git ", paste(c(...), collapse = " "), " failed:\n", paste(out, collapse = "\n"))
  }
  invisible(out)
}

# Minimal package for doctor() at `pkg_dir` inside a fresh temp dir `root`,
# which is a git repo when `git = TRUE`. Returns the package path with the
# root as attribute "root" (what the caller deletes afterwards).
make_tarball_pkg <- function(git = TRUE, pkg_dir = ".") {
  root <- tempfile("doctor-vendor-tracked-")
  pkg <- file.path(root, pkg_dir)
  dir.create(file.path(pkg, "src", "rust"), recursive = TRUE)
  writeLines(c("Package: testpkg", "Title: Test", "Version: 0.1.0", ""),
             file.path(pkg, "DESCRIPTION"))
  writeLines("", file.path(pkg, "NAMESPACE"))
  writeLines(c("[package]", 'name = "testpkg"'), file.path(pkg, "src", "rust", "Cargo.toml"))
  if (git) git_in(root, "init", "--quiet")
  structure(normalizePath(pkg, mustWork = TRUE), root = root)
}

plant_tarball <- function(pkg, rel_path, track = TRUE) {
  target <- file.path(pkg, rel_path)
  dir.create(dirname(target), recursive = TRUE, showWarnings = FALSE)
  writeBin(as.raw(1:64), target)
  # -f: the scaffold .gitignore (or a global excludes file) may ignore it.
  if (track) git_in(pkg, "add", "-f", "--", rel_path)
  invisible(target)
}

flatten <- function(msgs) gsub("\\s+", " ", paste(msgs, collapse = " "))

test_that("doctor fails on a tracked inst/vendor.tar.xz and advises git rm --cached", {
  skip_if(!nzchar(Sys.which("git")), "git not on PATH")
  pkg <- make_tarball_pkg()
  on.exit(unlink(attr(pkg, "root"), recursive = TRUE), add = TRUE)
  plant_tarball(pkg, "inst/vendor.tar.xz")

  msgs <- testthat::capture_messages(result <- miniextendr_doctor(pkg))

  expect_true("vendor tarball tracked in git: inst/vendor.tar.xz" %in% result$fail)
  flat <- flatten(msgs)
  expect_match(flat, "git rm --cached inst/vendor.tar.xz", fixed = TRUE)
  # The on-disk report points at the untracking step first.
  expect_match(flat, "It is tracked in git (see below): untrack it first", fixed = TRUE)
  # Advice only: still tracked, still on disk.
  expect_identical(as.character(git_in(pkg, "ls-files", "--", "inst/vendor.tar.xz")),
                   "inst/vendor.tar.xz")
  expect_true(file.exists(file.path(pkg, "inst", "vendor.tar.xz")))
})

test_that("doctor reports a tracked tarball outside inst/ as well", {
  skip_if(!nzchar(Sys.which("git")), "git not on PATH")
  pkg <- make_tarball_pkg()
  on.exit(unlink(attr(pkg, "root"), recursive = TRUE), add = TRUE)
  plant_tarball(pkg, "src/rust/vendor.tar.xz")

  msgs <- testthat::capture_messages(result <- miniextendr_doctor(pkg))

  expect_true("vendor tarball tracked in git: src/rust/vendor.tar.xz" %in% result$fail)
  expect_match(flatten(msgs), "git rm --cached src/rust/vendor.tar.xz", fixed = TRUE)
})

test_that("an untracked tarball on disk is not reported as tracked", {
  skip_if(!nzchar(Sys.which("git")), "git not on PATH")
  pkg <- make_tarball_pkg()
  on.exit(unlink(attr(pkg, "root"), recursive = TRUE), add = TRUE)
  plant_tarball(pkg, "inst/vendor.tar.xz", track = FALSE)

  result <- suppressMessages(miniextendr_doctor(pkg))

  expect_true("no vendor tarball tracked in git" %in% result$pass)
  expect_false(any(grepl("tracked in git: ", result$fail, fixed = TRUE)))
  # The on-disk leak is still reported, with the usual advice.
  expect_true("stale inst/vendor.tar.xz in source tree" %in% result$fail)
})

test_that("paths are relative to a package in a monorepo subdirectory", {
  skip_if(!nzchar(Sys.which("git")), "git not on PATH")
  pkg <- make_tarball_pkg(pkg_dir = "rpkg")
  on.exit(unlink(attr(pkg, "root"), recursive = TRUE), add = TRUE)
  plant_tarball(pkg, "inst/vendor.tar.xz")
  # A tarball elsewhere in the repo belongs to someone else's package.
  plant_tarball(attr(pkg, "root"), "other/inst/vendor.tar.xz")

  expect_identical(tracked_vendor_tarballs(pkg), "inst/vendor.tar.xz")
})

test_that("the check is skipped outside a git repository", {
  skip_if(!nzchar(Sys.which("git")), "git not on PATH")
  pkg <- make_tarball_pkg(git = FALSE)
  on.exit(unlink(attr(pkg, "root"), recursive = TRUE), add = TRUE)

  expect_null(tracked_vendor_tarballs(pkg))
  result <- suppressMessages(miniextendr_doctor(pkg))
  expect_false("no vendor tarball tracked in git" %in% result$pass)
  expect_false(any(grepl("vendor tarball tracked in git", result$fail, fixed = TRUE)))
})

test_that("the check is skipped when git is not installed", {
  pkg <- make_tarball_pkg(git = FALSE)
  on.exit(unlink(attr(pkg, "root"), recursive = TRUE), add = TRUE)
  # An empty PATH hides git (and everything else) from Sys.which().
  withr::local_envvar(PATH = withr::local_tempdir())

  expect_null(tracked_vendor_tarballs(pkg))
})
