# Release workflow scaffolding

#' Add an r-release.yml workflow with miniextendr platform fixes
#'
#' Copies the bundled `r-release.yml` template into a target package's
#' `.github/workflows/` directory. The template encodes the four platform
#' gotchas surfaced when running release workflows against AlmaLinux 8
#' container builds and macOS arm64 runners (issue #448):
#'
#' 1. `LANG=C.UTF-8` scoped to container jobs — AlmaLinux 8 minimal defaults
#'    to the `C` locale, which fails miniextendr's UTF-8 assertion.
#' 2. Not set workflow-wide — macOS rejects `C.UTF-8` (glibc-only identifier).
#' 3. `dnf install git gh` + `gh auth setup-git` for AlmaLinux 8 minimal.
#' 4. `CARGO_NET_GIT_FETCH_WITH_CLI=true` workflow-wide so cargo uses the
#'    system git binary and can authenticate private git dependencies.
#'
#' See `docs/RELEASE_WORKFLOW.md` in the miniextendr repository for the full
#' rationale, and \code{\link[miniextendr]{assert_utf8_locale_now}} for why the
#' UTF-8 check exists.
#'
#' @param path Path to the package root. Defaults to `"."`.
#' @param overwrite If `TRUE`, replace an existing
#'   `.github/workflows/r-release.yml`. Default `FALSE`.
#' @return The path to the written file, invisibly.
#' @export
use_release_workflow <- function(path = ".", overwrite = FALSE) {
  dest_dir <- file.path(path, ".github", "workflows")
  dest_file <- file.path(dest_dir, "r-release.yml")

  if (fs::file_exists(dest_file) && !overwrite) {
    cli::cli_abort(c(
      "{.path {dest_file}} already exists.",
      "i" = "Use {.code overwrite = TRUE} to replace it."
    ))
  }

  src <- system.file("templates", "r-release.yml", package = "minirextendr",
                     mustWork = TRUE)

  fs::dir_create(dest_dir, recurse = TRUE)
  fs::file_copy(src, dest_file, overwrite = overwrite)

  cli::cli_alert_success("Written {.path {dest_file}}")
  cli::cli_alert_info(c(
    "Review the template and extend it with your package's setup steps.",
    "i" = "See {.url https://a2-ai.github.io/miniextendr/manual/release-workflow/} for the rationale."
  ))

  invisible(dest_file)
}
