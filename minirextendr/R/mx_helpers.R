# Convenience helpers for miniextendr development workflows

#' Run cargo check on package Rust code
#'
#' Runs `cargo check` in the package's `src/rust/` directory. Accepts an
#' explicit path, making it easy to use from scripts and CI (unlike
#' [cargo_check()] which uses usethis project detection).
#'
#' @param path Path to the R package root. Defaults to `"."`.
#' @return Invisibly returns `TRUE` on success.
#' @export
miniextendr_check_rust <- function(path = ".") {
  path <- normalizePath(path, mustWork = TRUE)
  rust_dir <- file.path(path, "src", "rust")

  if (!dir.exists(rust_dir)) {
    cli::cli_abort(c(
      "No src/rust/ directory found",
      "i" = "Expected Rust code at {.path {rust_dir}}"
    ))
  }

  cargo_toml <- file.path(rust_dir, "Cargo.toml")
  if (!file.exists(cargo_toml)) {
    cli::cli_abort(c(
      "No Cargo.toml found in src/rust/",
      "i" = "Run {.code miniextendr_configure()} first"
    ))
  }

  check_rust()

  cli::cli_h1("miniextendr_check_rust")
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

#' Link a package to a local miniextendr checkout for development
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
miniextendr_dev_link <- function(path = ".", monorepo_path) {
  path <- normalizePath(path, mustWork = TRUE)
  monorepo_path <- normalizePath(monorepo_path, mustWork = TRUE)

  cargo_toml <- file.path(path, "src", "rust", "Cargo.toml")
  if (!file.exists(cargo_toml)) {
    cli::cli_abort(c(
      "No Cargo.toml found at {.path {cargo_toml}}",
      "i" = "Run {.code miniextendr_configure()} or {.code use_miniextendr()} first"
    ))
  }

  # Validate monorepo has the expected crates
  crates <- c("miniextendr-api", "miniextendr-macros", "miniextendr-macros-core",
              "miniextendr-lint", "miniextendr-engine")
  missing <- crates[!dir.exists(file.path(monorepo_path, crates))]
  if (length(missing) > 0) {
    cli::cli_abort(c(
      "Missing crates in monorepo path",
      "x" = "Not found: {paste(missing, collapse = ', ')}",
      "i" = "Ensure {.path {monorepo_path}} is the miniextendr repo root"
    ))
  }

  cli::cli_h1("miniextendr_dev_link")

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
  cli::cli_alert_info("Run {.code miniextendr_configure()} to regenerate build files")
  invisible(TRUE)
}
