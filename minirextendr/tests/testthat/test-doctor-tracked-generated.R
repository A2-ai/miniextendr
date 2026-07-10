# Tests for the tracked-generated-files check in miniextendr_doctor() (#1250).
#
# The scaffold .gitignore keeps configure-/install-generated files out of
# version control, but a .gitignore only affects UNTRACKED files: packages
# scaffolded before the #1226 pattern fix (the old `.cargo/config.toml`
# pattern was mis-anchored and never matched the nested path) may have
# already committed the generated, install-mode-specific
# src/rust/.cargo/config.toml. doctor must detect the tracked copy and advise
# `git rm --cached <path>` -- and must never mutate the user's git index
# itself. Outside a git work tree (CRAN's offline farm, extracted tarballs)
# the check is skipped silently.

# Run git in `dir`, aborting on failure. Uses `git -C` so the test process
# working directory never changes.
git_in <- function(dir, ...) {
  out <- suppressWarnings(
    system2("git", c("-C", dir, ...), stdout = TRUE, stderr = TRUE)
  )
  status <- attr(out, "status")
  if (!is.null(status) && status != 0L) {
    stop(
      "git ", paste(c(...), collapse = " "), " failed:\n",
      paste(out, collapse = "\n")
    )
  }
  invisible(out)
}

# Build a minimal fake R-package directory suitable for doctor(), optionally
# initialised as a real git repository. (Unlike the bare `.git` stub
# directory used by the vendor-tarball tests, `git ls-files` needs an actual
# repo -- a stub makes the check skip, which is itself covered below.)
make_doctor_git_pkg <- function(git = TRUE) {
  tmp <- tempfile("doctor-tracked-")
  dir.create(tmp)
  writeLines(
    c("Package: testpkg", "Title: Test", "Version: 0.1.0", ""),
    file.path(tmp, "DESCRIPTION")
  )
  writeLines("", file.path(tmp, "NAMESPACE"))

  rust_dir <- file.path(tmp, "src", "rust")
  dir.create(rust_dir, recursive = TRUE)
  writeLines(
    c("[package]", 'name = "testpkg"', "", "[dependencies]", 'miniextendr-api = "*"'),
    file.path(rust_dir, "Cargo.toml")
  )

  if (git) {
    git_in(tmp, "init", "--quiet")
  }

  tmp
}

# Write a generated file at `rel_path` and (optionally) `git add` it.
# `git add -f` sidesteps any user-global core.excludesFile that might happen
# to ignore the path. `git ls-files` reads the index, so adding alone makes
# the file tracked -- no commit required.
plant_generated <- function(pkg, rel_path, track = FALSE) {
  target <- file.path(pkg, rel_path)
  dir.create(dirname(target), recursive = TRUE, showWarnings = FALSE)
  writeLines("# generated", target)
  if (track) {
    git_in(pkg, "add", "-f", "--", rel_path)
  }
  invisible(target)
}

test_that("doctor warns on a committed src/rust/.cargo/config.toml and advises git rm --cached", {
  skip_if(!nzchar(Sys.which("git")), "git not on PATH")
  pkg <- make_doctor_git_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  plant_generated(pkg, "src/rust/.cargo/config.toml", track = TRUE)
  # Commit (not just stage) to mirror the real-world pre-#1226 scenario. The
  # -c flags keep the commit independent of user-global identity/signing
  # configuration.
  git_in(
    pkg,
    "-c", "user.name=Test", "-c", "user.email=test@example.com",
    "-c", "commit.gpgsign=false",
    "commit", "--quiet", "-m", "scaffold"
  )

  msgs <- testthat::capture_messages(
    result <- miniextendr_doctor(pkg)
  )

  expect_true(any(grepl(
    "generated file tracked in git: src/rust/.cargo/config.toml",
    result$warn,
    fixed = TRUE
  )))
  expect_false(any(grepl("tracked in git", result$fail, fixed = TRUE)))

  # The console advice names the exact remediation command. cli wraps long
  # lines at the console width, so collapse all message text and normalise
  # whitespace before matching.
  flat <- gsub("\\s+", " ", paste(msgs, collapse = " "))
  expect_match(flat, "git rm --cached src/rust/.cargo/config.toml", fixed = TRUE)

  # doctor advises only: the file must still be on disk AND still tracked.
  expect_true(file.exists(file.path(pkg, "src", "rust", ".cargo", "config.toml")))
  still_tracked <- git_in(pkg, "ls-files", "--", "src/rust/.cargo/config.toml")
  expect_identical(as.character(still_tracked), "src/rust/.cargo/config.toml")
})

