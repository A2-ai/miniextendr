# Package upgrade functions

#' Upgrade a miniextendr package
#'
#' Comprehensively upgrades an existing miniextendr package to the latest
#' build system templates, vendored crates, and package metadata. This replaces
#' the old `miniextendr_update()` with a more thorough upgrade that covers
#' configure.ac, DESCRIPTION, .gitignore, .gitattributes, .Rbuildignore, build.rs, and
#' generated C files.
#'
#' User-authored files (lib.rs, Cargo.toml, R/ code) are never touched.
#' Missing generated-file merge rules are appended to `.gitattributes`,
#' preserving existing rules. On a generated-file conflict, regenerate from
#' the merged sources and stage the result with `git add`.
#'
#' In a monorepo layout (workspace root containing an rpkg subdirectory),
#' `upgrade_miniextendr_package()` automatically detects the rpkg subdir and
#' operates on it rather than the workspace root. The rpkg subdir is the first
#' immediate child directory that contains a miniextendr `configure.ac`. Pass
#' `rpkg_subdir` explicitly if auto-detection is ambiguous. `path` may also
#' point at the rpkg subdir itself, when its parent directory holds the
#' workspace `Cargo.toml`. Either way the upgrade renders the monorepo
#' templates and also adds the monorepo template's entries to the workspace
#' root `.gitignore`, keeping the lines already there.
#'
#' A `tools/config.guess` or `tools/config.sub` whose `timestamp=` line is
#' newer than the bundled copy's is kept. The closing summary lists files the
#' package no longer uses (`tools/wrapper-freshness.R`,
#' `tools/wrapper-inputs.rds`) and a git-tracked `src/<pkg>-win.def`, which
#' configure generates; the upgrade does not delete or untrack them.
#'
#' @param path Path to the R package root (standalone) or the monorepo workspace
#'   root. Defaults to `"."`. For a monorepo, this is the directory containing
#'   `Cargo.toml` -- the rpkg subdir is resolved automatically.
#' @param rpkg_subdir For monorepo layouts: name of the R package subdirectory
#'   (e.g. `"rpkg"`). If `NULL` (default), auto-detected by scanning immediate
#'   subdirectories for a miniextendr `configure.ac`.
#' @param version Version of miniextendr crates to vendor (default: `"main"`).
#' @param local_path Optional path to local miniextendr repository for vendoring.
#' @param configure_ac Logical. If `TRUE`, overwrites configure.ac with the
#'   current template. Defaults to `FALSE` because users often customise
#'   configure.ac with feature flags. When `FALSE`, a heuristic check warns
#'   if the existing configure.ac appears outdated.
#' @param autoconf Logical. If `TRUE` (default) and `autoconf` is available,
#'   regenerates the configure script after upgrading.
#' @param allow_dirty Logical. If `FALSE` (default), aborts when scaffolding
#'   files have uncommitted changes in git, to prevent accidental data loss.
#'   Set to `TRUE` to force the upgrade even with dirty files.
#' @return Invisibly returns TRUE on success.
#' @export
upgrade_miniextendr_package <- function(path = ".",
                                         rpkg_subdir = NULL,
                                         version = "main",
                                         local_path = NULL,
                                         configure_ac = FALSE,
                                         autoconf = TRUE,
                                         allow_dirty = FALSE) {
  layout <- upgrade_layout(path, rpkg_subdir)
  resolved_path <- layout$pkg
  monorepo <- !is.null(layout$root)
  if (monorepo) {
    cli::cli_alert_info("Monorepo layout detected -- upgrading rpkg subdir {.path {layout$subdir}}")
  }

  with_project(resolved_path)

  # Render the template set of this layout, whatever type an earlier
  # scaffolding call in the session left active, and restore that type on
  # exit. The monorepo's package templates live under
  # templates/monorepo/rpkg/, so every use_*() call below takes
  # subdir = "rpkg" there, as create_rpkg_subdirectory() does (#1712).
  old_template_type <- get_template_type()
  set_template_type(if (monorepo) "monorepo" else "rpkg")
  on.exit(set_template_type(old_template_type), add = TRUE)
  tpl_subdir <- if (monorepo) "rpkg" else NULL

  if (!is_miniextendr_package()) {
    cli::cli_abort(c(
      "This does not appear to be a miniextendr package.",
      "i" = "Expected configure.ac with CARGO_FEATURES variable and build templates.",
      "i" = "Use {.code create_miniextendr_package()} to scaffold a new package."
    ))
  }

  cli::cli_h1("Upgrading miniextendr package")

  # --- Dirty check ---
  if (!allow_dirty) {
    check_scaffolding_clean(resolved_path)
  }

  # --- Build system templates ---
  cli::cli_h2("Updating build system templates")
  use_miniextendr_stub(subdir = tpl_subdir)
  use_miniextendr_makevars(subdir = tpl_subdir)
  use_miniextendr_mx_abi(subdir = tpl_subdir)
  use_miniextendr_build_rs(subdir = tpl_subdir)
  use_miniextendr_bootstrap(subdir = tpl_subdir)
  use_miniextendr_cleanup(subdir = tpl_subdir)
  use_miniextendr_configure_win(subdir = tpl_subdir)
  use_miniextendr_config_scripts(subdir = tpl_subdir)
  use_miniextendr_html_reference(subdir = tpl_subdir)

  # --- Package metadata ---
  cli::cli_h2("Updating package metadata")
  use_miniextendr_description()
  use_miniextendr_rbuildignore(subdir = tpl_subdir)
  upgrade_gitignore(subdir = tpl_subdir)
  use_miniextendr_gitattributes(subdir = tpl_subdir)

  # --- configure.ac ---
  if (configure_ac) {
    cli::cli_h2("Replacing configure.ac")
    use_miniextendr_configure(subdir = tpl_subdir)
  } else {
    check_configure_ac_drift()
  }

  # --- Autoconf ---
  if (autoconf && nzchar(Sys.which("autoconf"))) {
    cli::cli_h2("Regenerating configure script")
    tryCatch(
      miniextendr_autoconf(),
      error = function(e) {
        cli::cli_alert_warning("autoconf failed: {conditionMessage(e)}")
      }
    )
  }

  # --- Summary ---
  cli::cli_h1("Upgrade complete!")
  report_upgrade_leftovers(resolved_path)
  cli::cli_alert_info("Next steps:")
  cli::cli_bullets(c(
    " " = "Review changes with {.code git diff}",
    " " = "Run {.code minirextendr::miniextendr_build()} to rebuild"
  ))

  invisible(TRUE)
}

