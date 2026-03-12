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
create_miniextendr_package <- function(path, open = interactive(),
                                        rstudio = TRUE) {
  # Validate package name (derived from directory basename)
  # R package names: ASCII letters, digits, and dots only.
  # Must start with a letter and not end with a dot. Minimum 2 characters.
  pkg_name <- basename(normalizePath(path, mustWork = FALSE))
  if (!grepl("^[a-zA-Z][a-zA-Z0-9.]*[a-zA-Z0-9]$", pkg_name) || nchar(pkg_name) < 2) {
    cli::cli_abort(c(
      "Package name {.val {pkg_name}} is not a valid R package name.",
      "i" = "R package names must start with a letter, contain only ASCII letters, digits, and dots, and not end with a dot.",
      "i" = "Try: {.val {gsub('[^a-zA-Z0-9.]', '', pkg_name)}}"
    ))
  }

  # Create basic package
  usethis::create_package(
    path,
    open = FALSE,
    rstudio = rstudio,
    check_name = FALSE
  )

  # Set project to the new package
  usethis::proj_set(path)

  # Add miniextendr scaffolding
  use_miniextendr(template_type = "rpkg")

  # Open if requested
  if (open) {
    usethis::proj_activate(path)
  }

  invisible(path)
}

#' Create a new miniextendr monorepo
#'
#' Creates a new Rust workspace with an embedded R package. This is the
#' "monorepo" template where Rust is the primary project and the R package
#' lives inside it (similar to how miniextendr itself is organized).
#'
#' @param path Path where to create the monorepo
#' @param package R package name (default: derived from path)
#' @param crate_name Main Rust crate name (default: derived from package name)
#' @param rpkg_name Name of the R package subdirectory (default: same as package name)
#' @param local_path Optional path to local miniextendr repository. If provided,
#'   vendors from local path instead of downloading from GitHub.
#' @param miniextendr_version Version tag to download (default: "main" for latest).
#'   Passed to [vendor_miniextendr()]. Ignored when `local_path` is provided.
#' @param open Whether to open the new project in RStudio/IDE
#' @return Path to the created monorepo (invisibly)
#' @export
create_miniextendr_monorepo <- function(path, package = basename(path),
                                         crate_name = gsub("\\.", "-", package),
                                         rpkg_name = package,
                                         local_path = NULL,
                                         miniextendr_version = "main",
                                         open = interactive()) {
  cli::cli_h1("Creating miniextendr monorepo")

  # Validate rpkg_name != crate_name (they're both directories under the project root)
  if (identical(rpkg_name, crate_name)) {
    cli::cli_abort(c(
      "{.arg rpkg_name} and {.arg crate_name} must be different (both are {.val {crate_name}}).",
      "i" = "Set {.arg rpkg_name} explicitly, e.g. {.code rpkg_name = \"{crate_name}-rpkg\"}"
    ))
  }

  # Check prerequisites
  check_rust()

  # Create root directory and normalize path
  fs::dir_create(path)
  path <- normalizePath(path, mustWork = TRUE)

  # Set project without checking for package structure
  usethis::proj_set(path, force = TRUE)

  set_template_type("monorepo")
  on.exit(set_template_type("rpkg"), add = TRUE)

  pkg_rs <- to_rust_name(package)
  data <- list(
    package = package,
    package_rs = pkg_rs,
    Package = tools::toTitleCase(package),
    crate_name = crate_name,
    rpkg_name = rpkg_name,
    features_var = paste0(toupper(pkg_rs), "_FEATURES"),
    year = format(Sys.Date(), "%Y")
  )

  # Root workspace files
  cli::cli_h2("Creating workspace root")
  use_template("Cargo.toml.tmpl", save_as = "Cargo.toml", data = data)
  # justfile uses {{variable}} syntax (just's interpolation) which collides
  # with mustache. Use copy_template for literal {{{key}}} substitution only.
  copy_template("justfile", data = data)
  use_template("gitignore", save_as = ".gitignore", data = data)

  # Version management tool (referenced by justfile version-* recipes)
  ensure_dir(usethis::proj_path("tools"))
  copy_template("bump-version.R", save_as = file.path("tools", "bump-version.R"),
                subdir = "tools", data = data)

  # Create main Rust crate
  cli::cli_h2("Creating main Rust crate")
  ensure_dir(usethis::proj_path(crate_name, "src"))
  use_template("Cargo.toml.tmpl", save_as = file.path(crate_name, "Cargo.toml"),
               subdir = "my-crate", data = data)
  use_template("lib.rs", save_as = file.path(crate_name, "src", "lib.rs"),
               subdir = file.path("my-crate", "src"), data = data)

  # Create R package in rpkg/
  cli::cli_h2("Creating R package")
  create_rpkg_subdirectory(data, rpkg_name = rpkg_name)

  # Vendor miniextendr crates into rpkg/vendor/
  cli::cli_h2("Vendoring miniextendr crates")
  vendor_miniextendr(
    version = miniextendr_version,
    dest = usethis::proj_path(rpkg_name, "vendor"),
    local_path = local_path
  )

  # Generate configure script from configure.ac
  if (nzchar(Sys.which("autoconf"))) {
    cli::cli_h2("Generating configure script")
    tryCatch(
      miniextendr_autoconf(usethis::proj_path(rpkg_name)),
      error = function(e) {
        cli::cli_alert_warning("autoconf failed: {conditionMessage(e)}")
        cli::cli_alert_info("Run {.code miniextendr_autoconf()} manually later")
      }
    )
  }

  # Initialize git
  if (!fs::file_exists(usethis::proj_path(".git"))) {
    usethis::use_git()
  }

  cli::cli_h1("Monorepo created!")
  cli::cli_alert_info("Next steps:")
  cli::cli_bullets(c(
    " " = "1. Edit {.path {crate_name}/src/lib.rs} for your main Rust library",
    " " = "2. Edit {.path {rpkg_name}/src/rust/lib.rs} for R-exposed functions",
    " " = "3. Run {.code just configure} to set up build system",
    " " = "4. Run {.code just rcmdinstall} to build and install"
  ))

  if (open) {
    usethis::proj_activate(path)
  }

  invisible(path)
}

