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
#' The template is the source of truth for the build-system files it owns
#' (`src/stub.c`, `src/Makevars.in`, `inst/include/mx_abi.h`, the `tools/`
#' scripts, ...): the upgrade rewrites them in full, so a local edit to one,
#' committed or not, is replaced; the `allow_dirty` check only stops the
#' upgrade over uncommitted edits. To make the replacements visible, the closing
#' summary lists every file whose content the upgrade changed or added; review
#' them with `git diff` (and `git status` for added files) before committing.
#'
#' A `tools/config.guess` or `tools/config.sub` whose `timestamp=` line is
#' newer than the bundled copy's is kept. The closing summary also lists files
#' the package no longer uses (`tools/wrapper-freshness.R`,
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
#'   configure.ac with feature flags. When `FALSE`, the upgrade first checks
#'   that the existing configure.ac substitutes every `@VAR@` placeholder in
#'   the new `src/Makevars.in` and `src/win.def.in`. If any is missing, it
#'   aborts before writing a file, names the missing variables and the
#'   template configure.ac that sets them, and leaves the package unchanged:
#'   merge those blocks into configure.ac, or use `TRUE`. The check asks
#'   autoconf for the exact list and falls back to an approximate scan of
#'   `AC_SUBST()` / `AC_ARG_VAR()` when autoconf is unavailable. Later, the
#'   upgrade also compares the existing file with the current template and
#'   warns about differences, including custom edits. Review those differences
#'   or use `TRUE` to replace the file.
#' @param autoconf Logical. If `TRUE` (default) and `autoconf` is available,
#'   regenerates the configure script after upgrading.
#' @param allow_dirty Logical. If `FALSE` (default), aborts when scaffolding
#'   files have uncommitted changes in git, to prevent accidental data loss.
#'   Set to `TRUE` to force the upgrade even with dirty files.
#' @return Invisibly, a list of three character vectors naming the files the
#'   upgrade touched: `changed` (existed before, content differs), `added`
#'   (did not exist before) and `removed` (existed before, gone after). Paths
#'   are relative to the package directory for a standalone package and to the
#'   workspace root for a monorepo, as `git diff` prints them there. All three
#'   are empty when the package already matched the templates.
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

  # --- configure.ac substitutions ---
  # A retained configure.ac must substitute every @VAR@ of the Makevars.in and
  # win.def.in written below; check before the first write (#1733).
  if (!configure_ac) {
    check_configure_ac_substitutions(monorepo)
  }

  # Fingerprint every file the upgrade may write, after the checks that abort
  # and before the first write, so the summary can name what changed (#1713).
  owned <- upgrade_owned_files(layout)
  before <- hash_files(owned)

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
  if (monorepo) upgrade_root_gitignore(layout$root, layout$subdir)
  use_miniextendr_gitattributes(subdir = tpl_subdir)

  # --- configure.ac ---
  if (configure_ac) {
    cli::cli_h2("Replacing configure.ac")
    use_miniextendr_configure(subdir = tpl_subdir)
  } else {
    check_configure_ac_drift(monorepo)
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
  changes <- upgrade_changes(before, hash_files(owned))
  cli::cli_h1("Upgrade complete!")
  report_upgrade_changes(changes)
  report_upgrade_leftovers(resolved_path)
  cli::cli_alert_info("Next steps:")
  cli::cli_bullets(c(
    " " = "Run {.code minirextendr::miniextendr_build()} to rebuild"
  ))

  invisible(changes)
}

