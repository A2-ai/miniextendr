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
  use_miniextendr_build_rs()

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

#' Add stub.c file
#'
#' Copies src/stub.c which is a minimal C file so R's build system
#' produces a shared library. All entry points are defined in Rust
#' via miniextendr_init!().
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE if file was created
#' @export
#' @keywords internal
use_miniextendr_stub <- function(path = ".") {
  with_project(path)
  ensure_dir(usethis::proj_path("src"))
  stub_src <- template_path("stub.c")
  stub_dest <- usethis::proj_path("src", "stub.c")
  fs::file_copy(stub_src, stub_dest, overwrite = TRUE)
  bullet_created("src/stub.c")
  invisible(TRUE)
}

#' Add mx_abi.h header template
#'
#' Creates inst/include/mx_abi.h which provides the C header for
#' cross-package trait dispatch. This enables other packages to
#' interact with miniextendr's trait system.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE if file was created
#' @keywords internal
use_miniextendr_mx_abi <- function(path = ".") {
  with_project(path)
  data <- template_data()

  # inst/include/ directory and header
  ensure_dir(usethis::proj_path("inst", "include"))
  use_template("mx_abi.h", save_as = "inst/include/mx_abi.h",
               subdir = "inst_include", data = data)

  invisible(TRUE)
}

#' Add build.rs template
#'
#' Creates src/rust/build.rs which runs miniextendr-lint during cargo build
#' to check `#[miniextendr]` attributes and registration consistency.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns TRUE if file was created
#' @keywords internal
#' @export
use_miniextendr_build_rs <- function(path = ".") {
  with_project(path)
  ensure_dir(usethis::proj_path("src", "rust"))
  use_template("build.rs", save_as = "src/rust/build.rs")
  invisible(TRUE)
}
