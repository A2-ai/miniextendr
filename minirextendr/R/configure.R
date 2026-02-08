#' Run autoconf and configure
#'
#' Convenience wrapper that runs both `autoconf` (to regenerate the configure
#' script from configure.ac) and `./configure` (to generate Makevars and other
#' build files). Equivalent to running [miniextendr_autoconf()] followed by
#' [miniextendr_configure()], with the addition of `NOT_CRAN` environment
#' variable support.
#'
#' @param path Path to the R package root. Defaults to the current
#'   usethis project.
#' @param not_cran Logical. If `TRUE` (the default), sets `NOT_CRAN=true`
#'   when running `./configure`, which skips vendoring and enables
#'   development-mode symlinks.
#' @return Invisibly returns `TRUE` on success.
#' @export
mx_configure <- function(path = ".", not_cran = TRUE) {
  path <- normalizePath(path, mustWork = TRUE)

  # Check prerequisites
  check_autoconf()

  # Step 1: autoconf
  cli::cli_h2("Running autoconf")
  autoconf_result <- run_with_logging(
    "autoconf",
    args = c("-v", "-i", "-f"),
    log_prefix = "autoconf",
    wd = path
  )
  check_result(autoconf_result, "autoconf")

  # Make configure executable
  configure_path <- file.path(path, "configure")
  if (file.exists(configure_path)) {
    fs::file_chmod(configure_path, "755")
    cli::cli_alert_success("Generated {.path configure}")
  }

  # Step 2: ./configure
  cli::cli_h2("Running ./configure")
  env <- character()
  if (not_cran) {
    env <- c(NOT_CRAN = "true")
    cli::cli_alert_info("Setting {.envvar NOT_CRAN}={.val true}")
  }

  configure_result <- run_with_logging(
    "./configure",
    log_prefix = "configure",
    wd = path,
    env = env
  )
  check_result(configure_result, "./configure")

  cli::cli_alert_success("Configure complete")
  invisible(TRUE)
}
