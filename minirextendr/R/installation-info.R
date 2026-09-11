# Identify the copy actually loaded, including when rv and user libraries differ.
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

report_minirextendr_installation <- function() {
  copies <- minirextendr_installations()
  active <- copies[[1L]]
  revision <- if (is.na(active$sha)) "" else paste0(", git ", active$sha)
  cli::cli_alert_info("minirextendr {active$version}{revision} loaded from {.path {active$path}}")
  invisible(copies)
}