#' Create rpkg subdirectory for monorepo template
#'
#' @param data Template data list
#' @param rpkg_name Name of the R package subdirectory (default: "rpkg")
#' @noRd
create_rpkg_subdirectory <- function(data, rpkg_name = "rpkg") {
  # Create directory structure
  ensure_dir(usethis::proj_path(rpkg_name, "R"))
  ensure_dir(usethis::proj_path(rpkg_name, "src", "rust"))
  ensure_dir(usethis::proj_path(rpkg_name, "vendor"))

  # Create DESCRIPTION manually (not from template)
  desc_path <- usethis::proj_path(rpkg_name, "DESCRIPTION")
  desc_content <- sprintf(
    "Package: %s\nTitle: What the Package Does (One Line, Title Case)\nVersion: 0.0.0.9000\nAuthors@R:\n    person(\"First\", \"Last\", , \"first.last@example.com\", role = c(\"aut\", \"cre\"))\nDescription: What the package does (one paragraph).\nLicense: MIT + file LICENSE\nEncoding: UTF-8\nRoxygen: list(markdown = TRUE)\nRoxygenNote: 7.3.2\nSystemRequirements: Rust (>= 1.85)\nConfig/build/bootstrap: TRUE\nConfig/build/never-clean: true\nConfig/build/extra-sources: src/rust/Cargo.lock\n",
    data$package
  )
  writeLines(desc_content, desc_path)
  bullet_created(file.path(rpkg_name, "DESCRIPTION"))

  # Create LICENSE file (required by License: MIT + file LICENSE)
  license_path <- usethis::proj_path(rpkg_name, "LICENSE")
  license_content <- sprintf("YEAR: %s\nCOPYRIGHT HOLDER: %s authors\n",
                             format(Sys.Date(), "%Y"), data$package)
  writeLines(license_content, license_path)
  bullet_created(file.path(rpkg_name, "LICENSE"))

  # Create minimal NAMESPACE (required for configure.ac check)
  # Must include the roxygen2 header so devtools::document() can overwrite it
  namespace_path <- usethis::proj_path(rpkg_name, "NAMESPACE")
  namespace_content <- sprintf(
    "# Generated by roxygen2: do not edit by hand\n\nuseDynLib(%s, .registration = TRUE)\n",
    data$package
  )
  writeLines(namespace_content, namespace_path)
  bullet_created(file.path(rpkg_name, "NAMESPACE"))

  # R package files (from rpkg subdir of monorepo template)
  use_template("package.R", save_as = file.path(rpkg_name, "R", paste0(data$package, "-package.R")),
               subdir = "rpkg", data = data)

  # Build system files
  use_template("configure.ac", save_as = file.path(rpkg_name, "configure.ac"), subdir = "rpkg", data = data)
  use_template("bootstrap.R", save_as = file.path(rpkg_name, "bootstrap.R"), subdir = "rpkg")
  use_template("cleanup", save_as = file.path(rpkg_name, "cleanup"), subdir = "rpkg")
  use_template("cleanup.win", save_as = file.path(rpkg_name, "cleanup.win"), subdir = "rpkg")
  use_template("cleanup.ucrt", save_as = file.path(rpkg_name, "cleanup.ucrt"), subdir = "rpkg")
  use_template("configure.win", save_as = file.path(rpkg_name, "configure.win"), subdir = "rpkg")
  use_template("configure.ucrt", save_as = file.path(rpkg_name, "configure.ucrt"), subdir = "rpkg")

  # Ensure cleanup and configure scripts are executable
  for (script in c("cleanup", "cleanup.win", "cleanup.ucrt", "configure.win", "configure.ucrt")) {
    script_path <- usethis::proj_path(rpkg_name, script)
    if (fs::file_exists(script_path)) {
      fs::file_chmod(script_path, "755")
    }
  }

  # src/ files
  use_template("Makevars.in", save_as = file.path(rpkg_name, "src", "Makevars.in"), subdir = "rpkg")
  # stub.c — minimal C file so R's build system produces a shared library
  stub_src <- template_path("stub.c", subdir = "rpkg")
  fs::file_copy(stub_src, usethis::proj_path(rpkg_name, "src", "stub.c"), overwrite = TRUE)
  bullet_created(file.path(rpkg_name, "src", "stub.c"))
  use_template("win.def.in", save_as = file.path(rpkg_name, "src", "win.def.in"), subdir = "rpkg")

  # inst/include/ for cross-package header
  ensure_dir(usethis::proj_path(rpkg_name, "inst", "include"))
  use_template("mx_abi.h", save_as = file.path(rpkg_name, "inst", "include", "mx_abi.h"),
               subdir = file.path("rpkg", "inst_include"), data = data)

  # Rust project files
  use_template("Cargo.toml.tmpl", save_as = file.path(rpkg_name, "src", "rust", "Cargo.toml"), subdir = "rpkg", data = data)
  use_template("build.rs", save_as = file.path(rpkg_name, "src", "rust", "build.rs"), subdir = "rpkg")
  use_template("lib.rs", save_as = file.path(rpkg_name, "src", "rust", "lib.rs"), subdir = "rpkg", data = data)
  use_template("cargo-config.toml.in", save_as = file.path(rpkg_name, "src", "rust", "cargo-config.toml.in"), subdir = "rpkg")

  # Ignore files
  use_template("Rbuildignore", save_as = file.path(rpkg_name, ".Rbuildignore"), subdir = "rpkg")
  use_template("gitignore", save_as = file.path(rpkg_name, ".gitignore"), subdir = "rpkg")

  # Copy config.guess and config.sub (required for autoconf)
  # These go in tools/ directory per AC_CONFIG_AUX_DIR([tools]) in configure.ac
  ensure_dir(usethis::proj_path(rpkg_name, "tools"))
  for (script in c("config.guess", "config.sub")) {
    fs::file_copy(
      script_path(script),
      usethis::proj_path(rpkg_name, "tools", script),
      overwrite = TRUE
    )
    bullet_created(file.path(rpkg_name, "tools", script), "Copied")
  }

  # Copy vendor-local.R (standalone workspace vendor script for configure.ac)
  vendor_local_src <- template_path("vendor-local.R", subdir = "tools")
  if (fs::file_exists(vendor_local_src)) {
    fs::file_copy(vendor_local_src,
      usethis::proj_path(rpkg_name, "tools", "vendor-local.R"),
      overwrite = TRUE)
    bullet_created(file.path(rpkg_name, "tools", "vendor-local.R"), "Copied")
  }

  invisible(TRUE)
}

