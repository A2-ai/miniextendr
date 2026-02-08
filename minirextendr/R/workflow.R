# Workflow helper functions

#' Run autoconf to generate configure script
#'
#' Runs `autoconf -vif` in the package root to regenerate the configure
#' script from configure.ac. Requires autoconf to be installed.
#'
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_autoconf <- function() {
  check_autoconf()

  cli::cli_alert("Running autoconf...")

  result <- run_with_logging(
    "autoconf",
    args = c("-v", "-i", "-f"),
    log_prefix = "autoconf",
    wd = usethis::proj_get()
  )

  check_result(result, "autoconf")

  # Make configure executable
  configure_path <- usethis::proj_path("configure")
  if (fs::file_exists(configure_path)) {
    fs::file_chmod(configure_path, "755")
    cli::cli_alert_success("Generated {.path configure}")
  }

  invisible(TRUE)
}

#' Run configure script
#'
#' Runs `./configure` in the package root to generate Makevars,
#' Cargo.toml, and other build files from templates.
#'
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_configure <- function() {
  configure_path <- usethis::proj_path("configure")

  if (!fs::file_exists(configure_path)) {
    abort(c(
      "configure script not found",
      "i" = "Run {.code minirextendr::miniextendr_autoconf()} first"
    ))
  }

  # Ensure configure is executable
  perms <- fs::file_info(configure_path)$permissions
  if (!grepl("x", as.character(perms))) {
    cli::cli_alert_info("Making {.path configure} executable")
    fs::file_chmod(configure_path, "755")
  }

  cli::cli_alert("Running ./configure...")

  result <- run_with_logging(
    "./configure",
    log_prefix = "configure",
    wd = usethis::proj_get(),
    env = devtools::r_env_vars()
  )

  check_result(result, "./configure")

  # Also mention config.log if it exists
  config_log <- usethis::proj_path("config.log")
  if (fs::file_exists(config_log)) {
    cli::cli_alert_info("Configure log also saved to: {.path config.log}")
  }

  cli::cli_alert_success("Generated build files")
  invisible(TRUE)
}

#' Generate R wrapper functions from Rust
#'
#' Runs the document binary to generate miniextendr_wrappers.R from
#' the Rust source. Requires the package to be built first.
#'
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_document <- function() {
  check_rust()

  cargo_toml <- usethis::proj_path("src", "rust", "Cargo.toml")
  if (!fs::file_exists(cargo_toml)) {
    abort(c(
      "Cargo.toml not found",
      "i" = "Run {.code minirextendr::miniextendr_configure()} first"
    ))
  }

  cli::cli_alert("Running document binary...")

  result <- run_with_logging(
    "cargo",
    args = c("run", "--bin", "document", "--release"),
    log_prefix = "document",
    wd = usethis::proj_path("src", "rust")
  )

  check_result(result, "document binary")

  # Copy generated wrappers to R/
  src_wrappers <- usethis::proj_path("src", "rust", "miniextendr_wrappers.R")
  r_wrappers <- usethis::proj_path("R", "miniextendr_wrappers.R")

  if (fs::file_exists(src_wrappers)) {
    fs::file_copy(src_wrappers, r_wrappers, overwrite = TRUE)
    cli::cli_alert_success("Generated {.path R/miniextendr_wrappers.R}")
  }

  invisible(TRUE)
}

#' Full miniextendr build workflow
#'
#' Runs the complete two-pass install: autoconf -> configure -> install
#' (compiles Rust) -> document (generates R wrappers) -> install again
#' (incorporates wrappers). The two installs are needed because R wrapper
#' generation requires the compiled Rust `document` binary.
#'
#' @param install Whether to run `R CMD INSTALL` steps. If `FALSE`, only
#'   runs autoconf + configure + document.
#' @param not_cran Logical. If `TRUE` (the default), sets `NOT_CRAN=true`
#'   for configure and install steps.
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_build <- function(install = TRUE, not_cran = TRUE) {
  cli::cli_h1("miniextendr build workflow")

  env_vars <- if (not_cran) c(NOT_CRAN = "true") else character()
  pkg_path <- usethis::proj_get()

  cli::cli_h2("Step 1: autoconf")
  miniextendr_autoconf()

  cli::cli_h2("Step 2: configure")
  withr::with_envvar(env_vars, miniextendr_configure())

  if (install) {
    cli::cli_h2("Step 3: first install (compile Rust)")
    if (!requireNamespace("devtools", quietly = TRUE)) {
      warn("devtools not installed, skipping install step")
    } else {
      withr::with_envvar(env_vars, {
        devtools::install(pkg_path, upgrade = "never", quiet = TRUE)
      })
      cli::cli_alert_success("Installed package (first pass)")
    }
  }

  cli::cli_h2("Step 4: document (generate R wrappers)")
  miniextendr_document()

  if (install) {
    cli::cli_h2("Step 5: second install (with R wrappers)")
    withr::with_envvar(env_vars, {
      devtools::install(pkg_path, upgrade = "never", quiet = TRUE)
    })
    cli::cli_alert_success("Installed package (second pass)")
  }

  cli::cli_alert_success("Build complete!")
  invisible(TRUE)
}