#' Files an upgrade may write
#'
#' Every file `upgrade_miniextendr_package()` writes, whether it rewrites the
#' file from a template (`src/stub.c`, `inst/include/mx_abi.h`, the `tools/`
#' scripts, `configure.ac` with `configure_ac = TRUE`, ...), regenerates it
#' (`configure`, by autoconf), merges entries into it (`DESCRIPTION`, the
#' ignore files, `.gitattributes`) or creates it when missing (`LICENSE`).
#' In a monorepo the
#' workspace-root `.gitignore` is included too. Files a particular run leaves
#' alone, such as `configure.ac` with `configure_ac = FALSE`, are listed
#' anyway: their content does not change, so the summary skips them. A file
#' the upgrade starts writing has to be added here, or the summary will not
#' report it.
#'
#' @param layout The list `upgrade_layout()` returns.
#' @return A character vector of absolute paths, named by the path shown to the
#'   user: relative to the package directory for a standalone package, and to
#'   the workspace root for a monorepo.
#' @keywords internal
upgrade_owned_files <- function(layout) {
  pkg_files <- c(
    "src/stub.c",
    "src/r_shim.h",
    "src/Makevars.in",
    "src/Makevars.win",
    "src/win.def.in",
    "src/rust/build.rs",
    "inst/include/mx_abi.h",
    "bootstrap.R",
    "cleanup",
    "cleanup.win",
    "cleanup.ucrt",
    "configure.ac",
    "configure",
    "configure.win",
    "configure.ucrt",
    "tools/config.guess",
    "tools/config.sub",
    "tools/vendor-cache.R",
    "tools/dev-bootstrap.R",
    "tools/write-wrappers.R",
    "tools/lock-shape-check.R",
    "tools/build-html-reference.R",
    "DESCRIPTION",
    "LICENSE",
    ".Rbuildignore",
    ".gitignore",
    ".gitattributes"
  )
  if (is.null(layout$root)) {
    return(stats::setNames(file.path(layout$pkg, pkg_files), pkg_files))
  }
  c(stats::setNames(file.path(layout$pkg, pkg_files),
                    file.path(layout$subdir, pkg_files)),
    ".gitignore" = file.path(layout$root, ".gitignore"))
}

#' MD5 fingerprints of files
#'
#' @param files Named character vector of file paths, as
#'   `upgrade_owned_files()` returns.
#' @return A character vector of MD5 hashes with the names of `files`; `NA`
#'   for a file that does not exist.
#' @keywords internal
hash_files <- function(files) {
  stats::setNames(unname(tools::md5sum(files)), names(files))
}

#' Classify the files an upgrade touched
#'
#' @param before,after Fingerprints of the same files from `hash_files()`,
#'   taken before the upgrade's first write and after its last one.
#' @return A list of character vectors of file names (the names of `before`):
#'   `changed` (existed before and after, content differs), `added` (absent
#'   before, present after) and `removed` (present before, absent after).
#' @keywords internal
upgrade_changes <- function(before, after) {
  existed <- !is.na(before)
  exists <- !is.na(after)
  list(
    changed = names(before)[existed & exists & before != after],
    added = names(before)[!existed & exists],
    removed = names(before)[existed & !exists]
  )
}

