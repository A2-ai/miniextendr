# Fast Cargo.lock drift repair

#' Restore `src/rust/Cargo.lock` to tarball-shape
#'
#' The committed `src/rust/Cargo.lock` must be in **tarball-shape** (no
#' `checksum = "..."` lines, no `path+...` source entries for the
#' `miniextendr-{api,lint,macros}` framework crates). Day-to-day
#' `devtools::install()`, `cargo build`, or `devtools::document()` silently
#' rewrites the lock in source-shape: checksums come back for crates.io deps
#' and framework crates switch to `path+file:///...` entries.
#'
#' `miniextendr_repair_lock()` is a sub-second lock-only repair:
#' 1. Moves `.cargo/config.toml` aside temporarily so `cargo update` runs
#'    without the `[patch."git+url"]` override that would re-introduce
#'    `path+` entries.
#' 2. Runs `cargo update` to resolve the lockfile against the bare git URLs.
#' 3. Strips any remaining `checksum = "..."` lines.
#' 4. Restores `.cargo/config.toml` on exit (success or error).
#'
#' This is the lightweight alternative to [miniextendr_vendor()] -- use it
#' during dev iteration when you only need a clean lock, not a fresh
#' `inst/vendor.tar.xz`. Use [miniextendr_vendor()] only before
#' `R CMD build` for CRAN submission.
#'
#' For the full invariants and reasoning, see the
#' [Cargo.lock shape](https://a2-ai.github.io/miniextendr/manual/cargo-lock-shape/)
#' page in the framework docs.
#'
#' @param path Path to the R package root, or `"."` to use the current
#'   directory.
#' @param quiet If `TRUE`, suppress informational messages. Errors are always
#'   printed.
#' @return Invisibly returns `TRUE` if the lock was repaired, `FALSE` if it
#'   was already in tarball-shape (no action taken).
#' @seealso [miniextendr_vendor()] for a full vendor rebuild,
#'   [miniextendr_doctor()] to detect this and other configuration issues.
#' @export
miniextendr_repair_lock <- function(path = ".", quiet = FALSE) {
  with_project(path)

  lockfile <- tryCatch(
    usethis::proj_path("src", "rust", "Cargo.lock"),
    error = function(e) NULL
  )
  if (is.null(lockfile) || !fs::file_exists(lockfile)) {
    cli::cli_abort(c(
      "{.path src/rust/Cargo.lock} not found",
      "i" = "Run {.code miniextendr_configure()} first to set up the Rust project"
    ))
  }

  lock_lines <- readLines(lockfile, warn = FALSE)

  has_checksums <- any(grepl("^checksum = ", lock_lines))
  has_path_sources <- any(grepl('^source = "path\\+', lock_lines))

  if (!has_checksums && !has_path_sources) {
    if (!quiet) {
      cli::cli_alert_success("{.path src/rust/Cargo.lock} is already in tarball-shape.")
    }
    return(invisible(FALSE))
  }

  if (!quiet) {
    n_checksums <- sum(grepl("^checksum = ", lock_lines))
    n_path <- sum(grepl('^source = "path\\+', lock_lines))
    cli::cli_alert_info(
      "Repairing {.path src/rust/Cargo.lock}: {n_path} {.code path+} entr{?y/ies}, {n_checksums} {.code checksum =} line{?s}."
    )
  }

  # Move .cargo/config.toml aside so cargo update doesn't see the
  # [patch."git+url"] override that would rewrite framework crates to path+.
  cargo_cfg <- tryCatch(
    usethis::proj_path("src", "rust", ".cargo", "config.toml"),
    error = function(e) NULL
  )
  cargo_cfg_bak <- if (!is.null(cargo_cfg)) paste0(cargo_cfg, ".tmp_repair_lock") else NULL

  if (!is.null(cargo_cfg) && fs::file_exists(cargo_cfg)) {
    fs::file_move(cargo_cfg, cargo_cfg_bak)
    withr::defer({
      if (!is.null(cargo_cfg_bak) && fs::file_exists(cargo_cfg_bak)) {
        fs::file_move(cargo_cfg_bak, cargo_cfg)
      }
    })
  }

  cargo_toml <- tryCatch(
    usethis::proj_path("src", "rust", "Cargo.toml"),
    error = function(e) NULL
  )
  if (is.null(cargo_toml) || !fs::file_exists(cargo_toml)) {
    cli::cli_abort("{.path src/rust/Cargo.toml} not found")
  }

  if (!quiet) cli::cli_alert("Running {.code cargo update}...")

  result <- run_with_logging(
    "cargo",
    args = c("update", "--manifest-path", cargo_toml),
    log_prefix = "cargo-update-repair-lock",
    wd = usethis::proj_get()
  )
  check_result(result, "cargo update")

  # Strip checksum lines — vendored crates ship with empty
  # .cargo-checksum.json; registry checksums in the lock cause offline
  # install to abort with "the listed checksum of '<crate>' has changed".
  updated_lines <- readLines(lockfile, warn = FALSE)
  stripped_lines <- updated_lines[!grepl("^checksum = ", updated_lines)]

  # Verify no path+ entries remain. If cargo resolved without the patch
  # override, framework crates should now be back to git+https:// source.
  remaining_path <- stripped_lines[grepl('^source = "path\\+', stripped_lines)]
  if (length(remaining_path) > 0) {
    cli::cli_abort(c(
      "{.path src/rust/Cargo.lock} still has {.code path+} source entries after repair:",
      stats::setNames(remaining_path, rep("x", length(remaining_path))),
      "i" = paste0(
        "This may mean your {.path src/rust/Cargo.toml} declares path dependencies ",
        "directly (not via {.code [patch]}). Those cannot be stripped automatically."
      )
    ))
  }

  writeLines(stripped_lines, lockfile)

  if (!quiet) {
    cli::cli_alert_success(
      "{.path src/rust/Cargo.lock} restored to tarball-shape."
    )
  }

  invisible(TRUE)
}
