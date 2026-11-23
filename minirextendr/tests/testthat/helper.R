# Shared test helpers (auto-sourced by testthat before tests)

# Find the local miniextendr monorepo root, or NULL if unavailable.
find_miniextendr_repo <- function() {
  # Check environment variable first (set by justfile recipes)
  env_path <- Sys.getenv("MINIEXTENDR_LOCAL_PATH", "")
  if (nzchar(env_path) && dir.exists(file.path(env_path, "miniextendr-api"))) {
    return(env_path)
  }

  # Check relative to minirextendr package source (development mode)
  # minirextendr is at miniextendr/minirextendr/, so parent is the repo
  pkg_path <- normalizePath(
    file.path(testthat::test_path(), "..", ".."),
    mustWork = FALSE
  )
  parent <- dirname(pkg_path)
  if (dir.exists(file.path(parent, "miniextendr-api"))) {
    return(parent)
  }

  NULL
}

# Skip test if local miniextendr repo is not available.
skip_if_no_local_repo <- function() {
  testthat::skip_if(
    is.null(find_miniextendr_repo()),
    "Local miniextendr repo not found (set MINIEXTENDR_LOCAL_PATH)"
  )
}

# Standard e2e skip guards (test-templates.R, test-scaffold-smoke.R).
#
# These tests cold-compile a Rust crate, run autoconf, and (for the standalone
# round-trip) network-fetch + vendor miniextendr — minutes per run. They're
# skipped on CI by default so the per-PR `minirextendr` job stays fast.
#
# Opt-in bypass: set MINIEXTENDR_RUN_E2E=1 to run them on CI anyway (used by the
# weekly/label-gated round-trip job — see .github/workflows/ci.yml
# `r-roundtrip-e2e`). The bypass only lifts the skip_on_ci() gate; the
# tool-availability + local-repo guards below still apply, so a misconfigured
# runner skips with a clear reason instead of erroring. Local runs (where
# skip_on_ci() is already a no-op) are unaffected by the flag. See #775 / #805:
# before this, skip_on_ci() ran first and the round-trip was unreachable in CI.
skip_e2e <- function() {
  # as.logical("1") is NA in R, so accept the common truthy spellings explicitly.
  run_e2e <- tolower(Sys.getenv("MINIEXTENDR_RUN_E2E", "")) %in% c("1", "true", "yes")
  if (!run_e2e) {
    skip_on_ci()
  }
  skip_if_not(nzchar(Sys.which("autoconf")), "autoconf not available")
  skip_if_not(nzchar(Sys.which("cargo")), "Rust toolchain not available")
  skip_if_not(nzchar(Sys.which("R")), "R not available")
  skip_if_no_local_repo()
}
