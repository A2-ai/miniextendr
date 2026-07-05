# Package creation functions

#' Create a new miniextendr package
#'
#' Creates a new R package with full miniextendr scaffolding, ready for
#' Rust development. This combines `usethis::create_package()` with
#' `use_miniextendr()`.
#'
#' Works non-interactively even when `path` is **already a project** (e.g. an
#' existing git repo). `usethis::create_package()` normally raises an
#' interactive "would be nested inside an existing project" challenge in that
#' case, which aborts in a non-interactive session. To scaffold cleanly we set
#' `options(usethis.allow_nested_project = TRUE)` for the duration of the call
#' (restored on exit), suppressing that prompt.
#'
#' @param path Path where to create the package
#' @inheritParams usethis::create_package
#' @return Path to the created package (invisibly)
#' @export
create_miniextendr_package <- function(path, open = interactive(),
                                        rstudio = TRUE, check_name = FALSE) {
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

  # Allow scaffolding into an existing project (e.g. a git repo) without the
  # interactive nested-project challenge that would otherwise abort a
  # non-interactive session. Saved + restored via base on.exit() (withr is
  # Suggests-only, matching the rest of minirextendr).
  old_nested <- getOption("usethis.allow_nested_project")
  options(usethis.allow_nested_project = TRUE)
  on.exit(options(usethis.allow_nested_project = old_nested), add = TRUE)

  # Create basic package
  usethis::create_package(
    path,
    open = FALSE,
    rstudio = rstudio,
    check_name = check_name
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
#'   Passed to [miniextendr_vendor()]. Ignored when `local_path` is provided.
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
    crate_name_rs = to_rust_name(crate_name),
    rpkg_name = rpkg_name,
    features_var = "CARGO_FEATURES",
    year = format(Sys.Date(), "%Y")
  )

  # Root workspace files
  cli::cli_h2("Creating workspace root")
  use_template("Cargo.toml.tmpl", save_as = "Cargo.toml", data = data)
  use_template("gitignore", save_as = ".gitignore", data = data)

  # Standalone version-sync helper: `Rscript tools/bump-version.R --sync`
  # keeps DESCRIPTION / Cargo.toml / configure.ac versions in lockstep.
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

  # Git hooks
  cli::cli_h2("Installing git hooks")
  tryCatch(
    use_miniextendr_git_hooks(),
    error = function(e) {
      cli::cli_alert_info("Tip: run {.code minirextendr::use_miniextendr_git_hooks()} to install git hooks")
    }
  )

  cli::cli_h1("Monorepo created!")
  cli::cli_alert_info("Next steps:")
  cli::cli_bullets(c(
    " " = "1. Edit {.path {crate_name}/src/lib.rs} for your main Rust library",
    " " = "2. Edit {.path {rpkg_name}/src/rust/lib.rs} for R-exposed functions",
    " " = "3. {.code cd {rpkg_name} && minirextendr::miniextendr_build()} to compile, generate R wrappers + NAMESPACE, and install"
  ))

  if (open) {
    usethis::proj_activate(path)
  }

  invisible(path)
}

