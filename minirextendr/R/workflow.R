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

  result <- withr::with_dir(usethis::proj_get(), {
    system2("autoconf", c("-v", "-i", "-f"), stdout = TRUE, stderr = TRUE)
  })

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    abort(c(
      "autoconf failed",
      "i" = "Output: {paste(result, collapse = '\n')}"
    ))
  }

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

  cli::cli_alert("Running ./configure...")

  result <- withr::with_dir(usethis::proj_get(), {
    system2("./configure", stdout = TRUE, stderr = TRUE)
  })

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    abort(c(
      "./configure failed",
      "i" = "Check config.log for details"
    ))
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

  result <- withr::with_dir(usethis::proj_path("src", "rust"), {
    system2("cargo", c("run", "--bin", "document", "--release"),
            stdout = TRUE, stderr = TRUE)
  })

  status <- attr(result, "status")
  if (!is.null(status) && status != 0) {
    abort(c(
      "document binary failed",
      "i" = "Output: {paste(utils::tail(result, 20), collapse = '\n')}"
    ))
  }

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
#' Runs the complete workflow: autoconf -> configure -> build -> document.
#' This is equivalent to running the following in sequence:
#' 1. `miniextendr_autoconf()`
#' 2. `miniextendr_configure()`
#' 3. `devtools::install()` (optional)
#' 4. `miniextendr_document()`
#'
#' @param install Whether to run `devtools::install()` after configure
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_build <- function(install = TRUE) {
  cli::cli_h1("miniextendr build workflow")

  cli::cli_h2("Step 1: autoconf")
  miniextendr_autoconf()

  cli::cli_h2("Step 2: configure")
  miniextendr_configure()

  if (install) {
    cli::cli_h2("Step 3: install")
    if (!requireNamespace("devtools", quietly = TRUE)) {
      warn("devtools not installed, skipping install step")
    } else {
      devtools::install(usethis::proj_get(), upgrade = "never", quiet = TRUE)
      cli::cli_alert_success("Installed package")
    }
  }

  cli::cli_h2("Step 4: document")
  miniextendr_document()

  cli::cli_alert_success("Build complete!")
  invisible(TRUE)
}
