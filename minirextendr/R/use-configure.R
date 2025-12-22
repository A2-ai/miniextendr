# Configure-related scaffolding functions

#' Add configure.ac to package
#'
#' Creates the autoconf configure.ac template that handles Rust toolchain
#' discovery, feature flags, and generates build files.
#'
#' @return Invisibly returns TRUE if file was created
#' @export
use_miniextendr_configure <- function() {
  use_template("configure.ac", data = template_data())
  cli::cli_alert_info("Run {.code minirextendr::miniextendr_autoconf()} to generate configure script")
  invisible(TRUE)
}

#' Add bootstrap.R to package
#'
#' Creates bootstrap.R which runs during package build when
#' `Config/build/bootstrap: TRUE` is set in DESCRIPTION.
#'
#' @return Invisibly returns TRUE if file was created
#' @export
use_miniextendr_bootstrap <- function() {
  use_template("bootstrap.R")
  invisible(TRUE)
}

#' Add cleanup scripts to package
#'
#' Creates cleanup, cleanup.win, and cleanup.ucrt scripts that clean
#' build artifacts after installation (used on CRAN).
#'
#' @return Invisibly returns TRUE if files were created
#' @export
use_miniextendr_cleanup <- function() {
  use_template("cleanup")
  use_template("cleanup.win")
  use_template("cleanup.ucrt")
  invisible(TRUE)
}

#' Add Windows configure wrappers
#'
#' Creates configure.win and configure.ucrt that delegate to the main
#' POSIX configure script.
#'
#' @return Invisibly returns TRUE if files were created
#' @export
use_miniextendr_configure_win <- function() {
  use_template("configure.win")
  use_template("configure.ucrt")
  invisible(TRUE)
}

#' Copy config.guess and config.sub scripts
#'
#' Copies the GNU autoconf helper scripts config.guess and config.sub
#' from the system autoconf installation. These are required for
#' cross-compilation support.
#'
#' @return Invisibly returns TRUE if files were copied
#' @export
use_miniextendr_config_scripts <- function() {
  check_autoconf()

  # Find autoconf's data directory
  autoconf_dir <- tryCatch(
    {
      dir <- system2("autoconf", "--print-datadir", stdout = TRUE, stderr = TRUE)
      if (length(dir) == 0 || !dir.exists(dir)) {
        # Fallback: try common locations
        candidates <- c(
          "/usr/share/autoconf",
          "/usr/local/share/autoconf",
          "/opt/homebrew/share/autoconf"
        )
        dir <- candidates[dir.exists(candidates)][1]
      }
      dir
    },
    error = function(e) NULL
  )

  if (is.null(autoconf_dir) || is.na(autoconf_dir)) {
    # Use bundled copies as fallback
    cli::cli_alert_info("Using bundled config.guess and config.sub")
    fs::file_copy(
      script_path("config.guess"),
      usethis::proj_path("config.guess"),
      overwrite = TRUE
    )
    fs::file_copy(
      script_path("config.sub"),
      usethis::proj_path("config.sub"),
      overwrite = TRUE
    )
  } else {
    # Copy from autoconf
    for (script in c("config.guess", "config.sub")) {
      src <- file.path(autoconf_dir, script)
      if (file.exists(src)) {
        fs::file_copy(src, usethis::proj_path(script), overwrite = TRUE)
        bullet_created(script, "Copied")
      } else {
        # Try fallback to bundled
        fs::file_copy(
          script_path(script),
          usethis::proj_path(script),
          overwrite = TRUE
        )
        bullet_created(script, "Copied bundled")
      }
    }
  }

  invisible(TRUE)
}

#' Add Makevars.in template
#'
#' Creates src/Makevars.in which is processed by configure to generate
#' the actual Makevars used during package build.
#'
#' @return Invisibly returns TRUE if file was created
#' @export
use_miniextendr_makevars <- function() {
  ensure_dir(usethis::proj_path("src"))
  use_template("Makevars.in", save_as = "src/Makevars.in")
  invisible(TRUE)
}
