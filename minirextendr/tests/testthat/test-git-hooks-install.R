# Behavioral tests for use_miniextendr_git_hooks() / has_miniextendr_git_hooks()
# git detection: the hooks directory must be resolved through git itself so a
# nested package, a linked worktree, or a configured core.hooksPath all land in
# the enclosing repo's real hooks directory (audit 2026-07-06 finding #3).

skip_if_no_git <- function() {
  if (Sys.which("git") == "") testthat::skip("git not available")
}

# Create a fresh git repo in a tempdir with a quiet identity and one commit
# (needed before `git worktree add` can attach). Returns the repo path.
# Uses plain tempfile/dir.create so the dir outlives this helper's frame.
make_git_repo <- function() {
  repo <- tempfile("git-hooks-repo-")
  dir.create(repo)
  withr::with_dir(repo, {
    system2("git", c("init", "-q"))
    system2("git", c("config", "user.email", "test@example.com"))
    system2("git", c("config", "user.name", "Test"))
    writeLines("seed", "README")
    system2("git", c("add", "README"), stdout = FALSE, stderr = FALSE)
    system2("git", c("commit", "-qm", "init"), stdout = FALSE, stderr = FALSE)
  })
  normalizePath(repo, mustWork = TRUE)
}

test_that("hooks install into the enclosing repo from a nested package dir", {
  skip_if_no_git()
  repo <- make_git_repo()
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  pkg <- file.path(repo, "pkgsub")
  dir.create(pkg)

  # Nested package: no `.git` at `pkg`, but the enclosing repo has one. The old
  # dir.exists(proj_path(".git")) probe emitted a false "Initialize git first".
  msgs <- testthat::capture_messages(res <- use_miniextendr_git_hooks(path = pkg))
  expect_true(res)
  expect_false(any(grepl("No git repository|Initialize git first", msgs)))

  # Hooks must land in the enclosing repo's hooks dir, not a phantom pkg/.git.
  installed <- file.path(repo, ".git", "hooks", miniextendr_hook_names)
  expect_true(all(file.exists(installed)))
  expect_false(dir.exists(file.path(pkg, ".git")))

  # The checker agrees when probed from the nested dir.
  expect_true(all(has_miniextendr_git_hooks(pkg)))
})

test_that("hooks honour core.hooksPath instead of .git/hooks", {
  skip_if_no_git()
  repo <- make_git_repo()
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  # Point git at a custom hooks dir (relative, resolved against the repo root).
  system2("git", c("-C", repo, "config", "core.hooksPath", ".githooks"))

  suppressMessages(res <- use_miniextendr_git_hooks(path = repo))
  expect_true(res)

  # Hooks land in the configured dir, and NOT in the default .git/hooks.
  configured <- file.path(repo, ".githooks", miniextendr_hook_names)
  expect_true(all(file.exists(configured)))
  default <- file.path(repo, ".git", "hooks", miniextendr_hook_names)
  expect_false(any(file.exists(default)))

  expect_true(all(has_miniextendr_git_hooks(repo)))
})

test_that("hooks install from a linked git worktree (.git is a file)", {
  skip_if_no_git()
  repo <- make_git_repo()
  on.exit(unlink(repo, recursive = TRUE), add = TRUE)
  wt <- tempfile("git-hooks-wt-")
  on.exit(unlink(wt, recursive = TRUE), add = TRUE)

  added <- suppressWarnings(system2(
    "git", c("-C", repo, "worktree", "add", "-q", wt),
    stdout = TRUE, stderr = TRUE
  ))
  status <- attr(added, "status")
  if (!is.null(status) && status != 0L) {
    testthat::skip("git worktree add unavailable in this environment")
  }
  # Confirm the precondition the old dir.exists() probe tripped on: a linked
  # worktree's `.git` is a gitlink FILE, not a directory.
  expect_true(file.exists(file.path(wt, ".git")))
  expect_false(dir.exists(file.path(wt, ".git")))

  suppressMessages(res <- use_miniextendr_git_hooks(path = wt))
  expect_true(res)

  # A linked worktree shares the main repo's hooks (common git dir).
  installed <- file.path(repo, ".git", "hooks", miniextendr_hook_names)
  expect_true(all(file.exists(installed)))

  expect_true(all(has_miniextendr_git_hooks(wt)))
})

test_that("no git repository warns and returns FALSE", {
  skip_if_no_git()
  bare <- tempfile("git-hooks-norepo-")
  dir.create(bare)
  on.exit(unlink(bare, recursive = TRUE), add = TRUE)

  expect_message(
    res <- use_miniextendr_git_hooks(path = bare),
    "No git repository found"
  )
  expect_false(res)
  # Checker agrees: nothing installed.
  expect_false(any(has_miniextendr_git_hooks(bare)))
})
