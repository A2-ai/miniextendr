# High-level mx_* convenience functions

#' Build and install a miniextendr package
#'
#' Runs the full two-pass install dance: configure, install (compiles Rust),
#' generate R wrappers via `devtools::document()`, then install again
#' (incorporates the new wrappers). This is the single command that replaces
#' the manual `configure -> install -> document -> install` workflow.
#'
#' @param path Path to the R package root. Defaults to `"."`.
#' @param not_cran Logical. If `TRUE` (the default), sets `NOT_CRAN=true`
#'   for both configure and install steps.
#' @return Invisibly returns `TRUE` on success.
#' @export
mx_build <- function(path = ".", not_cran = TRUE) {
  path <- normalizePath(path, mustWork = TRUE)
  env_vars <- if (not_cran) c(NOT_CRAN = "true") else character()

  cli::cli_h1("mx_build: full two-pass install")

  # Step 1: Configure
  cli::cli_h2("Step 1/4: Configure")
  mx_configure(path = path, not_cran = not_cran)

  # Step 2: First install (compiles Rust code)
  cli::cli_h2("Step 2/4: First install (compile Rust)")
  install_package(path, env_vars)

  # Step 3: Generate R wrappers
  cli::cli_h2("Step 3/4: Generate R wrappers")
  withr::with_envvar(env_vars, {
    devtools::document(path)
  })
  cli::cli_alert_success("R wrappers regenerated")

  # Step 4: Second install (incorporate wrappers)
  cli::cli_h2("Step 4/4: Second install (with wrappers)")
  install_package(path, env_vars)

  cli::cli_alert_success("Package built and installed successfully.")
  invisible(TRUE)
}

#' Install an R package via R CMD INSTALL
#'
#' @param path Path to the package.
#' @param env_vars Named character vector of environment variables.
#' @noRd
install_package <- function(path, env_vars) {
  result <- withr::with_envvar(env_vars, {
    run_with_logging(
      "R",
      args = c("CMD", "INSTALL", path),
      log_prefix = "rcmdinstall",
      wd = dirname(path)
    )
  })
  check_result(result, "R CMD INSTALL")
}

#' Run cargo check on package Rust code
#'
#' Convenience wrapper that runs `cargo check` in the package's `src/rust/`
#' directory. Unlike [cargo_check()] which uses usethis project detection,
#' this accepts an explicit `path` argument making it easier to use from
#' scripts and the command line.
#'
#' @param path Path to the R package root. Defaults to `"."`.
#' @return Invisibly returns `TRUE` on success.
#' @export
mx_check_rust <- function(path = ".") {
  path <- normalizePath(path, mustWork = TRUE)
  rust_dir <- file.path(path, "src", "rust")

  if (!dir.exists(rust_dir)) {
    abort(c(
      "No src/rust/ directory found",
      "i" = "Expected Rust code at {.path {rust_dir}}"
    ))
  }

  cargo_toml <- file.path(rust_dir, "Cargo.toml")
  if (!file.exists(cargo_toml)) {
    abort(c(
      "No Cargo.toml found in src/rust/",
      "i" = "Run {.code mx_configure()} first"
    ))
  }

  check_rust()

  cli::cli_h1("mx_check_rust")
  cli::cli_alert("Running cargo check in {.path {rust_dir}}")

  result <- run_with_logging(
    "cargo",
    args = c("check", "--manifest-path", cargo_toml),
    log_prefix = "cargo-check",
    wd = rust_dir
  )
  check_result(result, "cargo check")

  cli::cli_alert_success("Rust code checks passed")
  invisible(TRUE)
}

