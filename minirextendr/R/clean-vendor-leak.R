# Recovery helper for leaked inst/vendor.tar.xz

#' Remove a leaked inst/vendor.tar.xz
#'
#' `inst/vendor.tar.xz` is the single signal that flips `./configure` into
#' offline tarball mode. Once that file exists, every subsequent
#' `R CMD INSTALL`, `devtools::install()`, or `devtools::document()` call
#' builds against the vendored snapshot rather than pulling live workspace
#' or network sources.
#'
#' This is intentional during CRAN submission prep (run
#' [miniextendr_vendor()] first, then `R CMD build`). It becomes a trap
#' when a prior `R CMD build` or check run leaves the file behind in your
#' source tree.
#'
#' Call this function after any unexpected tarball-mode install to restore
#' normal source-mode dev iteration. If `src/rust/Cargo.toml` is still frozen
#' (its dependency and patch entries point into `vendor/`, as written by
#' `cargo revendor --freeze`) and the pre-freeze snapshot that cargo-revendor
#' leaves next to it, `src/rust/.Cargo.toml.prefreeze`, is present, the
#' manifest is restored from that snapshot byte for byte and the snapshot is
#' removed. Without a snapshot the affected entries are reported with repair
#' instructions: their original source paths cannot be reconstructed from a
#' frozen manifest alone. `Cargo.lock` is not touched; cargo re-resolves it on
#' the next build (or restore it from version control together with the
#' manifest).
#'
#' @param path Path to the R package root, or `"."` to use the current
#'   directory. From a monorepo workspace root (no `DESCRIPTION`, one
#'   immediate subdirectory holding a miniextendr package) the package
#'   subdirectory is used; any other directory that is not an R package is an
#'   error naming the directory to pass.
#' @return Invisibly returns `TRUE` if a leaked tarball was removed or the
#'   manifest was restored from its snapshot, `FALSE` if there was nothing to
#'   clean.
#' @seealso [miniextendr_vendor()] to create the tarball intentionally,
#'   [miniextendr_doctor()] to detect this and other configuration issues.
#' @export
miniextendr_clean_vendor_leak <- function(path = ".") {
  with_project(resolve_package_dir(path, "leak-check"))
  root <- usethis::proj_get()
  restored <- restore_prefreeze_manifest(root)
  frozen <- frozen_manifest_entries(root)
  if (length(frozen)) report_frozen_manifest(frozen)
  tarball <- tryCatch(
    usethis::proj_path("inst", "vendor.tar.xz"),
    error = function(e) NULL
  )
  removed <- !is.null(tarball) && fs::file_exists(tarball)
  if (removed) {
    fs::file_delete(tarball)
    cli::cli_alert_success("Removed {.path inst/vendor.tar.xz} (tarball-mode leak).")
  }
  if (!removed && !restored && !length(frozen)) {
    cli::cli_alert_success("No {.path inst/vendor.tar.xz} leak to clean.")
  }
  if ((removed || restored) && !length(frozen)) {
    cli::cli_alert_info(
      "Run {.code miniextendr_configure()} (or {.code bash ./configure}) to regenerate build files in source mode."
    )
  }
  invisible(removed || restored)
}

#' Frozen-manifest helpers
#'
#' `cargo revendor --freeze` rewrites `src/rust/Cargo.toml` so its
#' dependency and patch entries point into `vendor/`, after saving the
#' original as `src/rust/.Cargo.toml.prefreeze` (#1509). These helpers find,
#' report and undo that state for [miniextendr_clean_vendor_leak()] and
#' [miniextendr_doctor()].
#'
#' @param root,path Package root directory.
#' @param entries Frozen entries, as returned by `frozen_manifest_entries()`.
#' @param snapshot Whether `src/rust/.Cargo.toml.prefreeze` is present,
#'   which changes the advice from "repair by hand" to "run
#'   `miniextendr_clean_vendor_leak()`".
#' @name prefreeze_manifest
#' @keywords internal
NULL

#' @rdname prefreeze_manifest
#' @details `prefreeze_sidecar_path()`: path of the pre-freeze snapshot
#'   `cargo revendor --freeze` writes next to the manifest before rewriting
#'   it. Gitignored and Rbuildignored by the scaffold; deleted by
#'   `restore_prefreeze_manifest()` once the manifest is restored.
#' @return `prefreeze_sidecar_path()`: the snapshot path (which may not
#'   exist).
prefreeze_sidecar_path <- function(root) {
  file.path(root, "src", "rust", ".Cargo.toml.prefreeze")
}

#' @rdname prefreeze_manifest
#' @details `restore_prefreeze_manifest()`: restores `src/rust/Cargo.toml`
#'   from the pre-freeze snapshot when the manifest is still vendor-bound (or
#'   missing), and drops a snapshot that outlived a manifest already restored
#'   by other means (`git checkout`, `miniextendr_build()`'s own in-memory
#'   restore).
#' @return `restore_prefreeze_manifest()`: `TRUE` only when the manifest was
#'   rewritten.
restore_prefreeze_manifest <- function(root) {
  sidecar <- prefreeze_sidecar_path(root)
  if (!file.exists(sidecar)) return(FALSE)
  manifest <- file.path(root, "src", "rust", "Cargo.toml")
  if (file.exists(manifest) && !length(frozen_manifest_entries(root))) {
    fs::file_delete(sidecar)
    cli::cli_alert_info(
      "Removed a stale {.path src/rust/.Cargo.toml.prefreeze}: {.path src/rust/Cargo.toml} is no longer vendor-bound."
    )
    return(FALSE)
  }
  fs::file_copy(sidecar, manifest, overwrite = TRUE)
  fs::file_delete(sidecar)
  cli::cli_alert_success(
    "Restored {.path src/rust/Cargo.toml} from its pre-freeze snapshot {.path src/rust/.Cargo.toml.prefreeze}."
  )
  TRUE
}

