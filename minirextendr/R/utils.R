# Internal utility functions for minirextendr

#' Get path to package template
#'
#' @param name Name of the template file
#' @return Full path to the template
#' @noRd
template_path <- function(name) {
  system.file("templates", name, package = "minirextendr", mustWork = TRUE)
}

#' Get path to bundled script
#'
#' @param name Name of the script file
#' @return Full path to the script
#' @noRd
script_path <- function(name) {
  system.file("scripts", name, package = "minirextendr", mustWork = TRUE)
}

#' Use a minirextendr template
#'
#' Wrapper around usethis::use_template that defaults to minirextendr package.
#'
#' @param template Name of template file in inst/templates
#' @param save_as Path to save the file (relative to project root)
#' @param data Named list of template variables
#' @param open Whether to open the file after creation
#' @return Invisibly returns TRUE if file was created
#' @noRd
use_template <- function(template, save_as = template, data = list(), open = FALSE) {
  usethis::use_template(
    template = template,
    save_as = save_as,
    data = data,
    open = open,
    package = "minirextendr"
  )
}

#' Check if a system command is available
#'
#' @param cmd Command name to check
#' @param msg Optional custom error message
#' @return TRUE if available, otherwise aborts
#' @noRd
check_installed_cmd <- function(cmd, msg = NULL) {
  path <- Sys.which(cmd)
  if (path == "") {
    msg <- msg %||% glue::glue(
      "{cmd} is required but not found on PATH. ",
      "Please install {cmd} and ensure it's available."
    )
    abort(msg)
  }
  invisible(TRUE)
}

#' Check autoconf availability
#'
#' @return TRUE if autoconf is available
#' @noRd
check_autoconf <- function() {
  check_installed_cmd(
    "autoconf",
    c(
      "autoconf is required for miniextendr packages.",
      "i" = "Install via: brew install autoconf (macOS) or apt install autoconf (Ubuntu)"
    )
  )
}

#' Check cargo/rustc availability
#'
#' @return TRUE if Rust toolchain is available
#' @noRd
check_rust <- function() {
 check_installed_cmd(
    "cargo",
    c(
      "Rust toolchain is required for miniextendr packages.",
      "i" = "Install from https://rustup.rs"
    )
  )
  check_installed_cmd("rustc")
}

#' Get package name from current project
#'
#' @return Package name as string
#' @noRd
get_package_name <- function() {
  desc <- desc::desc(usethis::proj_get())
  desc$get_field("Package")
}

#' Convert R package name to Rust-safe identifier
#'
#' Replaces dots and hyphens with underscores.
#'
#' @param name R package name
#' @return Rust-safe name
#' @noRd
to_rust_name <- function(name) {
  gsub("[.-]", "_", name)
}

#' Standard template data for current project
#'
#' @return Named list with package, package_rs, year, etc.
#' @noRd
template_data <- function() {
 pkg <- get_package_name()
  list(
    package = pkg,
    package_rs = to_rust_name(pkg),
    Package = tools::toTitleCase(pkg),
    year = format(Sys.Date(), "%Y")
  )
}

#' Ensure directory exists
#'
#' @param path Path to directory
#' @return Invisibly returns path
#' @noRd
ensure_dir <- function(path) {
  if (!fs::dir_exists(path)) {
    fs::dir_create(path, recurse = TRUE)
    cli::cli_alert_success("Created {.path {path}}")
  }
  invisible(path)
}

#' Check if current project has miniextendr setup
#'
#' @return TRUE if project appears to be a miniextendr package
#' @noRd
is_miniextendr_package <- function() {
  configure_ac <- usethis::proj_path("configure.ac")
  if (!fs::file_exists(configure_ac)) {
    return(FALSE)
  }

  contents <- readLines(configure_ac, warn = FALSE)
  if (!any(grepl("MINIEXTENDR_FEATURES", contents, fixed = TRUE))) {
    return(FALSE)
  }

  templates <- c(
    "src/rust/Cargo.toml.in",
    "src/rust/document.rs.in",
    "src/entrypoint.c.in",
    "src/Makevars.in"
  )
  generated <- c(
    "src/rust/Cargo.toml",
    "src/rust/document.rs",
    "src/entrypoint.c",
    "src/Makevars"
  )

  has_templates <- all(fs::file_exists(usethis::proj_path(templates)))
  has_generated <- all(fs::file_exists(usethis::proj_path(generated)))

  has_templates || has_generated
}

#' CLI bullet for file creation
#'
#' @param path Path that was created
#' @param verb Action verb (default "Created")
#' @noRd
bullet_created <- function(path, verb = "Created") {
  cli::cli_alert_success("{verb} {.path {path}}")
}