#' Find the rpkg subdirectory in a monorepo workspace root
#'
#' Scans immediate subdirectories of `path` for one that contains a
#' `configure.ac` with the `CARGO_FEATURES` marker (the canonical signal that
#' a directory is a miniextendr rpkg). Returns the single match, `NULL` if
#' none is found, or aborts if more than one match is found.
#'
#' @param path Path to the monorepo workspace root.
#' @return Name of the rpkg subdirectory (not a full path), or `NULL`.
#' @noRd
find_rpkg_subdir <- function(path) {
  subdirs <- list.dirs(path, full.names = FALSE, recursive = FALSE)
  matches <- character(0)
  for (d in subdirs) {
    configure_ac <- file.path(path, d, "configure.ac")
    if (file.exists(configure_ac)) {
      content <- readLines(configure_ac, warn = FALSE)
      if (any(grepl("CARGO_FEATURES", content, fixed = TRUE))) {
        matches <- c(matches, d)
      }
    }
  }
  if (length(matches) > 1) {
    cli::cli_abort(c(
      "Multiple rpkg subdirectories detected in {.path {path}}:",
      stats::setNames(matches, rep("*", length(matches))),
      "i" = "Pass {.code rpkg_subdir = '<name>'} explicitly to disambiguate."
    ))
  }
  if (length(matches) == 0L) return(NULL)
  matches
}

