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
  # NULL is documented as "use the active project" on several exported
  # signatures; usethis::local_project(NULL) would instead UNSET the active
  # project, so treat NULL exactly like "." (audit 2026-07-06 #7).
  if (is.null(path) || identical(path, ".")) {
    path <- "."
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

#' Find the root of a Rust project
#'
#' Walks up the directory tree to find a directory containing Cargo.toml.
#' Uses rprojroot for reliable detection.
#'
#' @param path Path to start searching from
#' Walk up directories to find a file
#'
#' Starting from `path`, walks up parent directories looking for `filename`.
#' Returns the directory containing the file, or NULL if not found.
#'
#' @param filename File to search for
#' @param path Starting directory
#' @return Path to the directory containing `filename`, or NULL
#' @noRd
find_root_with_file <- function(filename, path) {
  path <- normalizePath(path, mustWork = FALSE)
  for (i in seq_len(100)) {
    if (file.exists(file.path(path, filename))) return(path)
    parent <- dirname(path)
    if (parent == path) return(NULL)
    path <- parent
  }
  NULL
}

#' @return Path to Rust project root, or NULL if not found
#' @noRd
find_rust_root <- function(path = usethis::proj_get()) {
  find_root_with_file("Cargo.toml", path)
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

# region: shared scaffold content --------------------------------------------
# Byte-identical fragments needed by more than one of the three scaffold
# paths (standalone `use_miniextendr()`, monorepo `create_rpkg_subdirectory()`,
# inline `scaffold_inline_package()`). Centralized here so a change to one
# path can't silently diverge from the others. See audit
# `2026-07-03-dogfooding-minirextendr-r.md` finding #4.

#' `Config/build/*` DESCRIPTION fields required by miniextendr rpkgs
#'
#' Applied via `mx_desc_set()` in the standalone path
#' (`use_miniextendr_description()`) and baked into the hand-written
#' DESCRIPTION literal in the monorepo path (`create_rpkg_subdirectory()`,
#' which never has an existing DESCRIPTION to update). Not used by the inline
#' `rust_source()` path, which only needs `Config/build/bootstrap`.
#'
#' @noRd
MX_CONFIG_BUILD_FIELDS <- c(
  "Config/build/bootstrap" = "TRUE",
  "Config/build/never-clean" = "true",
  "Config/build/extra-sources" = "src/rust/Cargo.lock"
)

#' Minimal roxygen2-managed NAMESPACE content
#'
#' Carries the roxygen2 header so a later `devtools::document()` treats the
#' file as roxygen-managed and overwrites it cleanly, plus the single
#' `useDynLib()` directive so the freshly built shared library loads before
#' the first `document()` pass. Shared by `use_miniextendr_namespace()`
#' (standalone) and `create_rpkg_subdirectory()` (monorepo).
#'
#' @param pkg_name Package name to substitute into `useDynLib()`.
#' @return Character string (NAMESPACE file content).
#' @noRd
mx_minimal_namespace <- function(pkg_name) {
  sprintf(
    "# Generated by roxygen2: do not edit by hand\n\nuseDynLib(%s, .registration = TRUE)\n",
    pkg_name
  )
}

#' Minimal LICENSE file content for `License: MIT + file LICENSE`
#'
#' Shared by `use_miniextendr_description()` (standalone) and
#' `create_rpkg_subdirectory()` (monorepo).
#'
#' @param pkg_name Package name to substitute as the copyright holder.
#' @return Character string (LICENSE file content).
#' @noRd
mx_license_content <- function(pkg_name) {
  sprintf("YEAR: %s\nCOPYRIGHT HOLDER: %s authors\n",
          format(Sys.Date(), "%Y"), pkg_name)
}

#' Ignore-file patterns from an ignore template
#'
#' Reads an ignore-file template (`Rbuildignore` / `gitignore`) and filters it
#' down to the bare patterns: blank lines and `#` comment lines are dropped.
#' Shared prep step between the standalone path
#' (`use_miniextendr_rbuildignore()` / `use_miniextendr_gitignore()`, which
#' hand the patterns to `usethis::use_build_ignore()` /
#' `usethis::use_git_ignore()` for append + dedupe into a possibly
#' pre-existing file) and the monorepo path (`create_rpkg_subdirectory()`,
#' which writes them to a brand-new file). Empirically the two write paths
#' produce byte-identical files when the target does not yet exist (usethis
#' writes exactly the given lines, in order, newline-terminated), so a fresh
#' scaffold gets the same ignore-file content on both paths. See #1151.
#'
#' @param template Template file name (`"Rbuildignore"` or `"gitignore"`),
#'   resolved against the active template type via `template_path()`.
#' @param subdir Optional subdirectory within the template type (e.g. `"rpkg"`
#'   when scaffolding the R package subdirectory of a monorepo).
#' @return Character vector of ignore patterns.
#' @noRd
mx_ignore_patterns <- function(template, subdir = NULL) {
  lines <- readLines(template_path(template, subdir = subdir))
  lines[nzchar(lines) & !grepl("^#", lines)]
}

#' Copy the bundled config.guess / config.sub autoconf helper scripts
#'
#' All three scaffold paths need these under `tools/` for cross-compilation
#' support (`AC_CONFIG_AUX_DIR([tools])` in configure.ac): the standalone
#' path (`use_miniextendr_config_scripts()`), the monorepo path
#' (`create_rpkg_subdirectory()`), and the inline path
#' (`scaffold_inline_package()`).
#'
#' @param dest_dir Absolute path to the (already-created) destination directory.
#' @param display_prefix Path prefix used in the "Copied" bullet message, or
#'   `NULL` to suppress the bullet (used by the quiet inline path).
#' @return Invisibly returns TRUE.
#' @noRd
copy_config_scripts <- function(dest_dir, display_prefix = dest_dir) {
  for (script in c("config.guess", "config.sub")) {
    dest <- file.path(dest_dir, script)
    fs::file_copy(script_path(script), dest, overwrite = TRUE)
    fs::file_chmod(dest, "755")
    if (!is.null(display_prefix)) {
      bullet_created(file.path(display_prefix, script), "Copied")
    }
  }
  invisible(TRUE)
}

# endregion -------------------------------------------------------------------

#' Use a minirextendr template
#'
#' Uses usethis' templating machinery to render and write a template from
#' the current template type directory. Always overwrites existing files —
#' both initial scaffolding (in empty directories) and upgrade paths rely
#' on the template being the source of truth. usethis::use_template() skips
#' silently in non-interactive mode when the target exists; deleting first
#' makes the behavior predictable.
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

  if (fs::file_exists(target_path)) {
    fs::file_delete(target_path)
  }

  new <- usethis::use_template(
    template = template_rel,
    save_as = save_as,
    data = data,
    open = open && interactive(),
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
    msg <- msg %||% paste0(
      cmd, " is required but not found on PATH. ",
      "Please install ", cmd, " and ensure it's available."
    )
    cli::cli_abort(msg)
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
  mx_desc_get_field("Package", file = usethis::proj_path("DESCRIPTION"))
}

# =============================================================================
# DESCRIPTION file helpers (replaces desc package)
# =============================================================================

#' Read a single field from a DESCRIPTION file
#'
#' @param field Field name
#' @param file Path to DESCRIPTION
#' @param default Value if field missing
#' @return Field value as string
#' @noRd
mx_desc_get_field <- function(field, file, default = NA_character_) {
  dcf <- read.dcf(file, fields = field)
  val <- dcf[1, 1]
  if (is.na(val)) default else trimws(val)
}

#' Set fields in a DESCRIPTION file
#'
#' @param file Path to DESCRIPTION
#' @param ... Named values to set (e.g., Package = "foo")
#' @noRd
mx_desc_set <- function(file, ...) {
  fields <- list(...)
  lines <- readLines(file, warn = FALSE)

  for (nm in names(fields)) {
    val <- fields[[nm]]
    # Find existing field line
    pattern <- paste0("^", nm, ":")
    idx <- grep(pattern, lines)

    new_line <- paste0(nm, ": ", val)

    if (length(idx) > 0) {
      # Replace existing field (and any continuation lines)
      end <- idx[1]
      while (end < length(lines) && grepl("^\\s", lines[end + 1])) {
        end <- end + 1
      }
      lines <- c(lines[seq_len(idx[1] - 1)], new_line,
                  if (end < length(lines)) lines[(end + 1):length(lines)])
    } else {
      # Append new field before last empty line or at end
      lines <- c(lines, new_line)
    }
  }

  writeLines(lines, file)
}

#' Get dependencies from DESCRIPTION
#'
#' @param file Path to DESCRIPTION
#' @return Data frame with columns: type, package, version
#' @noRd
mx_desc_get_deps <- function(file) {
  dcf <- read.dcf(file)
  result <- data.frame(type = character(), package = character(),
                       version = character(), stringsAsFactors = FALSE)

  for (type in c("Depends", "Imports", "Suggests", "LinkingTo", "Enhances")) {
    val <- dcf[1, type]
    if (is.na(val)) next
    pkgs <- trimws(strsplit(val, ",")[[1]])
    pkgs <- pkgs[nzchar(pkgs)]
    for (pkg in pkgs) {
      # Parse "pkg (>= 1.0)" or just "pkg"
      m <- regmatches(pkg, regexec("^([^(]+)\\s*(?:\\((.+)\\))?$", pkg))[[1]]
      pkg_name <- trimws(m[2])
      pkg_ver <- if (length(m) >= 3 && !is.na(m[3])) trimws(m[3]) else "*"
      result <- rbind(result, data.frame(type = type, package = pkg_name,
                                         version = pkg_ver,
                                         stringsAsFactors = FALSE))
    }
  }
  result
}

#' Add or update a dependency in DESCRIPTION
#'
#' @param file Path to DESCRIPTION
#' @param pkg Package name
#' @param type Dependency type (e.g., "Imports")
#' @param version Version constraint (e.g., ">= 1.0") or NULL
#' @noRd
mx_desc_set_dep <- function(file, pkg, type = "Imports", version = NULL) {
  lines <- readLines(file, warn = FALSE)

  # Build the dependency string
  dep_str <- if (!is.null(version) && nzchar(version) && version != "*") {
    paste0(pkg, " (", version, ")")
  } else {
    pkg
  }

  # Find the section
  section_idx <- grep(paste0("^", type, ":"), lines)

  if (length(section_idx) == 0) {
    # Add new section
    lines <- c(lines, paste0(type, ":\n    ", dep_str))
    writeLines(lines, file)
    return(invisible())
  }

  # Find extent of section (continuation lines start with whitespace)
  start <- section_idx[1]
  end <- start
  while (end < length(lines) && grepl("^\\s", lines[end + 1])) {
    end <- end + 1
  }

  # Extract current deps
  section_text <- paste(lines[start:end], collapse = "\n")
  # Remove field name
  deps_text <- sub(paste0("^", type, ":\\s*"), "", section_text)
  deps <- trimws(strsplit(deps_text, ",")[[1]])
  deps <- deps[nzchar(deps)]

  # Remove existing entry for this package
  pkg_pattern <- paste0("^", pkg, "\\b")
  deps <- deps[!grepl(pkg_pattern, deps)]

  # Add new entry
  deps <- c(deps, dep_str)
  deps <- sort(deps)

  # Rebuild section
  new_section <- paste0(type, ":\n", paste0("    ", deps, collapse = ",\n"))
  lines <- c(
    if (start > 1) lines[1:(start - 1)],
    new_section,
    if (end < length(lines)) lines[(end + 1):length(lines)]
  )

  writeLines(lines, file)
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

#' Get package name from Cargo.toml
#'
#' @param cargo_path Path to Cargo.toml file
#' @return Package name from Cargo.toml, with hyphens replaced by dots for R
#' @noRd
get_package_name_from_cargo <- function(cargo_path = file.path(usethis::proj_get(), "Cargo.toml")) {
  if (!file.exists(cargo_path)) {
    cli::cli_abort(c(
      "No {.file Cargo.toml} found at {.path {cargo_path}}.",
      "i" = "The {.val monorepo} template reads the crate name from a Rust workspace {.file Cargo.toml} at the project root.",
      "i" = "For a standalone R package, pass {.code template_type = \"rpkg\"}."
    ))
  }

  lines <- readLines(cargo_path, warn = FALSE)

  # Look for: name = "package-name"
  name_line <- grep('^name\\s*=\\s*"', lines, value = TRUE)[1]
  if (is.na(name_line)) {
    cli::cli_abort("Could not find package name in Cargo.toml")
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

  pkg_rs <- to_rust_name(pkg)

  data <- list(
    package = pkg,
    package_rs = pkg_rs,
    Package = tools::toTitleCase(pkg),
    features_var = "CARGO_FEATURES",
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
  if (!any(grepl("CARGO_FEATURES", contents, fixed = TRUE))) {
    return(FALSE)
  }

  required <- c(
    "src/rust/Cargo.toml",
    "src/Makevars.in"
  )

  has_required <- all(fs::file_exists(usethis::proj_path(required)))
  # Also accept if stub.c or generated Makevars exists
  has_stub <- fs::file_exists(usethis::proj_path("src", "stub.c"))
  has_makevars <- fs::file_exists(usethis::proj_path("src", "Makevars"))

  has_required || (has_stub && has_makevars)
}

#' CLI bullet for file creation
#'
#' @param path Path that was created
#' @param verb Action verb (default "Created")
#' @noRd
bullet_created <- function(path, verb = "Created") {
  cli::cli_alert_success("{verb} {.path {path}}")
}
