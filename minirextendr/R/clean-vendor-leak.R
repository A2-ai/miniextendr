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
#' normal source-mode dev iteration. If `Cargo.toml` still contains paths into
#' `vendor/` (for example after `cargo revendor --freeze`), the affected
#' dependency and patch entries are reported with repair instructions. Restore
#' their original source paths from before freezing; those paths cannot be
#' reconstructed from a frozen manifest alone.
#'
#' @param path Path to the R package root, or `"."` to use the current
#'   directory.
#' @return Invisibly returns `TRUE` if the file was removed, `FALSE` if it
#'   was already absent.
#' @seealso [miniextendr_vendor()] to create the tarball intentionally,
#'   [miniextendr_doctor()] to detect this and other configuration issues.
#' @export
miniextendr_clean_vendor_leak <- function(path = ".") {
  with_project(path)
  frozen <- frozen_manifest_entries(usethis::proj_get())
  if (length(frozen)) report_frozen_manifest(frozen)
  tarball <- tryCatch(
    usethis::proj_path("inst", "vendor.tar.xz"),
    error = function(e) NULL
  )
  if (is.null(tarball) || !fs::file_exists(tarball)) {
    if (!length(frozen)) {
      cli::cli_alert_success("No {.path inst/vendor.tar.xz} leak to clean.")
    }
    return(invisible(FALSE))
  }
  fs::file_delete(tarball)
  cli::cli_alert_success("Removed {.path inst/vendor.tar.xz} (tarball-mode leak).")
  if (!length(frozen)) {
    cli::cli_alert_info(
      "Run {.code miniextendr_configure()} (or {.code bash ./configure}) to regenerate build files in source mode."
    )
  }
  invisible(TRUE)
}

# Freeze writes relative vendor paths in dependency and patch tables. Reuse the
# existing Cargo dependency parser after selecting each relevant table family.
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

report_frozen_manifest <- function(entries) {
  cli::cli_alert_warning("Cargo.toml contains vendor-bound paths, as written by {.code cargo revendor --freeze}:")
  for (entry in entries) {
    cli::cli_bullets(c("x" = "[{entry$section}] {entry$crate}: {.path {entry$path}}"))
  }
  cli::cli_alert_info("Removing the tarball does not restore these source paths. Restore the original dependency paths and patch entries from before freezing (review {.code git diff -- src/rust/Cargo.toml} when tracked), then run {.code miniextendr_configure()}.")
  invisible(entries)
}
