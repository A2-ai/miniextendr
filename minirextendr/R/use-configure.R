# Configure-related scaffolding functions

#' Add configure.ac to package
#'
#' Creates the autoconf configure.ac template that handles Rust toolchain
#' discovery, feature flags, and generates build files.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE if file was created
#' @keywords internal
use_miniextendr_configure <- function(path = ".") {
  with_project(path)
  use_template("configure.ac", data = template_data())
  cli::cli_alert_info("Run {.code minirextendr::miniextendr_autoconf()} to generate configure script")
  invisible(TRUE)
}

#' Add bootstrap.R to package
#'
#' Creates bootstrap.R which runs during package build when
#' `Config/build/bootstrap: TRUE` is set in DESCRIPTION.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE if file was created
#' @keywords internal
use_miniextendr_bootstrap <- function(path = ".") {
  with_project(path)
  use_template("bootstrap.R")
  invisible(TRUE)
}

#' Add cleanup scripts to package
#'
#' Creates cleanup, cleanup.win, and cleanup.ucrt scripts that clean
#' build artifacts after installation (used on CRAN).
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE if files were created
#' @keywords internal
use_miniextendr_cleanup <- function(path = ".") {
  with_project(path)
  use_template("cleanup")
  use_template("cleanup.win")
  use_template("cleanup.ucrt")

  # Ensure cleanup scripts are executable (R CMD build warns otherwise)
  for (script in c("cleanup", "cleanup.win", "cleanup.ucrt")) {
    script_path <- usethis::proj_path(script)
    if (fs::file_exists(script_path)) {
      fs::file_chmod(script_path, "755")
    }
  }

  invisible(TRUE)
}

#' Add Windows configure wrappers
#'
#' Creates configure.win and configure.ucrt that delegate to the main
#' POSIX configure script.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE if files were created
#' @keywords internal
use_miniextendr_configure_win <- function(path = ".") {
  with_project(path)
  use_template("configure.win")
  use_template("configure.ucrt")
  invisible(TRUE)
}

#' Copy config.guess and config.sub scripts
#'
#' Copies the bundled GNU autoconf helper scripts config.guess and config.sub
#' to the tools/ directory. These are required for cross-compilation support
#' and are referenced by AC_CONFIG_AUX_DIR([tools]) in configure.ac.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE if files were copied
#' @keywords internal
use_miniextendr_config_scripts <- function(path = ".") {
  with_project(path)
  # These go in tools/ directory per AC_CONFIG_AUX_DIR([tools]) in configure.ac
  ensure_dir(usethis::proj_path("tools"))
  for (script in c("config.guess", "config.sub")) {
    fs::file_copy(
      script_path(script),
      usethis::proj_path("tools", script),
      overwrite = TRUE
    )
    bullet_created(file.path("tools", script), "Copied")
  }

  invisible(TRUE)
}

#' Add Makevars.in template
#'
#' Creates src/Makevars.in which is processed by configure to generate
#' the actual Makevars used during package build.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE if file was created
#' @keywords internal
use_miniextendr_makevars <- function(path = ".") {
  with_project(path)
  ensure_dir(usethis::proj_path("src"))
  use_template("Makevars.in", save_as = "src/Makevars.in")
  use_template("win.def.in", save_as = "src/win.def.in")

  # cdylib-exports.def: Windows DLL symbol export for wrapper generation
  cdylib_src <- template_path("cdylib-exports.def")
  cdylib_dest <- usethis::proj_path("src", "cdylib-exports.def")
  fs::file_copy(cdylib_src, cdylib_dest, overwrite = TRUE)
  bullet_created("src/cdylib-exports.def")

  invisible(TRUE)
}
