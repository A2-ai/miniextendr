# Package creation functions

#' Create a new miniextendr package
#'
#' Creates a new R package with full miniextendr scaffolding, ready for
#' Rust development. This combines `usethis::create_package()` with
#' `use_miniextendr()`.
#'
#' @param path Path where to create the package
#' @param open Whether to open the new project in RStudio
#' @param rstudio Whether to create an RStudio project file
#' @return Path to the created package (invisibly)
#' @export
create_miniextendr_package <- function(path, open = rlang::is_interactive(),
                                        rstudio = TRUE) {
  # Create basic package
  usethis::create_package(
    path,
    open = FALSE,
    rstudio = rstudio,
    check_name = TRUE
  )

  # Set project to the new package
  usethis::proj_set(path)

  # Add miniextendr scaffolding
  use_miniextendr()

  # Open if requested
  if (open) {
    usethis::proj_activate(path)
  }

  invisible(path)
}

#' Add miniextendr to an existing package
#'
#' Sets up all miniextendr scaffolding in the current R package. This is
#' an all-in-one function that calls all the individual `use_miniextendr_*()`
#' functions.
#'
#' @param miniextendr_version Version of miniextendr to vendor (default: "latest")
#' @return Invisibly returns TRUE
#' @export
use_miniextendr <- function(miniextendr_version = "latest") {
  cli::cli_h1("Setting up miniextendr")

  # Check prerequisites
  check_rust()

  # Update DESCRIPTION first
  cli::cli_h2("Updating DESCRIPTION")
  use_miniextendr_description()

  # Build system
  cli::cli_h2("Adding build system")
  use_miniextendr_configure()
  use_miniextendr_bootstrap()
  use_miniextendr_cleanup()
  use_miniextendr_configure_win()
  use_miniextendr_config_scripts()
  use_miniextendr_makevars()

  # Rust project
  cli::cli_h2("Creating Rust project")
  use_miniextendr_rust()
  use_miniextendr_cargo_config()
  use_miniextendr_document()
  use_miniextendr_entrypoint()

  # R package files
  cli::cli_h2("Setting up R package")
  use_miniextendr_package_doc()
  use_miniextendr_rbuildignore()
  use_miniextendr_gitignore()

  # Vendor miniextendr crates
  cli::cli_h2("Vendoring miniextendr crates")
  use_miniextendr_vendor(version = miniextendr_version)

  # Summary
  cli::cli_h1("Setup complete!")
  cli::cli_alert_info("Next steps:")
  cli::cli_bullets(c(
    " " = "1. Edit {.path src/rust/lib.rs} to add your Rust functions",
    " " = "2. Run {.code minirextendr::miniextendr_autoconf()} to generate configure",
    " " = "3. Run {.code devtools::document()} to update NAMESPACE",
    " " = "4. Run {.code devtools::install()} to build and install"
  ))

  invisible(TRUE)
}