#' Locate the R package an upgrade operates on, and its monorepo workspace
#'
#' `path` may be a standalone package, a monorepo workspace root (the R
#' package is then `rpkg_subdir` or what `find_rpkg_subdir()` finds), or the R
#' package subdirectory of a monorepo itself. That last case counts as a
#' monorepo only when the package's immediate parent holds the workspace
#' `Cargo.toml`, the same immediate-child layout `find_rpkg_subdir()` scans:
#' a package that merely sits somewhere inside a Rust repository stays
#' standalone, so the upgrade never writes to an unrelated ancestor's files.
#'
#' @param path Path passed to `upgrade_miniextendr_package()`.
#' @param rpkg_subdir Explicit R package subdirectory, or `NULL`.
#' @return A list with `pkg` (absolute path to the R package), `root` (the
#'   monorepo workspace root, `NULL` for a standalone package) and `subdir`
#'   (the package directory relative to `root`, `NULL` for a standalone
#'   package).
#' @noRd
upgrade_layout <- function(path, rpkg_subdir = NULL) {
  path <- normalizePath(path, mustWork = FALSE)
  standalone <- list(pkg = path, root = NULL, subdir = NULL)
  if (!identical(detect_project_type(path), "monorepo")) return(standalone)

  if (file.exists(file.path(path, "configure.ac"))) {
    # `path` is the R package itself.
    root <- dirname(path)
    if (!file.exists(file.path(root, "Cargo.toml"))) return(standalone)
    return(list(pkg = path, root = root, subdir = basename(path)))
  }

  # `path` is the workspace root -- resolve the R package subdirectory.
  subdir <- rpkg_subdir %||% find_rpkg_subdir(path)
  if (is.null(subdir)) {
    cli::cli_abort(c(
      "Could not find an rpkg subdirectory in {.path {path}}.",
      "i" = "Pass {.code rpkg_subdir = '<name>'} explicitly.",
      "i" = "Expected a subdirectory with a miniextendr {.path configure.ac}."
    ))
  }
  list(pkg = file.path(path, subdir), root = path, subdir = subdir)
}

#' Point out files an older scaffold leaves behind
#'
#' The upgrade never deletes files, so its closing summary lists them for the
#' user to remove (#1669):
#' - `tools/wrapper-freshness.R` and `tools/wrapper-inputs.rds`, from the
#'   pre-shipped wrapper fast path #1656 replaced with `tools/write-wrappers.R`;
#' - a git-tracked `src/<pkg>-win.def`. configure generates it and the
#'   scaffold now gitignores it, but an ignore rule does not untrack a file
#'   committed before the rule existed.
#'
#' @param proj_dir Package directory to inspect.
#' @return Called for its messages; returns `NULL` invisibly.
#' @noRd
report_upgrade_leftovers <- function(proj_dir) {
  unused <- c("tools/wrapper-freshness.R", "tools/wrapper-inputs.rds")
  unused <- unused[file.exists(file.path(proj_dir, unused))]
  if (length(unused) > 0L) {
    cli::cli_alert_info("No longer used by the package; you can delete {.path {unused}}")
  }

  if (!nzchar(Sys.which("git"))) return(invisible())
  # shQuote: system2() goes through a shell, and git, not the shell, should
  # expand the pathspec (it also matches tracked files missing on disk).
  tracked <- run_command("git", c("ls-files", "--", shQuote("src/*-win.def")),
                         wd = proj_dir)
  # Outside a git repository the command fails; there is nothing to untrack.
  if (!is.null(attr(tracked, "status"))) return(invisible())
  tracked <- tracked[nzchar(tracked)]
  if (length(tracked) > 0L) {
    untrack <- paste(c("git rm --cached", tracked), collapse = " ")
    cli::cli_alert_info(
      "Tracked in git but generated by configure (and gitignored): {.path {tracked}}"
    )
    cli::cli_bullets(c(" " = "Untrack it with {.code {untrack}}"))
  }
  invisible()
}