#' Report the files an upgrade changed
#'
#' Prints the changed, added and removed files, one per line, then points at
#' `git diff`: the template replaced any local edit to a file it owns, and a
#' committed edit gets past the uncommitted-changes check (#1713). Prints one
#' line when nothing changed.
#'
#' @param changes The list `upgrade_changes()` returns.
#' @return Called for its messages; returns `NULL` invisibly.
#' @keywords internal
report_upgrade_changes <- function(changes) {
  if (sum(lengths(changes)) == 0L) {
    cli::cli_alert_success("No scaffold files changed; the package already matched the templates.")
    return(invisible())
  }
  groups <- c(changed = "Changed", added = "Added", removed = "Removed")
  for (group in names(groups)) {
    files <- changes[[group]]
    if (length(files) == 0L) next
    label <- groups[[group]]
    cli::cli_alert_info("{label} {length(files)} file{?s}:")
    for (file in files) cli::cli_bullets(c("*" = "{.path {file}}"))
  }
  status <- if (length(changes$added) > 0L) {
    " (and {.code git status} for the added files)"
  } else {
    ""
  }
  cli::cli_alert_warning(paste0(
    "Review them with {.code git diff}", status, " before committing: ",
    "the template replaced any local edits to the files it owns, committed ones included."
  ))
  invisible()
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
#' @keywords internal
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
#' @keywords internal
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

#' Add the monorepo root template's entries to the workspace `.gitignore`
#'
#' The package-level `upgrade_gitignore()` never reaches the workspace root
#' (#1670). This adds the patterns of `templates/monorepo/gitignore`, with
#' `{{rpkg_name}}` rendered as the package subdirectory, the way
#' `use_miniextendr_gitignore()` does: entries are only added, and existing
#' lines, user lines included, stay. No root entry has been retired yet, so
#' there is no obsolete list here. Call it with the "monorepo" template type
#' active, as `upgrade_miniextendr_package()` does.
#'
#' @param root Absolute path to the monorepo workspace root.
#' @param rpkg_name The R package subdirectory, relative to `root`.
#' @return `NULL` invisibly; called for its side effect of editing
#'   `<root>/.gitignore`.
#' @keywords internal
upgrade_root_gitignore <- function(root, rpkg_name) {
  patterns <- gsub("{{rpkg_name}}", rpkg_name, mx_ignore_patterns("gitignore"),
                   fixed = TRUE)
  with_project(root)
  usethis::use_git_ignore(patterns, directory = ".")
  invisible()
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
#' @keywords internal
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
#' @return `NULL` invisibly when the files are clean, or when git is missing
#'   or `proj_dir` is not in a git repository; otherwise aborts, listing the
#'   `git status --porcelain` lines.
#' @keywords internal
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
    "src/Makevars.win",
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
#' @return `NULL` invisibly; called for its side effect of editing the active
#'   project's `.gitignore`.
#' @keywords internal
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
#' Compare the complete file with the current template, rendered for this
#' package. Custom edits also produce a warning; leave the file untouched so
#' users can review and retain those edits when updating the build system.
#'
#' @param monorepo Whether the package is the R package of a monorepo, which
#'   selects `templates/monorepo/rpkg/configure.ac` over
#'   `templates/rpkg/configure.ac`. Defaults to what `upgrade_layout()` says
#'   about the active project, so the drift check and the upgrade agree on the
#'   template set (#1720). The template type a previous scaffolding call left
#'   active in this R session plays no part.
#' @return `NULL` invisibly; called for the warning it raises when the
#'   active project's `configure.ac` differs from the template (nothing
#'   happens when the project has no `configure.ac`).
#' @keywords internal
check_configure_ac_drift <- function(monorepo = !is.null(upgrade_layout(usethis::proj_get())$root)) {
  configure_ac <- usethis::proj_path("configure.ac")
  if (!fs::file_exists(configure_ac)) return(invisible())

  content <- readLines(configure_ac, warn = FALSE)
  template <- readLines(configure_ac_template(monorepo), warn = FALSE)
  # configure.ac has one template variable: the package name in AC_INIT.
  template <- gsub("{{package}}", get_package_name(), template, fixed = TRUE)

  if (!identical(content, template)) {
    cli::cli_warn(c(
      "configure.ac differs from the current template and was left unchanged.",
      "i" = "Differences may be custom edits or outdated build-system logic; review them before rebuilding.",
      "i" = "Re-run with {.code configure_ac = TRUE} to replace it with the current template.",
      "!" = "This will overwrite any custom feature flags in configure.ac."
    ))
  }

  invisible()
}

#' Path of the configure.ac template for a layout
#'
#' @param monorepo `TRUE` for the R package of a monorepo, `FALSE` for a
#'   standalone package.
#' @return Path of the installed `templates/monorepo/rpkg/configure.ac` or
#'   `templates/rpkg/configure.ac`.
#' @keywords internal
configure_ac_template <- function(monorepo) {
  layout <- if (monorepo) file.path("monorepo", "rpkg") else "rpkg"
  system.file("templates", layout, "configure.ac",
              package = "minirextendr", mustWork = TRUE)
}
