# Internal utility functions for minirextendr

# Template type for current session (used by template functions)
.template_type <- new.env(parent = emptyenv())
.template_type$current <- "rpkg"

#' Set template type for scaffolding
#'
#' @param type Either "rpkg" (standalone R package) or "monorepo" (Rust workspace)
#' @noRd
set_template_type <- function(type = c("rpkg", "monorepo")) {
  type <- match.arg(type)
  .template_type$current <- type
  invisible(type)
}

#' Get current template type
#'
#' @return Current template type
#' @noRd
get_template_type <- function() {
  .template_type$current
}

#' Detect project type from directory structure
#'
#' Auto-detects whether the current project is:
#' - "monorepo": Has a Cargo.toml in the current directory or parent
#'   (indicates Rust project context where rpkg/ will be embedded)
#' - "rpkg": Is a standalone R package (has DESCRIPTION, no parent Cargo.toml)
#'
#' @param path Path to check (default: current project)
#' @return "monorepo" or "rpkg", or NULL if can't detect
#' @noRd
detect_project_type <- function(path = usethis::proj_get()) {
  # Check if we're in a Rust project (has Cargo.toml in current dir)
  cargo_toml <- file.path(path, "Cargo.toml")
  if (file.exists(cargo_toml)) {
    # Current directory is a Rust crate/workspace - this is a monorepo
    return("monorepo")
  }

  # Check if we're in an R package directory
  if (file.exists(file.path(path, "DESCRIPTION"))) {
    # Check if this rpkg is embedded in a Rust project (parent has Cargo.toml)
    parent_cargo <- file.path(dirname(path), "Cargo.toml")
    if (file.exists(parent_cargo)) {
      # This is an rpkg inside a Rust project (monorepo)
      return("monorepo")
    }
    # Standalone R package
    return("rpkg")
  }

  NULL
}

#' Check if project is inside a Rust project
#'
#' Looks for a Cargo.toml in the current directory or parent.
#' This indicates the R package is embedded in a Rust project context.
#'
#' @param path Path to check
#' @return TRUE if inside a Rust project, FALSE otherwise
#' @noRd
is_in_rust_project <- function(path = usethis::proj_get()) {
  # Check current directory
  cargo_toml <- file.path(path, "Cargo.toml")
  if (file.exists(cargo_toml)) {
    return(TRUE)
  }

  # Check parent (for rpkg/ inside monorepo)
  parent_cargo <- file.path(dirname(path), "Cargo.toml")
  if (file.exists(parent_cargo)) {
    return(TRUE)
  }

  FALSE
}

#' Check if project is inside a Cargo workspace
#'
#' Looks for a Cargo.toml with [workspace] in the current directory or parent.
#' A workspace allows multiple crates to share dependencies.
#'
#' @param path Path to check
#' @return TRUE if inside a workspace, FALSE otherwise
#' @noRd
is_in_rust_workspace <- function(path = usethis::proj_get()) {
  # Check current directory
  cargo_toml <- file.path(path, "Cargo.toml")
  if (file.exists(cargo_toml)) {
    cargo_content <- readLines(cargo_toml, warn = FALSE)
    if (any(grepl("^\\[workspace\\]", cargo_content))) {
      return(TRUE)
    }
  }

  # Check parent (for rpkg/ inside monorepo)
  parent_cargo <- file.path(dirname(path), "Cargo.toml")
  if (file.exists(parent_cargo)) {
    parent_content <- readLines(parent_cargo, warn = FALSE)
    if (any(grepl("^\\[workspace\\]", parent_content))) {
      return(TRUE)
    }
  }

  FALSE
}

#' Get path to package template
#'
#' For "rpkg" templates, returns templates from `templates/rpkg/`.
#' For "monorepo" templates, returns templates from `templates/monorepo/`.
#'
#' @param name Name of the template file (relative to template type directory)
#' @param subdir Optional subdirectory within the template type
#' @return Full path to the template
#' @noRd
template_path <- function(name, subdir = NULL) {
  type <- get_template_type()
  if (!is.null(subdir)) {
    path <- file.path("templates", type, subdir, name)
  } else {
    path <- file.path("templates", type, name)
  }
  system.file(path, package = "minirextendr", mustWork = TRUE)
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
#' Reads a template from the current template type directory, performs
#' mustache-style variable substitution, and writes to the target location.
#'
#' @param template Name of template file (relative to template type directory)
#' @param save_as Path to save the file (relative to project root)
#' @param data Named list of template variables for {{variable}} substitution
#' @param subdir Optional subdirectory within the template type (e.g., "rpkg" for monorepo)
#' @param open Whether to open the file after creation
#' @return Invisibly returns TRUE if file was created
#' @noRd
use_template <- function(template, save_as = template, data = list(),
                         subdir = NULL, open = FALSE) {
  # Get full path to template
  src_path <- template_path(template, subdir = subdir)

  # Read template content
  content <- readLines(src_path, warn = FALSE)
  content <- paste(content, collapse = "\n")

  # Perform mustache-style substitution for {{variable}}
  for (nm in names(data)) {
    pattern <- paste0("\\{\\{", nm, "\\}\\}")
    content <- gsub(pattern, data[[nm]], content)
  }

  # Ensure target directory exists
  target_path <- usethis::proj_path(save_as)
  ensure_dir(dirname(target_path))

  # Write file
  writeLines(content, target_path)
  bullet_created(save_as)

  # Open if requested
  if (open && rlang::is_interactive()) {
    usethis::edit_file(target_path)
  }

  invisible(TRUE)
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
#' @param crate_name Optional crate name for monorepo template
#' @return Named list with package, package_rs, crate_name, year, etc.
#' @noRd
template_data <- function(crate_name = NULL) {
  pkg <- get_package_name()
  data <- list(
    package = pkg,
    package_rs = to_rust_name(pkg),
    Package = tools::toTitleCase(pkg),
    year = format(Sys.Date(), "%Y")
  )

  # Add monorepo-specific data
  if (!is.null(crate_name)) {
    data$crate_name <- crate_name
  } else if (get_template_type() == "monorepo") {
    # Default crate name is package name with dashes
    data$crate_name <- gsub("\\.", "-", pkg)
  }

  data
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