#' @rdname prefreeze_manifest
#' @details `frozen_manifest_entries()`: freeze writes relative vendor paths
#'   in dependency and patch tables. Reuses the Cargo dependency parser
#'   [parse_relative_path_deps()] after selecting each relevant table family
#'   (`[dependencies]`, `[build-dependencies]`, `[patch.crates-io]`).
#' @return `frozen_manifest_entries()`: a list with one
#'   `list(crate, path, section)` per entry whose path resolves into
#'   `<path>/vendor/`; empty when there is none or no manifest.
frozen_manifest_entries <- function(path) {
  manifest <- file.path(path, "src", "rust", "Cargo.toml")
  if (!file.exists(manifest)) return(list())
  lines <- readLines(manifest, warn = FALSE)
  vendor <- paste0(fs::path_norm(fs::path_abs(file.path(path, "vendor"))), "/")
  result <- list()
  for (section in c("dependencies", "build-dependencies", "patch.crates-io")) {
    selected <- vapply(lines, function(line) {
      header <- trimws(line)
      if (!startsWith(header, "[")) return(line)
      prefix <- paste0("[", section)
      if (startsWith(header, paste0(prefix, "]")) ||
          startsWith(header, paste0(prefix, "."))) {
        sub(prefix, "[dependencies", header, fixed = TRUE)
      } else {
        "[ignored]"
      }
    }, character(1))
    entries <- parse_relative_path_deps(selected)
    for (entry in entries) {
      resolved <- fs::path_norm(fs::path_abs(entry$path, start = dirname(manifest)))
      if (startsWith(resolved, vendor)) {
        entry$section <- section
        result <- c(result, list(entry))
      }
    }
  }
  result
}

#' @rdname prefreeze_manifest
#' @details `report_frozen_manifest()`: warns with one bullet per frozen
#'   entry, then the repair advice.
#' @return `report_frozen_manifest()`: `entries`, invisibly.
report_frozen_manifest <- function(entries, snapshot = FALSE) {
  cli::cli_alert_warning("Cargo.toml contains vendor-bound paths, as written by {.code cargo revendor --freeze}:")
  for (entry in entries) {
    cli::cli_bullets(c("x" = "[{entry$section}] {entry$crate}: {.path {entry$path}}"))
  }
  if (snapshot) {
    cli::cli_alert_info("The pre-freeze snapshot {.path src/rust/.Cargo.toml.prefreeze} is present: run {.code miniextendr_clean_vendor_leak()} to restore {.path src/rust/Cargo.toml} from it, then {.code miniextendr_configure()}.")
  } else {
    cli::cli_alert_info("Removing the tarball does not restore these source paths, and no pre-freeze snapshot ({.path src/rust/.Cargo.toml.prefreeze}) is present. Restore the original dependency paths and patch entries from before freezing (review {.code git diff -- src/rust/Cargo.toml} when tracked), then run {.code miniextendr_configure()}.")
  }
  invisible(entries)
}

#' Resolve the R package directory a package-level helper operates on
#'
#' `path` is returned as is when it holds a `DESCRIPTION`. A monorepo
#' workspace root has none: the miniextendr package subdirectory found by
#' `find_rpkg_subdir()` is used instead, as `upgrade_miniextendr_package()`
#' does. Anything else aborts, so a directory that is not an R package is never
#' reported as clean.
#'
#' @param path Path passed by the caller; `NULL` and `"."` mean the active
#'   project, as in `with_project()`.
#' @param action Short verb phrase for the monorepo message.
#' @return Absolute path to the R package directory.
#' @keywords internal
resolve_package_dir <- function(path, action) {
  if (is.null(path) || identical(path, ".")) {
    path <- tryCatch(usethis::proj_get(), error = function(e) ".")
  }
  path <- normalizePath(path, mustWork = FALSE)
  if (file.exists(file.path(path, "DESCRIPTION"))) return(path)

  subdir <- if (dir.exists(path)) find_rpkg_subdir(path)
  if (!is.null(subdir)) {
    cli::cli_alert_info(
      "Monorepo layout detected -- checking rpkg subdir {.path {subdir}} ({action})"
    )
    return(file.path(path, subdir))
  }

  candidates <- if (dir.exists(path)) {
    dirs <- list.dirs(path, full.names = FALSE, recursive = FALSE)
    dirs[file.exists(file.path(path, dirs, "DESCRIPTION"))]
  } else {
    character()
  }
  example <- sprintf(
    "miniextendr_clean_vendor_leak(\"%s\")",
    if (length(candidates) == 1L) candidates else "<package directory>"
  )
  cli::cli_abort(c(
    "{.path {path}} has no {.file DESCRIPTION}, so it is not an R package.",
    "i" = "Pass the package directory, e.g. {.code {example}}."
  ))
}
