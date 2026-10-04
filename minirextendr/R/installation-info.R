#' Installed copies of minirextendr
#'
#' Identifies the copy of minirextendr actually loaded, including when rv and
#' user libraries differ. `report_minirextendr_installation()` prints the
#' loaded copy's version (and git revision, when installed from a remote) and
#' path; the build workflows call it first so a log shows which minirextendr
#' produced it.
#'
#' @param active_path Install path of the loaded minirextendr namespace.
#' @param lib_paths Library paths searched for further installed copies.
#' @return `minirextendr_installations()`: a list with one
#'   `list(path, version, sha)` per distinct installed copy (the loaded one
#'   first; `sha` is the DESCRIPTION's `RemoteSha`, `NA` when absent).
#'   `report_minirextendr_installation()`: that list, invisibly.
#' @keywords internal
minirextendr_installations <- function(
    active_path = getNamespaceInfo(asNamespace("minirextendr"), "path"),
    lib_paths = .libPaths()) {
  paths <- unique(normalizePath(c(active_path, file.path(lib_paths, "minirextendr")),
                                mustWork = FALSE))
  paths <- paths[file.exists(file.path(paths, "DESCRIPTION"))]
  lapply(paths, function(path) {
    desc <- read.dcf(file.path(path, "DESCRIPTION"), fields = c("Version", "RemoteSha"))
    list(path = path, version = unname(desc[1, "Version"]),
         sha = unname(desc[1, "RemoteSha"]))
  })
}

#' @rdname minirextendr_installations
report_minirextendr_installation <- function() {
  copies <- minirextendr_installations()
  active <- copies[[1L]]
  revision <- if (is.na(active$sha)) "" else paste0(", git ", active$sha)
  cli::cli_alert_info("minirextendr {active$version}{revision} loaded from {.path {active$path}}")
  invisible(copies)
}