test_that("doctor passes when generated files exist but are untracked", {
  skip_if(!nzchar(Sys.which("git")), "git not on PATH")
  pkg <- make_doctor_git_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  plant_generated(pkg, "src/rust/.cargo/config.toml", track = FALSE)
  plant_generated(pkg, "src/Makevars", track = FALSE)

  result <- suppressMessages(miniextendr_doctor(pkg))

  expect_true(any(grepl(
    "no generated files tracked in git", result$pass,
    fixed = TRUE
  )))
  expect_false(any(grepl("generated file tracked in git", result$warn, fixed = TRUE)))
})

test_that("doctor collects every tracked generated file in one batched pass", {
  skip_if(!nzchar(Sys.which("git")), "git not on PATH")
  pkg <- make_doctor_git_pkg()
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  # Staged-but-uncommitted files are already tracked (git ls-files reads the
  # index): exactly the set `git rm --cached` operates on. R/*-wrappers.R
  # exercises the glob pathspecs.
  tracked_files <- c(
    "src/rust/.cargo/config.toml",
    "src/Makevars",
    "R/testpkg-wrappers.R",
    "src/rust/wasm_registry.rs"
  )
  for (f in tracked_files) plant_generated(pkg, f, track = TRUE)
  # An untracked generated file must NOT be reported.
  plant_generated(pkg, "src/testpkg-wrappers.R", track = FALSE)

  result <- suppressMessages(miniextendr_doctor(pkg))

  tracked_warns <- grep("^generated file tracked in git: ", result$warn, value = TRUE)
  expect_setequal(
    tracked_warns,
    paste0("generated file tracked in git: ", tracked_files)
  )
  expect_false(any(grepl("generated files tracked in git", result$pass, fixed = TRUE)))
})

test_that("doctor skips the tracked-generated-files check silently outside a git work tree", {
  skip_if(!nzchar(Sys.which("git")), "git not on PATH")
  pkg <- make_doctor_git_pkg(git = FALSE)
  on.exit(unlink(pkg, recursive = TRUE), add = TRUE)
  plant_generated(pkg, "src/rust/.cargo/config.toml", track = FALSE)

  msgs <- testthat::capture_messages(
    result <- miniextendr_doctor(pkg)
  )

  # Neither a pass nor a warn: the check must not run at all (CRAN's offline
  # farm and extracted tarballs have no .git ancestor).
  expect_false(any(grepl("generated files tracked in git", result$pass, fixed = TRUE)))
  expect_false(any(grepl("generated file tracked in git", result$warn, fixed = TRUE)))
  expect_false(any(grepl("Tracked generated files", msgs, fixed = TRUE)))
})

test_that("tracked_generated_files distinguishes cannot-run (NULL) from none-tracked (empty)", {
  skip_if(!nzchar(Sys.which("git")), "git not on PATH")

  # No .git at all: the check cannot run.
  no_git <- make_doctor_git_pkg(git = FALSE)
  on.exit(unlink(no_git, recursive = TRUE), add = TRUE)
  expect_null(tracked_generated_files(no_git))

  # A bare `.git` stub directory (the vendor-tarball tests' fixture style) is
  # not a repository either: `git rev-parse` fails, the check skips.
  stub <- make_doctor_git_pkg(git = FALSE)
  on.exit(unlink(stub, recursive = TRUE), add = TRUE)
  dir.create(file.path(stub, ".git"))
  expect_null(tracked_generated_files(stub))

  # A real repo with nothing tracked: the check ran and found nothing.
  repo <- make_doctor_git_pkg(git = TRUE)
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  expect_identical(tracked_generated_files(repo), character(0))
})
