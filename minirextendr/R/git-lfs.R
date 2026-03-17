# Git LFS helpers for miniextendr packages

#' Set up Git LFS tracking for vendor.tar.xz
#'
#' Configures Git LFS to track `inst/vendor.tar.xz` (the vendored Rust
#' dependencies archive). This prevents large binary files from bloating
#' the Git repository.
#'
#' The function:
#' 1. Checks that `git lfs` is available
#' 2. Reads any existing `.gitattributes` (preserving user entries)
#' 3. Adds `inst/vendor.tar.xz filter=lfs diff=lfs merge=lfs -text` if not
#'    already tracked
#' 4. Runs `git lfs install` if not already initialized
#'
#' @param path Path to the R package (or monorepo) root.
#' @param patterns Character vector of glob patterns to track with LFS.
#'   Defaults to `"inst/vendor.tar.xz"`. Set to `NULL` to only run
#'   `git lfs install` without adding patterns.
#' @return Invisibly returns `TRUE` if changes were made, `FALSE` otherwise.
#' @export
use_git_lfs <- function(path = ".", patterns = "inst/vendor.tar.xz") {
  with_project(path)
  changed <- FALSE

  # Check git lfs is available
  git_lfs <- Sys.which("git-lfs")
  if (!nzchar(git_lfs)) {
    # Also check via git lfs version
    lfs_ok <- tryCatch({
      system2("git", c("lfs", "version"), stdout = TRUE, stderr = TRUE)
      TRUE
    }, error = function(e) FALSE, warning = function(e) FALSE)
    if (!lfs_ok) {
      cli::cli_abort(c(
        "Git LFS is not installed.",
        "i" = "Install it from {.url https://git-lfs.com}"
      ))
    }
  }

  # Ensure git lfs is initialized in this repo
  git_dir <- tryCatch(usethis::proj_path(".git"), error = function(e) NULL)
  if (!is.null(git_dir) && dir.exists(git_dir)) {
    lfs_hooks <- file.path(git_dir, "hooks", "pre-push")
    if (!file.exists(lfs_hooks) || !any(grepl("lfs", readLines(lfs_hooks, warn = FALSE)))) {
      tryCatch({
        system2("git", c("lfs", "install"), stdout = TRUE, stderr = TRUE)
        cli::cli_alert_success("Initialized Git LFS")
        changed <- TRUE
      }, error = function(e) {
        cli::cli_alert_warning("Could not initialize Git LFS: {conditionMessage(e)}")
      })
    }
  }

  if (is.null(patterns) || length(patterns) == 0) {
    return(invisible(changed))
  }

  # Read existing .gitattributes
  gitattributes_path <- usethis::proj_path(".gitattributes")
  existing_lines <- character()
  if (file.exists(gitattributes_path)) {
    existing_lines <- readLines(gitattributes_path, warn = FALSE)
  }

  # Add each pattern if not already tracked
  for (pattern in patterns) {
    lfs_line <- paste0(pattern, " filter=lfs diff=lfs merge=lfs -text")
    # Check if this pattern is already tracked (match the pattern portion)
    already_tracked <- any(grepl(
      paste0("^", gsub("([.\\[\\]{}()+*?^$|])", "\\\\\\1", pattern), "\\s"),
      existing_lines
    ))
    if (!already_tracked) {
      existing_lines <- c(existing_lines, lfs_line)
      cli::cli_alert_success("Tracking {.path {pattern}} with Git LFS")
      changed <- TRUE
    } else {
      cli::cli_alert_info("{.path {pattern}} already tracked by Git LFS")
    }
  }

  if (changed) {
    writeLines(existing_lines, gitattributes_path)
    cli::cli_alert_info("Updated {.path .gitattributes}")
  }

  invisible(changed)
}

#' Check if Git LFS is tracking vendor.tar.xz
#'
#' @param path Path to the R package root.
#' @return `TRUE` if `inst/vendor.tar.xz` is tracked by LFS, `FALSE` otherwise.
#' @keywords internal
has_git_lfs_tracking <- function(path = ".") {
  gitattributes_path <- file.path(path, ".gitattributes")
  if (!file.exists(gitattributes_path)) return(FALSE)
  lines <- readLines(gitattributes_path, warn = FALSE)
  any(grepl("inst/vendor\\.tar\\.xz.*filter=lfs", lines))
}

#' Check if Git LFS is available
#'
#' @return `TRUE` if `git lfs` is available, `FALSE` otherwise.
#' @keywords internal
has_git_lfs <- function() {
  tryCatch({
    out <- system2("git", c("lfs", "version"), stdout = TRUE, stderr = TRUE)
    length(out) > 0 && any(grepl("git-lfs", out))
  }, error = function(e) FALSE, warning = function(e) FALSE)
}
