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
#' normal source-mode dev iteration.
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
  tarball <- tryCatch(
    usethis::proj_path("inst", "vendor.tar.xz"),
    error = function(e) NULL
  )
  if (is.null(tarball) || !fs::file_exists(tarball)) {
    cli::cli_alert_success("No {.path inst/vendor.tar.xz} leak to clean.")
    return(invisible(FALSE))
  }
  fs::file_delete(tarball)
  cli::cli_alert_success("Removed {.path inst/vendor.tar.xz} (tarball-mode leak).")
  cli::cli_alert_info(
    "Run {.code miniextendr_configure()} (or {.code bash ./configure}) to regenerate build files in source mode."
  )
  invisible(TRUE)
}