#' Add miniextendr to an existing package
#'
#' Sets up all miniextendr scaffolding. Automatically detects whether to use
#' the standalone R package template or the monorepo template based on context:
#'
#' - **In an R package** (has DESCRIPTION): Adds Rust scaffolding to current directory
#' - **In a Rust crate** (has Cargo.toml): Creates `rpkg/` subdirectory with R package
#'
#' This is an all-in-one function that calls all the individual `use_miniextendr_*()`
#' functions.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param template_type Template type: "auto" (detect from directory structure),
#'   "rpkg" for standalone R package, or "monorepo" for Rust workspace.
#'   Default is "auto" which auto-detects based on whether Cargo.toml or DESCRIPTION exists.
#' @param rpkg_name Name of the R package subdirectory for monorepo template
#'   (default: derived from package name). Only used when template_type is "monorepo".
#' @param miniextendr_version Version of miniextendr to vendor (default: "main").
#'   For monorepo projects, vendoring is only needed for CRAN submission.
#' @param local_path Optional path to local miniextendr repository. If provided,
#'   vendors from local path instead of downloading from GitHub. Useful for
#'   development and testing before the package is published.
#' @return Invisibly returns TRUE
#' @export
use_miniextendr <- function(path = ".",
                            template_type = "auto", rpkg_name = NULL,
                            miniextendr_version = "main", local_path = NULL) {
  with_project(path)
  # Warn if not at git workspace root
  git_available <- nzchar(Sys.which("git"))
  if (git_available) {
    git_root <- tryCatch(
      {
        res <- run_command("git", c("rev-parse", "--show-toplevel"))
        trimws(res)
      },
      warning = function(w) NULL,
      error = function(e) NULL
    )
    if (!is.null(git_root) &&
        normalizePath(git_root) != normalizePath(getwd())) {
      warning(
        "use_miniextendr() is not being called from the git workspace root. ",
        "Current directory: ", getwd(), "\n",
        "Git workspace root: ", git_root,
        call. = FALSE
      )
    }
  }

  cli::cli_h1("Setting up miniextendr")

  # Auto-detect template type if requested
  if (template_type == "auto") {
    detected <- detect_project_type()
    if (is.null(detected)) {
      cli::cli_alert_info("Could not auto-detect project type, defaulting to 'rpkg'")
      template_type <- "rpkg"
    } else {
      template_type <- detected
      cli::cli_alert_info("Detected project type: {.val {template_type}}")
    }
  }

  # Set template type for this session
  set_template_type(template_type)
  on.exit(set_template_type("rpkg"), add = TRUE)

  # Check prerequisites
  check_rust()

  # Handle monorepo differently: create rpkg/ subdirectory
  if (template_type == "monorepo") {
    # Derive package name from Cargo.toml (convert my-crate → my.crate)
    package_name <- get_package_name_from_cargo()
    rpkg_name <- rpkg_name %||% package_name
    cli::cli_alert_info("Detected Rust project - creating R package in {.path {rpkg_name}/} subdirectory")
    cli::cli_alert_info("Using package name: {.val {package_name}}")

    data <- template_data(package = package_name, rpkg_name = rpkg_name)
    create_rpkg_subdirectory(data, rpkg_name = rpkg_name)

    # Vendor miniextendr crates
    cli::cli_h2("Vendoring miniextendr crates")
    vendor_miniextendr(
      version = miniextendr_version,
      dest = usethis::proj_path(rpkg_name, "vendor"),
      local_path = local_path
    )

    # Configuration file at workspace root
    cli::cli_h2("Creating configuration")
    use_miniextendr_config()

    cli::cli_h1("Setup complete!")
    cli::cli_alert_info("Next steps:")
    cli::cli_bullets(c(
      " " = "1. Edit {.path {rpkg_name}/src/rust/lib.rs} to add R-exposed functions",
      " " = "2. Run {.code just configure} to set up build system",
      " " = "3. Run {.code just rcmdinstall} to build and install"
    ))

    return(invisible(TRUE))
  }

  # Standard rpkg template: add to current R package directory
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
  use_miniextendr_stub()
  use_miniextendr_mx_abi()

  # R package files
  cli::cli_h2("Setting up R package")
  use_miniextendr_package_doc()
  use_miniextendr_rbuildignore()
  use_miniextendr_gitignore()

  # Vendor miniextendr crates
  cli::cli_h2("Vendoring miniextendr crates")
  vendor_miniextendr(version = miniextendr_version, local_path = local_path)

  # Auto-run autoconf if available
  has_configure <- FALSE
  if (nzchar(Sys.which("autoconf"))) {
    cli::cli_h2("Generating configure script")
    tryCatch(
      {
        miniextendr_autoconf()
        has_configure <- TRUE
      },
      error = function(e) {
        cli::cli_alert_warning("autoconf failed: {conditionMessage(e)}")
      }
    )
  }

  # Configuration file
  cli::cli_h2("Creating configuration")
  use_miniextendr_config()

  # Summary
  cli::cli_h1("Setup complete!")
  cli::cli_alert_info("Next steps:")
  if (has_configure) {
    cli::cli_bullets(c(
      " " = "1. Edit {.path src/rust/lib.rs} to add your Rust functions",
      " " = "2. Run {.code minirextendr::miniextendr_build()} to compile and install"
    ))
  } else {
    cli::cli_bullets(c(
      " " = "1. Edit {.path src/rust/lib.rs} to add your Rust functions",
      " " = "2. Install {.pkg autoconf}, then run {.code minirextendr::miniextendr_build()} to compile and install"
    ))
  }

  invisible(TRUE)
}
