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
  # Read template content
  template_content <- readLines(template_path("Rbuildignore"))

  rbuildignore_path <- usethis::proj_path(".Rbuildignore")

  if (fs::file_exists(rbuildignore_path)) {
    existing <- readLines(rbuildignore_path)
    # Add patterns that aren't already present
    new_patterns <- setdiff(template_content, existing)
    if (length(new_patterns) > 0) {
      cat(c("", new_patterns), file = rbuildignore_path, sep = "\n", append = TRUE)
      cli::cli_alert_success("Added {length(new_patterns)} patterns to .Rbuildignore")
    } else {
      cli::cli_alert_info(".Rbuildignore already has all miniextendr patterns")
    }
  } else {
    writeLines(template_content, rbuildignore_path)
    bullet_created(".Rbuildignore")
  }

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

  gitignore_path <- usethis::proj_path(".gitignore")

  if (fs::file_exists(gitignore_path)) {
    existing <- readLines(gitignore_path)
    # Add patterns that aren't already present
    new_patterns <- setdiff(template_content, existing)
    if (length(new_patterns) > 0) {
      cat(c("", "# miniextendr", new_patterns), file = gitignore_path, sep = "\n", append = TRUE)
      cli::cli_alert_success("Added {length(new_patterns)} patterns to .gitignore")
    } else {
      cli::cli_alert_info(".gitignore already has all miniextendr patterns")
    }
  } else {
    writeLines(template_content, gitignore_path)
    bullet_created(".gitignore")
  }

  invisible(TRUE)
}
