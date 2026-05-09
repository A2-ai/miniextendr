# Package upgrade functions

#' Upgrade a miniextendr package
#'
#' Comprehensively upgrades an existing miniextendr package to the latest
#' build system templates, vendored crates, and package metadata. This replaces
#' the old `miniextendr_update()` with a more thorough upgrade that covers
#' configure.ac, DESCRIPTION, .gitignore, .Rbuildignore, build.rs, and
#' generated C files.
#'
#' User-authored files (lib.rs, Cargo.toml, R/ code) are never touched.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
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
                                         version = "main",
                                         local_path = NULL,
                                         configure_ac = FALSE,
                                         autoconf = TRUE,
                                         allow_dirty = FALSE) {
  with_project(path)

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
    check_scaffolding_clean()
  }

  # --- Build system templates ---
  cli::cli_h2("Updating build system templates")
  use_miniextendr_stub()
  use_miniextendr_makevars()
  use_miniextendr_mx_abi()
  use_miniextendr_build_rs()
  use_miniextendr_bootstrap()
  use_miniextendr_cleanup()
  use_miniextendr_configure_win()
  use_miniextendr_config_scripts()

  # --- Package metadata ---
  cli::cli_h2("Updating package metadata")
  use_miniextendr_description()
  use_miniextendr_rbuildignore()
  upgrade_gitignore()

  # --- configure.ac ---
  if (configure_ac) {
    cli::cli_h2("Replacing configure.ac")
    use_miniextendr_configure()
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
  cli::cli_alert_info("Next steps:")
  cli::cli_bullets(c(
    " " = "Review changes with {.code git diff}",
    " " = "Run {.code minirextendr::miniextendr_build()} to rebuild"
  ))

  invisible(TRUE)
}

#' Check that scaffolding files are clean in git
#'
#' Uses `git status --porcelain` to inspect build system files that will be
#' overwritten during upgrade. Aborts if any have uncommitted changes.
#'
#' @noRd
check_scaffolding_clean <- function() {
  # Bail out if git is not available
  if (!nzchar(Sys.which("git"))) return(invisible())

  # Bail out if not in a git repo
  repo <- tryCatch({
    out <- system2("git", c("rev-parse", "--show-toplevel"),
                   stdout = TRUE, stderr = TRUE)
    if (!is.null(attr(out, "status"))) NULL else out
  }, error = function(e) NULL)
  if (is.null(repo)) return(invisible())

  # Files that upgrade will overwrite
  scaffolding_files <- c(
    "src/stub.c",
    "src/r_shim.h",
    "src/cdylib-exports.def",
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
    ".gitignore"
  )

  out <- system2("git", c("status", "--porcelain", "--", scaffolding_files),
                 stdout = TRUE, stderr = TRUE)
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
#' @noRd
upgrade_gitignore <- function() {
  # Add current patterns (usethis handles deduplication)
  use_miniextendr_gitignore()

  # Remove obsolete entries that are now tracked in git
  gitignore_path <- usethis::proj_path(".gitignore")
  if (!fs::file_exists(gitignore_path)) return(invisible())

  lines <- readLines(gitignore_path, warn = FALSE)
  obsolete <- c("src/entrypoint.c", "src/entrypoint.c.in", "src/mx_abi.c",
                "src/mx_abi.c.in", "src/rust/document.rs")

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
  if (!grepl("PACKAGE_TARNAME_RS", text, fixed = TRUE)) {
    missing <- c(missing, "PACKAGE_TARNAME_RS substitution")
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
