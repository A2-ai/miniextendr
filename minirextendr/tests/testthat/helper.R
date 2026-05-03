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