#' Set up local development against a miniextendr checkout
#'
#' Adds a `[patch.crates-io]` section to `src/rust/Cargo.toml` that points
#' miniextendr crates at a local monorepo checkout. This lets you develop
#' against a local (possibly modified) copy of miniextendr without vendoring.
#'
#' @param path Path to the R package root. Defaults to `"."`.
#' @param monorepo_path Path to the local miniextendr monorepo root
#'   (the directory containing `miniextendr-api/`, `miniextendr-macros/`, etc.).
#' @return Invisibly returns `TRUE` on success.
#' @export
mx_use_local_dev <- function(path = ".", monorepo_path) {
  path <- normalizePath(path, mustWork = TRUE)
  monorepo_path <- normalizePath(monorepo_path, mustWork = TRUE)

  cargo_toml <- file.path(path, "src", "rust", "Cargo.toml")
  if (!file.exists(cargo_toml)) {
    abort(c(
      "No Cargo.toml found at {.path {cargo_toml}}",
      "i" = "Run {.code mx_configure()} or {.code use_miniextendr()} first"
    ))
  }

  # Validate monorepo has the expected crates
  crates <- c("miniextendr-api", "miniextendr-macros", "miniextendr-macros-core",
              "miniextendr-lint", "miniextendr-engine")
  missing <- crates[!dir.exists(file.path(monorepo_path, crates))]
  if (length(missing) > 0) {
    abort(c(
      "Missing crates in monorepo path",
      "x" = "Not found: {paste(missing, collapse = ', ')}",
      "i" = "Ensure {.path {monorepo_path}} is the miniextendr repo root"
    ))
  }

  cli::cli_h1("mx_use_local_dev")

  content <- readLines(cargo_toml, warn = FALSE)

  # Remove existing [patch.crates-io] section if present
  patch_start <- grep("^\\[patch\\.crates-io\\]", content)
  if (length(patch_start) > 0) {
    # Find the end: next section header or end of file
    next_section <- grep("^\\[", content)
    next_section <- next_section[next_section > patch_start[1]]
    patch_end <- if (length(next_section) > 0) next_section[1] - 1 else length(content)
    content <- content[-(patch_start[1]:patch_end)]
    cli::cli_alert_info("Replaced existing [patch.crates-io] section")
  }

  # Compute relative path from src/rust/ to monorepo
  rust_dir <- file.path(path, "src", "rust")
  rel_path <- fs::path_rel(monorepo_path, rust_dir)

  # Build patch section
  patch_lines <- c(
    "",
    "[patch.crates-io]",
    sprintf('miniextendr-api = { path = "%s/miniextendr-api" }', rel_path),
    sprintf('miniextendr-macros = { path = "%s/miniextendr-macros" }', rel_path),
    sprintf('miniextendr-macros-core = { path = "%s/miniextendr-macros-core" }', rel_path),
    sprintf('miniextendr-lint = { path = "%s/miniextendr-lint" }', rel_path),
    sprintf('miniextendr-engine = { path = "%s/miniextendr-engine" }', rel_path)
  )

  content <- c(content, patch_lines)
  writeLines(content, cargo_toml)

  cli::cli_alert_success("Added [patch.crates-io] pointing to {.path {monorepo_path}}")
  cli::cli_alert_info("Run {.code mx_configure()} to regenerate build files")
  invisible(TRUE)
}

#' Create a new miniextendr R package
#'
#' One-step package creation: creates an R package directory with
#' [usethis::create_package()] and immediately scaffolds miniextendr
#' with [use_miniextendr()]. Equivalent to running `create_package()`
#' followed by `use_miniextendr()`.
#'
#' @param path Path where the package directory will be created.
#' @param name Package name. Defaults to `basename(path)`.
#' @param local_path Optional path to a local miniextendr repo for vendoring.
#'   Passed to [use_miniextendr()].
#' @return Invisibly returns the normalized path to the created package.
#' @export
mx_new <- function(path, name = basename(path), local_path = NULL) {
  cli::cli_h1("Creating new miniextendr package: {.val {name}}")

  usethis::create_package(
    path,
    fields = list(Package = name),
    open = FALSE,
    rstudio = FALSE
  )

  usethis::proj_set(path)
  use_miniextendr(template_type = "rpkg", local_path = local_path)

  path <- normalizePath(path, mustWork = TRUE)
  cli::cli_alert_success("New miniextendr package {.val {name}} created at {.path {path}}")
  invisible(path)
}