#' Create rpkg subdirectory for monorepo template
#'
#' Reuses the standalone `use_miniextendr_*()` helpers (passing `path` as the
#' rpkg subdirectory and `subdir = "rpkg"` so they read from
#' `templates/monorepo/rpkg/` instead of `templates/<type>/`) for every file
#' whose content only depends on the package name — this is what keeps the
#' monorepo scaffold in lockstep with the standalone one instead of
#' re-implementing the sequence by hand. Genuinely monorepo-specific content
#' (DESCRIPTION shape, the Rust workspace-sibling `Cargo.toml`/`lib.rs`) is
#' still written directly here. See audit
#' `2026-07-03-dogfooding-minirextendr-r.md` finding #4.
#'
#' @param data Template data list
#' @param rpkg_name Name of the R package subdirectory (default: "rpkg")
#' @noRd
create_rpkg_subdirectory <- function(data, rpkg_name = "rpkg") {
  # Create directory structure
  ensure_dir(usethis::proj_path(rpkg_name, "R"))
  ensure_dir(usethis::proj_path(rpkg_name, "src", "rust"))
  ensure_dir(usethis::proj_path(rpkg_name, "vendor"))

  rpkg_path <- usethis::proj_path(rpkg_name)

  # Create DESCRIPTION manually (not from template — Title/Description/
  # Authors@R placeholders differ from usethis::use_description()'s output,
  # and there's never an existing DESCRIPTION to update here). Config/build/*
  # fields come from the same MX_CONFIG_BUILD_FIELDS constant the standalone
  # path applies via mx_desc_set() in use_miniextendr_description().
  desc_path <- usethis::proj_path(rpkg_name, "DESCRIPTION")
  config_build_lines <- paste0(names(MX_CONFIG_BUILD_FIELDS), ": ",
                               MX_CONFIG_BUILD_FIELDS, collapse = "\n")
  desc_content <- sprintf(
    "Package: %s\nTitle: What the Package Does (One Line, Title Case)\nVersion: 0.0.0.9000\nAuthors@R:\n    person(\"First\", \"Last\", , \"first.last@example.com\", role = c(\"aut\", \"cre\"))\nDescription: What the package does (one paragraph).\nLicense: MIT + file LICENSE\nEncoding: UTF-8\nSystemRequirements: Rust (>= 1.85)\n%s\nConfig/roxygen2/markdown: TRUE\nConfig/roxygen2/version: 8.0.0\n",
    data$package, config_build_lines
  )
  writeLines(desc_content, desc_path)
  bullet_created(file.path(rpkg_name, "DESCRIPTION"))

  # Create LICENSE file (required by License: MIT + file LICENSE) — shared
  # body with use_miniextendr_description() via mx_license_content().
  license_path <- usethis::proj_path(rpkg_name, "LICENSE")
  writeLines(mx_license_content(data$package), license_path)
  bullet_created(file.path(rpkg_name, "LICENSE"))

  # Create minimal NAMESPACE (required for configure.ac check). Must include
  # the roxygen2 header so devtools::document() can overwrite it — shared
  # content with use_miniextendr_namespace() via mx_minimal_namespace().
  namespace_path <- usethis::proj_path(rpkg_name, "NAMESPACE")
  writeLines(mx_minimal_namespace(data$package), namespace_path)
  bullet_created(file.path(rpkg_name, "NAMESPACE"))

  # From here on, DESCRIPTION exists at rpkg_path, so the use_miniextendr_*()
  # helpers below can resolve the package name themselves via template_data()
  # when called with path = rpkg_path.

  # R package files (from rpkg subdir of monorepo template)
  use_miniextendr_package_doc(path = rpkg_path, subdir = "rpkg")

  # Build system files
  use_miniextendr_configure(path = rpkg_path, subdir = "rpkg")
  use_miniextendr_bootstrap(path = rpkg_path, subdir = "rpkg")
  use_miniextendr_cleanup(path = rpkg_path, subdir = "rpkg")
  use_miniextendr_configure_win(path = rpkg_path, subdir = "rpkg")
  use_miniextendr_config_scripts(path = rpkg_path, subdir = "rpkg")

  # src/ files
  use_miniextendr_makevars(path = rpkg_path, subdir = "rpkg")
  use_miniextendr_stub(path = rpkg_path, subdir = "rpkg")
  use_miniextendr_mx_abi(path = rpkg_path, subdir = "rpkg")

  # Rust project files. Cargo.toml.tmpl and lib.rs are genuinely
  # monorepo-specific (they reference the workspace-sibling crate_name /
  # crate_name_rs, which use_miniextendr_rust()'s standalone templates don't
  # have) so they stay hand-rolled; build.rs is byte-identical to the
  # standalone template and goes through the shared helper.
  use_template("Cargo.toml.tmpl", save_as = file.path(rpkg_name, "src", "rust", "Cargo.toml"), subdir = "rpkg", data = data)
  use_miniextendr_build_rs(path = rpkg_path, subdir = "rpkg")
  use_template("lib.rs", save_as = file.path(rpkg_name, "src", "rust", "lib.rs"), subdir = "rpkg", data = data)

  # Ignore files
  use_template("Rbuildignore", save_as = file.path(rpkg_name, ".Rbuildignore"), subdir = "rpkg")
  use_template("gitignore", save_as = file.path(rpkg_name, ".gitignore"), subdir = "rpkg")

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
#' If the target package has no `NAMESPACE` (e.g. it was set up via
#' `usethis::use_description()` in an existing repo rather than
#' `usethis::create_package()`), a minimal roxygen2-managed `NAMESPACE`
#' containing `useDynLib(<pkg>, .registration = TRUE)` is seeded so the first
#' build does not fail on the configure-time guard. A later
#' `devtools::document()` populates it from your roxygen comments.
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
#' @param claude_skills Whether to install the bundled Claude Code skill set
#'   via [use_claude_skills()] (default `TRUE`). The skills are agent-facing
#'   documentation only — they are `.Rbuildignore`d and never affect the built
#'   package.
#' @return Invisibly returns TRUE
#' @export
use_miniextendr <- function(path = ".",
                            template_type = "auto", rpkg_name = NULL,
                            miniextendr_version = "main", local_path = NULL,
                            claude_skills = TRUE) {
  with_project(path)
  # Warn if the package directory being scaffolded is not at a git workspace
  # root. Resolve everything from the *project* directory, not getwd():
  # with_project() sets the usethis project without changing the working dir
  # (setwd = FALSE), so getwd() is the caller's CWD — which, for an explicit
  # `path` (e.g. create_miniextendr_package("/tmp/pkg")), is unrelated to the
  # package being created and produced spurious warnings.
  git_available <- nzchar(Sys.which("git"))
  if (git_available) {
    proj_dir <- usethis::proj_get()
    res <- run_command("git", c("rev-parse", "--show-toplevel"), wd = proj_dir)
    status <- attr(res, "status")
    # Only a clean exit yields a real toplevel; a non-zero exit (no repo) leaves
    # the error text in `res` (stdout+stderr are merged), so gate on status.
    git_root <- if (is.null(status) || identical(as.integer(status), 0L)) {
      trimws(paste(res, collapse = "\n"))
    } else {
      NA_character_
    }
    if (!is.na(git_root) && nzchar(git_root) &&
        normalizePath(git_root, mustWork = FALSE) !=
          normalizePath(proj_dir, mustWork = FALSE)) {
      warning(
        "use_miniextendr() is not being called from the git workspace root. ",
        "Package directory: ", proj_dir, "\n",
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

    # Auto-run autoconf if available
    if (nzchar(Sys.which("autoconf"))) {
      cli::cli_h2("Generating configure script")
      tryCatch(
        miniextendr_autoconf(usethis::proj_path(rpkg_name)),
        error = function(e) {
          cli::cli_alert_warning("autoconf failed: {conditionMessage(e)}")
        }
      )
    }

    # Configuration file at workspace root
    cli::cli_h2("Creating configuration")
    use_miniextendr_config()

    if (claude_skills) {
      cli::cli_h2("Installing Claude Code skills")
      use_claude_skills()
    }

    cli::cli_h1("Setup complete!")
    cli::cli_alert_info("Next steps:")
    cli::cli_bullets(c(
      " " = "1. Edit {.path {rpkg_name}/src/rust/lib.rs} to add R-exposed functions",
      " " = "2. {.code cd {rpkg_name} && minirextendr::miniextendr_build()} to compile, generate R wrappers + NAMESPACE, and install"
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
  use_miniextendr_stub()
  use_miniextendr_mx_abi()

  # R package files
  cli::cli_h2("Setting up R package")
  use_miniextendr_package_doc()
  # Ensure a NAMESPACE exists. usethis::create_package() seeds one, but when
  # use_miniextendr() is run against a package set up another way (e.g.
  # usethis::use_description() in an existing repo) there may be none, and the
  # first build then fails on the configure guard. Seed a roxygen-managed
  # minimal NAMESPACE so the shared library loads and document() can take over.
  use_miniextendr_namespace()
  use_miniextendr_rbuildignore()
  use_miniextendr_gitignore()

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

  if (claude_skills) {
    cli::cli_h2("Installing Claude Code skills")
    use_claude_skills()
  }

  # Git hooks
  cli::cli_h2("Installing git hooks")
  tryCatch(
    use_miniextendr_git_hooks(),
    error = function(e) {
      cli::cli_alert_info("Tip: run {.code minirextendr::use_miniextendr_git_hooks()} to install git hooks")
    }
  )

  # Summary
  cli::cli_h1("Setup complete!")
  cli::cli_alert_info("Next steps:")
  if (has_configure) {
    cli::cli_bullets(c(
      " " = "1. Edit {.path src/rust/lib.rs} to add your Rust functions",
      " " = "2. Run {.code minirextendr::miniextendr_build()} to compile, generate R wrappers + NAMESPACE, and install"
    ))
  } else {
    cli::cli_bullets(c(
      " " = "1. Edit {.path src/rust/lib.rs} to add your Rust functions",
      " " = "2. Install {.pkg autoconf}, then run {.code minirextendr::miniextendr_build()} to compile, generate R wrappers + NAMESPACE, and install"
    ))
  }
  cli::cli_alert_info(
    "Note: plain {.code R CMD INSTALL .} skips wrapper generation; use {.code miniextendr_build()} or {.code devtools::document()} so {.code library(...)} sees your functions."
  )

  invisible(TRUE)
}
