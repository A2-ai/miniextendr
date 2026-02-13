# Rust project scaffolding functions

#' Create Rust project structure
#'
#' Creates the src/rust/ directory with Cargo.toml, lib.rs, and build.rs.
#' This sets up a basic miniextendr Rust library with example functions.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE if files were created
#' @keywords internal
use_miniextendr_rust <- function(path = ".") {
  with_project(path)
  data <- template_data()

  # Create directories
  ensure_dir(usethis::proj_path("src", "rust"))
  ensure_dir(usethis::proj_path("vendor"))

  # Cargo.toml template (mustache-substituted at scaffolding time, not by autoconf)
  use_template("Cargo.toml.tmpl", save_as = "src/rust/Cargo.toml", data = data)

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
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE if file was created
#' @keywords internal
use_miniextendr_cargo_config <- function(path = ".") {
  with_project(path)
  ensure_dir(usethis::proj_path("src", "rust"))
  use_template("cargo-config.toml.in", save_as = "src/rust/cargo-config.toml.in")
  invisible(TRUE)
}

#' Add document.rs.in template
#'
#' Creates src/rust/document.rs.in which generates R wrapper functions
#' from the Rust code. This is processed by configure to create document.rs.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE if file was created
#' @keywords internal
use_miniextendr_document <- function(path = ".") {
  with_project(path)
  ensure_dir(usethis::proj_path("src", "rust"))
  use_template("document.rs.in", save_as = "src/rust/document.rs.in")
  generate_document_rs(
    usethis::proj_path("src", "rust", "document.rs.in"),
    usethis::proj_path("src", "rust", "document.rs"),
    package = get_package_name()
  )
  invisible(TRUE)
}

#' Add entrypoint.c.in template
#'
#' Creates src/entrypoint.c.in which provides the C entry point for R
#' to load the shared library. Initializes panic hooks and worker thread.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE if file was created
#' @keywords internal
use_miniextendr_entrypoint <- function(path = ".") {
  with_project(path)
  ensure_dir(usethis::proj_path("src"))
  use_template("entrypoint.c.in", save_as = "src/entrypoint.c.in")
  invisible(TRUE)
}

#' Add mx_abi.c.in and mx_abi.h templates
#'
#' Creates src/mx_abi.c.in and inst/include/mx_abi.h which provide the
#' C ABI for cross-package trait dispatch. These files enable other packages
#' to interact with miniextendr's trait system.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE if files were created
#' @keywords internal
use_miniextendr_mx_abi <- function(path = ".") {
  with_project(path)
  data <- template_data()

  # Create src/ directory if needed

  ensure_dir(usethis::proj_path("src"))

  # mx_abi.c.in template
  use_template("mx_abi.c.in", save_as = "src/mx_abi.c.in")


  # inst/include/ directory and header
  ensure_dir(usethis::proj_path("inst", "include"))
  use_template("mx_abi.h", save_as = "inst/include/mx_abi.h",
               subdir = "inst_include", data = data)

  invisible(TRUE)
}
