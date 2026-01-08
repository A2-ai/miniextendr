# R package setup functions

#' Add package documentation file
#'
#' Creates R/<package>-package.R with the package docstring and
#' useDynLib directive for loading the shared library.
#'
#' @return Invisibly returns TRUE if file was created
#' @export
use_miniextendr_package_doc <- function() {
  data <- template_data()
  ensure_dir(usethis::proj_path("R"))

  save_as <- paste0("R/", data$package, "-package.R")
  use_template("package.R", save_as = save_as, data = data)

  cli::cli_alert_info("Run {.code devtools::document()} to update NAMESPACE with useDynLib")
  invisible(TRUE)
}

#' Update DESCRIPTION with miniextendr fields
#'
#' Adds the required Config/* fields to DESCRIPTION for miniextendr:
#' - Config/build/bootstrap: TRUE
#' - Config/build/never-clean: true
#' - Config/build/extra-sources: src/rust/Cargo.lock
#'
#' Also adds SystemRequirements for Rust.
#'
#' @return Invisibly returns TRUE
#' @export
use_miniextendr_description <- function() {
  desc_path <- usethis::proj_path("DESCRIPTION")

  if (!fs::file_exists(desc_path)) {
    abort("DESCRIPTION file not found. Is this an R package?")
  }

  d <- desc::desc(desc_path)

  # Set Config fields
  d$set("Config/build/bootstrap" = "TRUE")
  d$set("Config/build/never-clean" = "true")
  d$set("Config/build/extra-sources" = "src/rust/Cargo.lock")

  # Add SystemRequirements if not present
  sys_req <- d$get_field("SystemRequirements", default = "")
  if (!grepl("Rust", sys_req, ignore.case = TRUE)) {
    if (nzchar(sys_req)) {
      sys_req <- paste0(sys_req, ", Rust (>= 1.85)")
    } else {
      sys_req <- "Rust (>= 1.85)"
    }
    d$set("SystemRequirements" = sys_req)
  }

  d$write()
  cli::cli_alert_success("Updated DESCRIPTION with miniextendr config fields")
  invisible(TRUE)
}

#' Add .Rbuildignore patterns for miniextendr
#'
#' Appends miniextendr-specific ignore patterns to .Rbuildignore,
#' or creates the file if it doesn't exist.
#'
#' @return Invisibly returns TRUE
#' @export
use_miniextendr_rbuildignore <- function() {
  # Read template content (already regex patterns, skip escaping)
  template_content <- readLines(template_path("Rbuildignore"))

  # Filter out empty lines and comments for usethis
  patterns <- template_content[nzchar(template_content) & !grepl("^#", template_content)]

  # usethis handles deduplication and file creation automatically
  # escape = FALSE because our template already contains regex patterns
  usethis::use_build_ignore(patterns, escape = FALSE)

  invisible(TRUE)
}

#' Add .gitignore patterns for miniextendr
#'
#' Appends miniextendr-specific ignore patterns to .gitignore,
#' or creates the file if it doesn't exist.
#'
#' @return Invisibly returns TRUE
#' @export
use_miniextendr_gitignore <- function() {
  # Read template content
  template_content <- readLines(template_path("gitignore"))

  # Filter out empty lines and comments for usethis
  patterns <- template_content[nzchar(template_content) & !grepl("^#", template_content)]

  # usethis handles deduplication and file creation automatically
  usethis::use_git_ignore(patterns, directory = ".")

  invisible(TRUE)
}
