# Git hook helpers for miniextendr packages

#' Install git hooks for a miniextendr package
#'
#' Installs pre-commit and post-merge hooks that help keep your miniextendr
#' package in a healthy state:
#'
#' - **pre-commit**: Checks `cargo fmt`, blocks on stale `configure` script
#'   or stale NAMESPACE (when `*-wrappers.R` changed without `devtools::document()`),
#'   notes when `inst/vendor.tar.xz` may need updating
#' - **post-merge**: Reminds you to reconfigure after pulling changes to
#'   build files (configure.ac, Makevars.in, Cargo.toml, Rust sources)
#'
#' Existing hooks are preserved: if a hook file already exists, the miniextendr
#' hook is appended as a sourced fragment so both run.
#'
#' @param path Path to the R package (or monorepo) root.
#' @param hooks Character vector of hook names to install. Defaults to
#'   `c("pre-commit", "post-merge")`.
#' @return Invisibly returns `TRUE` if any hooks were installed or updated.
#' @export
use_miniextendr_git_hooks <- function(path = ".", hooks = c("pre-commit", "post-merge")) {
  with_project(path)
  changed <- FALSE

  git_dir <- tryCatch(usethis::proj_path(".git"), error = function(e) NULL)
  if (is.null(git_dir) || !dir.exists(git_dir)) {
    cli::cli_alert_warning("No .git directory found. Initialize git first with {.code usethis::use_git()}.")
    return(invisible(FALSE))
  }

  hooks_dir <- file.path(git_dir, "hooks")
  if (!dir.exists(hooks_dir)) {
    dir.create(hooks_dir, recursive = TRUE)
  }

  for (hook_name in hooks) {
    src <- system.file("hooks", hook_name, package = "minirextendr", mustWork = FALSE)
    if (!nzchar(src) || !file.exists(src)) {
      cli::cli_alert_warning("Hook template {.val {hook_name}} not found in minirextendr package.")
      next
    }

    dest <- file.path(hooks_dir, hook_name)
    hook_content <- readLines(src, warn = FALSE)
    miniextendr_marker <- "miniextendr_pre_commit\\(\\)|miniextendr_post_merge\\(\\)|minirextendr::use_miniextendr_git_hooks"

    if (file.exists(dest)) {
      existing <- readLines(dest, warn = FALSE)

      # Already installed — check if update needed
      if (any(grepl(miniextendr_marker, existing))) {
        cli::cli_alert_info("Hook {.val {hook_name}} already has miniextendr section — skipping.")
        next
      }

      # Existing hook from another tool — append ours
      cli::cli_alert_info("Existing {.val {hook_name}} hook found — appending miniextendr checks.")

      # Remove shebang from our hook (the existing file already has one)
      hook_content <- hook_content[!grepl("^#!/", hook_content)]

      combined <- c(
        existing,
        "",
        "# ── miniextendr hooks (added by minirextendr::use_miniextendr_git_hooks) ──",
        hook_content
      )
      writeLines(combined, dest)
      Sys.chmod(dest, mode = "0755")
      changed <- TRUE
    } else {
      # No existing hook — install ours directly
      writeLines(hook_content, dest)
      Sys.chmod(dest, mode = "0755")
      cli::cli_alert_success("Installed {.val {hook_name}} hook.")
      changed <- TRUE
    }
  }

  if (changed) {
    cli::cli_alert_info("Git hooks installed in {.path {hooks_dir}}")
  }

  invisible(changed)
}

#' Check if miniextendr git hooks are installed
#'
#' @param path Path to the R package root.
#' @return Named logical vector indicating which hooks are installed.
#' @keywords internal
has_miniextendr_git_hooks <- function(path = ".") {
  git_dir <- file.path(path, ".git")
  if (!dir.exists(git_dir)) {
    return(c("pre-commit" = FALSE, "post-merge" = FALSE))
  }

  hooks_dir <- file.path(git_dir, "hooks")
  hook_names <- c("pre-commit", "post-merge")
  marker <- "miniextendr_pre_commit\\(\\)|miniextendr_post_merge\\(\\)|minirextendr::use_miniextendr_git_hooks"

  result <- vapply(hook_names, function(name) {
    hook_file <- file.path(hooks_dir, name)
    if (!file.exists(hook_file)) return(FALSE)
    lines <- readLines(hook_file, warn = FALSE)
    any(grepl(marker, lines))
  }, logical(1))

  result
}