#' Check that scaffolding files are clean in git
#'
#' Uses `git status --porcelain` to inspect build system files that will be
#' overwritten during upgrade. Aborts if any have uncommitted changes.
#'
#' Every git invocation runs *in* `proj_dir` (via `run_command(wd = )`), not the
#' caller's working directory. `with_project()` activates the usethis project
#' without changing the working directory (`setwd = FALSE`), so probing git in
#' `getwd()` would inspect an unrelated repo -- or no repo at all, which would
#' silently skip this guard and let the upgrade overwrite the target's
#' uncommitted files.
#'
#' @param proj_dir Package directory to inspect (default `usethis::proj_get()`).
#'   The relative `scaffolding_files` pathspecs are interpreted against it.
#' @noRd
check_scaffolding_clean <- function(proj_dir = usethis::proj_get()) {
  # Bail out if git is not available
  if (!nzchar(Sys.which("git"))) return(invisible())

  # Bail out if not in a git repo
  repo <- tryCatch({
    out <- run_command("git", c("rev-parse", "--show-toplevel"), wd = proj_dir)
    if (!is.null(attr(out, "status"))) NULL else out
  }, error = function(e) NULL)
  if (is.null(repo)) return(invisible())

  # Files that upgrade will overwrite
  scaffolding_files <- c(
    "src/stub.c",
    "src/r_shim.h",
    "src/rust/build.rs",
    "src/Makevars.in",
    "src/win.def.in",
    "inst/include/mx_abi.h",
    "bootstrap.R",
    "cleanup",
    "cleanup.win",
    "cleanup.ucrt",
    "configure.win",
    "configure.ucrt",
    "tools/config.guess",
    "tools/config.sub",
    ".Rbuildignore",
    ".gitignore",
    ".gitattributes"
  )

  out <- run_command("git", c("status", "--porcelain", "--", scaffolding_files),
                     wd = proj_dir)
  if (is.null(out) || length(out) == 0) return(invisible())

  cli::cli_abort(c(
    "Scaffolding files have uncommitted changes.",
    "i" = "Commit or stash your changes first, or use {.code allow_dirty = TRUE} to force.",
    paste(" ", out)
  ))
}

#' Upgrade .gitignore patterns
#'
#' Adds current miniextendr patterns (usethis deduplicates) and removes
#' known-obsolete entries that are no longer needed (files now tracked in git).
#'
#' @param subdir Optional template subdirectory, passed to
#'   `use_miniextendr_gitignore()` (`"rpkg"` in a monorepo).
#' @noRd
upgrade_gitignore <- function(subdir = NULL) {
  # Add current patterns (usethis handles deduplication)
  use_miniextendr_gitignore(subdir = subdir)

  # Remove obsolete entries that are now tracked in git
  gitignore_path <- usethis::proj_path(".gitignore")
  if (!fs::file_exists(gitignore_path)) return(invisible())

  lines <- readLines(gitignore_path, warn = FALSE)
  obsolete <- c("src/entrypoint.c", "src/entrypoint.c.in", "src/mx_abi.c",
                "src/mx_abi.c.in", "src/rust/document.rs",
                # Superseded by the correctly-anchored `src/rust/.cargo/config.toml`
                # (#1226); the bare pattern never matched the nested path.
                ".cargo/config.toml",
                # Added by #1523, retired with the wrapper fast path by #1656.
                "# Generated wrapper provenance, shipped with the package tarball",
                "/tools/wrapper-inputs.rds")

  # Remove exact matches (trimmed)
  trimmed <- trimws(lines)
  keep <- !(trimmed %in% obsolete)

  if (sum(!keep) > 0) {
    removed <- trimmed[!keep]
    writeLines(lines[keep], gitignore_path)
    for (entry in removed) {
      cli::cli_alert_success("Removed obsolete .gitignore entry: {.val {entry}}")
    }
  }

  invisible()
}

#' Check configure.ac for drift
#'
#' Heuristic check for key structural elements that indicate the configure.ac
#' is up to date. Warns if missing.
#'
#' @noRd
check_configure_ac_drift <- function() {
  configure_ac <- usethis::proj_path("configure.ac")
  if (!fs::file_exists(configure_ac)) return(invisible())

  content <- readLines(configure_ac, warn = FALSE)
  text <- paste(content, collapse = "\n")

  missing <- character()
  if (!grepl("CARGO_STATICLIB_NAME", text, fixed = TRUE)) {
    missing <- c(missing, "CARGO_STATICLIB_NAME substitution")
  }
  if (!grepl("AC_CONFIG_AUX_DIR", text, fixed = TRUE)) {
    missing <- c(missing, "AC_CONFIG_AUX_DIR([tools])")
  }
  if (!grepl("CARGO_TARGET_DIR", text, fixed = TRUE)) {
    missing <- c(missing, "CARGO_TARGET_DIR setup")
  }

  if (length(missing) > 0) {
    cli::cli_warn(c(
      "configure.ac may be outdated (missing: {paste(missing, collapse = ', ')})",
      "i" = "Re-run with {.code configure_ac = TRUE} to replace it with the current template.",
      "!" = "This will overwrite any custom feature flags in configure.ac."
    ))
  }

  invisible()
}
