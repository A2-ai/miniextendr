# Rust project scaffolding functions

#' Create Rust project structure
#'
#' Creates the src/rust/ directory with Cargo.toml.in, lib.rs, and build.rs.
#' This sets up a basic miniextendr Rust library with example functions.
#'
#' @return Invisibly returns TRUE if files were created
#' @export
use_miniextendr_rust <- function() {
  data <- template_data()

  # Create directories
  ensure_dir(usethis::proj_path("src", "rust"))
  ensure_dir(usethis::proj_path("src", "vendor"))

  # Cargo.toml.in template
  use_template("Cargo.toml.in", save_as = "src/rust/Cargo.toml.in", data = data)

  # build.rs
  use_template("build.rs", save_as = "src/rust/build.rs")

  # lib.rs starter template
  use_template("lib.rs", save_as = "src/rust/lib.rs", data = data)

  cli::cli_alert_success("Created Rust project in {.path src/rust/}")
  cli::cli_alert_info("Edit {.path src/rust/lib.rs} to add your Rust functions")

 invisible(TRUE)
}

#' Add cargo-config.toml.in template
#'
#' Creates src/rust/cargo-config.toml.in which is processed by configure
#' to set up Cargo's target directory and source replacement for CRAN.
#'
#' @return Invisibly returns TRUE if file was created
#' @export
use_miniextendr_cargo_config <- function() {
  ensure_dir(usethis::proj_path("src", "rust"))
  use_template("cargo-config.toml.in", save_as = "src/rust/cargo-config.toml.in")
  invisible(TRUE)
}

#' Add document.rs.in template
#'
#' Creates src/rust/document.rs.in which generates R wrapper functions
#' from the Rust code. This is processed by configure to create document.rs.
#'
#' @return Invisibly returns TRUE if file was created
#' @export
use_miniextendr_document <- function() {
  ensure_dir(usethis::proj_path("src", "rust"))
  use_template("document.rs.in", save_as = "src/rust/document.rs.in")
  invisible(TRUE)
}

#' Add entrypoint.c.in template
#'
#' Creates src/entrypoint.c.in which provides the C entry point for R
#' to load the shared library. Initializes panic hooks and worker thread.
#'
#' @return Invisibly returns TRUE if file was created
#' @export
use_miniextendr_entrypoint <- function() {
  ensure_dir(usethis::proj_path("src"))
  use_template("entrypoint.c.in", save_as = "src/entrypoint.c.in")
  invisible(TRUE)
}
