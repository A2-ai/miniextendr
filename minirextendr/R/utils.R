# Internal utility functions for minirextendr

#' Set active project for the duration of the calling function
#'
#' Calls `usethis::local_project()` so that all downstream
#' `usethis::proj_path()` / `usethis::proj_get()` calls resolve relative
#' to `path`. The project is restored when the calling function exits.
#'
#' When `path` is `"."` and a project is already active, the current project
#' is kept. This ensures nested function calls (e.g., `use_miniextendr()`
#' calling `use_miniextendr_description()`) inherit the parent's project
#' setting rather than resetting to the working directory.
#'
#' @param path Path to the R package root (default `"."`).
#' @param .local_envir Environment where the deferred restore should run
#'   (default: the caller's frame, i.e. the function that called
#'   `with_project()`).
#' @return Called for its side effect; returns `NULL` invisibly.
#' @noRd
with_project <- function(path, .local_envir = parent.frame()) {
  if (identical(path, ".")) {
    active <- tryCatch(usethis::proj_get(), error = function(e) NULL)
    if (!is.null(active)) {
      path <- active
    }
  }
  usethis::local_project(path, .local_envir = .local_envir, quiet = TRUE,
                         force = TRUE, setwd = FALSE)
  invisible()
}

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
#' - "monorepo": Has a Cargo.toml anywhere in the parent tree
#'   (indicates Rust project context where rpkg/ will be embedded)
#' - "rpkg": Is a standalone R package (has DESCRIPTION, no Cargo.toml in tree)
#'
#' Uses rprojroot for reliable tree-walking detection.
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
    # Check if this rpkg is embedded in a Rust project (Cargo.toml anywhere up the tree)
    rust_root <- find_rust_root(path)
    if (!is.null(rust_root)) {
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
#' Walks up the directory tree to find a Cargo.toml, indicating
#' the R package is embedded in a Rust project context.
#'
#' @param path Path to check
#' @return TRUE if inside a Rust project, FALSE otherwise
#' @noRd
is_in_rust_project <- function(path = usethis::proj_get()) {
  rust_root <- find_rust_root(path)
  !is.null(rust_root)
}

#' Find the root of a Rust project
#'
#' Walks up the directory tree to find a directory containing Cargo.toml.
#' Uses rprojroot for reliable detection.
#'
#' @param path Path to start searching from
#' @return Path to Rust project root, or NULL if not found
#' @noRd
find_rust_root <- function(path = usethis::proj_get()) {
  tryCatch(
    {
      rprojroot::find_root(rprojroot::has_file("Cargo.toml"), path = path)
    },
    error = function(e) {
      NULL
    }
  )
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
#' Uses usethis' templating machinery to render and write a template from
#' the current template type directory.
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
  template_rel <- if (is.null(subdir)) {
    file.path(get_template_type(), template)
  } else {
    file.path(get_template_type(), subdir, template)
  }

  target_path <- usethis::proj_path(save_as)
  ensure_dir(dirname(target_path))

  new <- usethis::use_template(
    template = template_rel,
    save_as = save_as,
    data = data,
    open = open && rlang::is_interactive(),
    package = "minirextendr"
  )

  invisible(new)
}

#' Copy a template with simple string replacements
#'
#' Unlike `use_template()`, this does NOT use mustache rendering.
#' It performs literal `{{{key}}}` replacements only, preserving
#' `{{just_variables}}` and other double-brace syntax.
#'
#' @param template Template file name
#' @param save_as Target file path (relative to project root)
#' @param data Named list of replacements (keys are matched as `{{{key}}}`)
#' @param subdir Optional subdirectory within template type
#' @noRd
copy_template <- function(template, save_as = template, data = list(),
                          subdir = NULL) {
  src <- template_path(template, subdir = subdir)
  content <- readLines(src, warn = FALSE)
  text <- paste(content, collapse = "\n")

  for (key in names(data)) {
    text <- gsub(paste0("{{{", key, "}}}"), data[[key]], text, fixed = TRUE)
  }

  target_path <- usethis::proj_path(save_as)
  ensure_dir(dirname(target_path))
  writeLines(text, target_path)
  bullet_created(save_as)

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

#' Convert R package name to tarname
#'
#' Matches autoconf's PACKAGE_TARNAME derivation (lowercase, dots to hyphens).
#'
#' @param name R package name
#' @return Tarname (lowercase, dots replaced with hyphens)
#' @noRd
to_tarname <- function(name) {
  tolower(gsub("\\.", "-", name))
}

#' Generate document.rs from document.rs.in
#'
#' Performs the three substitutions that configure normally does, so that
#' `cargo check` works without running `./configure` first.
#'
#' @param document_rs_in_path Path to document.rs.in
#' @param document_rs_path Path to write document.rs
#' @param package R package name
#' @noRd
generate_document_rs <- function(document_rs_in_path, document_rs_path, package) {
  crate_name <- to_rust_name(package)
  crate_upper <- toupper(crate_name)
  tarname <- to_tarname(package)

  content <- readLines(document_rs_in_path, warn = FALSE)
  content <- gsub("__CARGO_STATICLIB_NAME_PLACEHOLDER__", crate_name, content, fixed = TRUE)
  content <- gsub("@PACKAGE_TARNAME_RS_UPPERCASE@", crate_upper, content, fixed = TRUE)
  content <- gsub("@PACKAGE_TARNAME@", tarname, content, fixed = TRUE)
  writeLines(content, document_rs_path)
}

#' Generate entrypoint.c from entrypoint.c.in
#'
#' Performs the substitutions that configure normally does, so that
#' the generated C file works without running `./configure` first.
#'
#' @param in_path Path to entrypoint.c.in
#' @param out_path Path to write entrypoint.c
#' @param package R package name
#' @noRd
generate_entrypoint_c <- function(in_path, out_path, package) {
  content <- readLines(in_path, warn = FALSE)
  content <- gsub("@PACKAGE_NAME@", package, content, fixed = TRUE)
  content <- gsub("@PACKAGE_TARNAME_RS@", to_rust_name(package), content, fixed = TRUE)
  writeLines(content, out_path)
}

#' Generate mx_abi.c from mx_abi.c.in
#'
#' Performs the substitutions that configure normally does, so that
#' the generated C file works without running `./configure` first.
#'
#' @param in_path Path to mx_abi.c.in
#' @param out_path Path to write mx_abi.c
#' @param package R package name
#' @noRd
generate_mx_abi_c <- function(in_path, out_path, package) {
  content <- readLines(in_path, warn = FALSE)
  content <- gsub("@PACKAGE_NAME@", package, content, fixed = TRUE)
  writeLines(content, out_path)
}

#' Get package name from Cargo.toml
#'
#' @param cargo_path Path to Cargo.toml file
#' @return Package name from Cargo.toml, with hyphens replaced by dots for R
#' @noRd
get_package_name_from_cargo <- function(cargo_path = file.path(usethis::proj_get(), "Cargo.toml")) {
  if (!file.exists(cargo_path)) {
    abort("Cargo.toml not found")
  }

  lines <- readLines(cargo_path, warn = FALSE)

  # Look for: name = "package-name"
  name_line <- grep('^name\\s*=\\s*"', lines, value = TRUE)[1]
  if (is.na(name_line)) {
    abort("Could not find package name in Cargo.toml")
  }

  # Extract name from: name = "my-crate"
  name <- sub('^name\\s*=\\s*"([^"]+)".*$', '\\1', name_line)

  # Convert Rust naming (hyphens) to R naming (dots)
  gsub("-", ".", name)
}

#' Standard template data for current project
#'
#' @param crate_name Optional crate name for monorepo template
#' @param package Optional package name override (for when DESCRIPTION doesn't exist yet)
#' @param rpkg_name Optional R package subdirectory name for monorepo template
#' @return Named list with package, package_rs, crate_name, year, etc.
#' @noRd
template_data <- function(crate_name = NULL, package = NULL, rpkg_name = NULL) {
  # Get package name: use provided, or read from DESCRIPTION
  if (is.null(package)) {
    pkg <- get_package_name()
  } else {
    pkg <- package
  }

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

  if (!is.null(rpkg_name)) {
    data$rpkg_name <- rpkg_name
  } else if (get_template_type() == "monorepo") {
    # Default rpkg directory name
    data$rpkg_name <- "rpkg"
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
    "src/rust/Cargo.toml",
    "src/rust/document.rs.in",
    "src/entrypoint.c.in",
    "src/Makevars.in"
  )
  generated <- c(
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
