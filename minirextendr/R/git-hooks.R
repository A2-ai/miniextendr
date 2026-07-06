# Git hook helpers for miniextendr packages

# Hook names shipped with minirextendr (single source of truth)
miniextendr_hook_names <- c("pre-commit", "post-merge")

# Pattern that detects an installed miniextendr hook section
miniextendr_hook_marker <- "miniextendr_pre_commit\\(\\)|miniextendr_post_merge\\(\\)|minirextendr::use_miniextendr_git_hooks"

#' Install git hooks for a miniextendr package
#'
#' Installs pre-commit and post-merge hooks that help keep your miniextendr
#' package in a healthy state:
#'
#' - **pre-commit**: Checks `cargo fmt`, blocks on stale `configure` script
#'   or stale NAMESPACE (when `*-wrappers.R` changed without `devtools::document()`),
#'   blocks on source-shape `src/rust/Cargo.lock` (recommends
#'   [miniextendr_repair_lock()]), notes when `inst/vendor.tar.xz` may need
#'   updating
#' - **post-merge**: Reminds you to reconfigure after pulling changes to
#'   build files (configure.ac, Makevars.in, Cargo.toml, Rust sources)
#'
#' Existing hooks are preserved: if a hook file already exists, the miniextendr
#' hook is appended as a sourced fragment so both run.
#'
#' Git hooks belong to the **enclosing repository**, regardless of where the
#' package sits inside it. This matches [use_miniextendr()]'s worldview: a
#' package nested somewhere under a repo root, or scaffolded at the root of a
#' linked git worktree, still gets its hooks installed in that repo. The hooks
#' directory is resolved through git itself (`git rev-parse --git-path hooks`),
#' so nesting, linked worktrees (whose `.git` is a file, not a directory), and
#' a configured `core.hooksPath` are all honoured.
#'
#' @param path Path to the R package (or monorepo) directory. The enclosing git
#'   repository is discovered from here; the package need not be the repo root.
#' @param hooks Character vector of hook names to install. Defaults to
#'   `c("pre-commit", "post-merge")`.
#' @param force If `TRUE`, replaces an existing miniextendr hook section with
#'   the current version from minirextendr. Useful after upgrading minirextendr.
#'   Default is `FALSE` (skip if already installed).
#' @return Invisibly returns `TRUE` if any hooks were installed or updated.
#' @export
use_miniextendr_git_hooks <- function(path = ".", hooks = miniextendr_hook_names,
                                      force = FALSE) {
  with_project(path)
  changed <- FALSE

  proj_dir <- usethis::proj_get()
  hooks_dir <- resolve_git_hooks_dir(proj_dir)
  if (is.null(hooks_dir)) {
    cli::cli_alert_warning("No git repository found for {.path {proj_dir}}. Initialize git first with {.code usethis::use_git()}.")
    return(invisible(FALSE))
  }

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

    if (file.exists(dest)) {
      existing <- readLines(dest, warn = FALSE)
      has_miniextendr <- any(grepl(miniextendr_hook_marker, existing))

      if (has_miniextendr && !force) {
        cli::cli_alert_info("Hook {.val {hook_name}} already has miniextendr section -- skipping. Use {.code force = TRUE} to update.")
        next
      }

      if (has_miniextendr && force) {
        existing <- strip_miniextendr_section(existing)
        if (length(existing) == 0) {
          # Standalone install (no banner, entire file was ours) -- overwrite
          cli::cli_alert_info("Replacing {.val {hook_name}} hook.")
          writeLines(hook_content, dest)
          Sys.chmod(dest, mode = "0755")
          changed <- TRUE
          next
        }
        cli::cli_alert_info("Replacing miniextendr section in {.val {hook_name}} hook.")
      } else {
        cli::cli_alert_info("Existing {.val {hook_name}} hook found -- appending miniextendr checks.")
      }

      # Remove shebang from our hook (the existing file already has one)
      hook_content <- hook_content[!grepl("^#!/", hook_content)]

      combined <- c(
        existing,
        "",
        "# -- miniextendr hooks (added by minirextendr::use_miniextendr_git_hooks) --",
        hook_content
      )
      writeLines(combined, dest)
      Sys.chmod(dest, mode = "0755")
      changed <- TRUE
    } else {
      # No existing hook -- install ours directly
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
#' Resolves the hooks directory through git (the same way
#' [use_miniextendr_git_hooks()] installs them), so the check agrees with the
#' installer for nested packages, linked worktrees, and a configured
#' `core.hooksPath`.
#'
#' @param path Path to the R package directory (the enclosing git repository is
#'   discovered from here).
#' @return Named logical vector indicating which hooks are installed.
#' @keywords internal
has_miniextendr_git_hooks <- function(path = ".") {
  hooks_dir <- resolve_git_hooks_dir(path)
  if (is.null(hooks_dir) || !dir.exists(hooks_dir)) {
    result <- rep(FALSE, length(miniextendr_hook_names))
    names(result) <- miniextendr_hook_names
    return(result)
  }

  result <- vapply(miniextendr_hook_names, function(name) {
    hook_file <- file.path(hooks_dir, name)
    if (!file.exists(hook_file)) return(FALSE)
    lines <- readLines(hook_file, warn = FALSE)
    any(grepl(miniextendr_hook_marker, lines))
  }, logical(1))

  result
}

#' Resolve the git hooks directory for a package directory
#'
#' Asks git itself where hooks live, from the perspective of `proj_dir`:
#' `git rev-parse --git-path hooks`. Going through git (rather than assuming
#' `<proj>/.git/hooks`) means the result is correct when
#'
#' - the package is nested inside a larger repo (the enclosing repo's hooks dir
#'   is returned, not a non-existent `<proj>/.git`);
#' - `proj_dir` is a linked git worktree, whose `.git` is a *file* (a gitlink)
#'   rather than a directory — git returns the common hooks dir;
#' - `core.hooksPath` is configured (git >= 2.36 reflects it here — verified
#'   against this repo's own `core.hooksPath = .githooks`).
#'
#' `--git-path` yields a path relative to the current directory; `run_command()`
#' runs git with the working directory set to `proj_dir`, so a relative result
#' is anchored there before being normalised to an absolute path.
#'
#' @param proj_dir Package directory to resolve from.
#' @return Absolute path to the hooks directory, or `NULL` when git is
#'   unavailable or `proj_dir` is not inside a git repository.
#' @keywords internal
resolve_git_hooks_dir <- function(proj_dir) {
  if (!nzchar(Sys.which("git"))) {
    return(NULL)
  }
  res <- run_command("git", c("rev-parse", "--git-path", "hooks"), wd = proj_dir)
  status <- attr(res, "status")
  # Non-zero exit means git found no repository (or errored) -- treat as "no repo".
  if (!is.null(status) && !identical(as.integer(status), 0L)) {
    return(NULL)
  }
  hooks_path <- trimws(paste(res, collapse = "\n"))
  if (!nzchar(hooks_path)) {
    return(NULL)
  }
  if (!is_absolute_path(hooks_path)) {
    hooks_path <- file.path(proj_dir, hooks_path)
  }
  normalizePath(hooks_path, mustWork = FALSE)
}

#' Strip miniextendr section from a hook file's lines
#'
#' Removes everything from the miniextendr banner comment to the end of
#' the file (the miniextendr section is always appended last).
#'
#' @param lines Character vector of hook file lines.
#' @return Lines with the miniextendr section removed.
#' @noRd
strip_miniextendr_section <- function(lines) {
  banner_idx <- grep("miniextendr hooks (added by", lines, fixed = TRUE)
  if (length(banner_idx) == 0) {
    # No banner -- this was a standalone install (entire file is ours)
    return(character(0))
  }
  # Keep everything before the banner (drop trailing blank lines)
  keep <- lines[seq_len(banner_idx[1] - 1)]
  # Trim trailing empty lines
  while (length(keep) > 0 && !nzchar(trimws(keep[length(keep)]))) {
    keep <- keep[-length(keep)]
  }
  keep
}
