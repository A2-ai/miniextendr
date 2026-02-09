# Workflow helper functions

#' Run autoconf to generate configure script
#'
#' Runs `autoconf -vif` in the package root to regenerate the configure
#' script from configure.ac. Requires autoconf to be installed.
#'
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_autoconf <- function() {
  check_autoconf()

  cli::cli_alert("Running autoconf...")

  result <- run_with_logging(
    "autoconf",
    args = c("-v", "-i", "-f"),
    log_prefix = "autoconf",
    wd = usethis::proj_get()
  )

  check_result(result, "autoconf")

  # Make configure executable
  configure_path <- usethis::proj_path("configure")
  if (fs::file_exists(configure_path)) {
    fs::file_chmod(configure_path, "755")
    cli::cli_alert_success("Generated {.path configure}")
  }

  invisible(TRUE)
}

#' Run configure script
#'
#' Runs `./configure` in the package root to generate Makevars,
#' Cargo.toml, and other build files from templates.
#'
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_configure <- function() {
  configure_path <- usethis::proj_path("configure")

  if (!fs::file_exists(configure_path)) {
    abort(c(
      "configure script not found",
      "i" = "Run {.code minirextendr::miniextendr_autoconf()} first"
    ))
  }

  # Ensure configure is executable
  perms <- fs::file_info(configure_path)$permissions
  if (!grepl("x", as.character(perms))) {
    cli::cli_alert_info("Making {.path configure} executable")
    fs::file_chmod(configure_path, "755")
  }

  cli::cli_alert("Running ./configure...")

  result <- run_with_logging(
    "./configure",
    log_prefix = "configure",
    wd = usethis::proj_get(),
    env = devtools::r_env_vars()
  )

  check_result(result, "./configure")

  # Also mention config.log if it exists
  config_log <- usethis::proj_path("config.log")
  if (fs::file_exists(config_log)) {
    cli::cli_alert_info("Configure log also saved to: {.path config.log}")
  }

  cli::cli_alert_success("Generated build files")
  invisible(TRUE)
}

#' Generate R wrapper functions from Rust
#'
#' Runs the document binary to generate miniextendr_wrappers.R from
#' the Rust source. Requires the package to be built first.
#'
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_document <- function() {
  check_rust()

  cargo_toml <- usethis::proj_path("src", "rust", "Cargo.toml")
  if (!fs::file_exists(cargo_toml)) {
    abort(c(
      "Cargo.toml not found",
      "i" = "Run {.code minirextendr::miniextendr_configure()} first"
    ))
  }

  cli::cli_alert("Running document binary...")

  result <- run_with_logging(
    "cargo",
    args = c("run", "--bin", "document", "--release"),
    log_prefix = "document",
    wd = usethis::proj_path("src", "rust")
  )

  check_result(result, "document binary")

  # Copy generated wrappers to R/
  # The document binary writes {pkg_name}-wrappers.R in src/rust/
  pkg_name <- tryCatch(
    desc::desc_get("Package", file = usethis::proj_path("DESCRIPTION"))[[1]],
    error = function(e) NULL
  )
  if (!is.null(pkg_name)) {
    wrapper_name <- paste0(pkg_name, "-wrappers.R")
    src_wrappers <- usethis::proj_path("src", "rust", wrapper_name)
    r_wrappers <- usethis::proj_path("R", wrapper_name)

    if (fs::file_exists(src_wrappers)) {
      fs::file_copy(src_wrappers, r_wrappers, overwrite = TRUE)
      cli::cli_alert_success("Generated {.path {file.path('R', wrapper_name)}}")
    }
  }

  invisible(TRUE)
}

#' Full miniextendr build workflow
#'
#' Runs the complete two-pass install: autoconf -> configure -> install
#' (compiles Rust) -> document (generates R wrappers) -> install again
#' (incorporates wrappers). The two installs are needed because R wrapper
#' generation requires the compiled Rust `document` binary.
#'
#' @param install Whether to run `R CMD INSTALL` steps. If `FALSE`, only
#'   runs autoconf + configure + document.
#' @param not_cran Logical. If `TRUE` (the default), sets `NOT_CRAN=true`
#'   for configure and install steps.
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_build <- function(install = TRUE, not_cran = TRUE) {
  cli::cli_h1("miniextendr build workflow")

  env_vars <- if (not_cran) c(NOT_CRAN = "true") else character()
  pkg_path <- usethis::proj_get()

  cli::cli_h2("Step 1: autoconf")
  miniextendr_autoconf()

  cli::cli_h2("Step 2: configure")
  withr::with_envvar(env_vars, miniextendr_configure())

  if (install) {
    cli::cli_h2("Step 3: first install (compile Rust)")
    if (!requireNamespace("devtools", quietly = TRUE)) {
      warn("devtools not installed, skipping install step")
    } else {
      withr::with_envvar(env_vars, {
        devtools::install(pkg_path, upgrade = "never", quiet = TRUE)
      })
      cli::cli_alert_success("Installed package (first pass)")
    }
  }

  cli::cli_h2("Step 4: document (generate R wrappers)")
  miniextendr_document()

  cli::cli_h2("Step 5: roxygen2 (update NAMESPACE + man pages)")
  if (!requireNamespace("devtools", quietly = TRUE)) {
    warn("devtools not installed, skipping roxygen2 step")
  } else {
    devtools::document(pkg_path)
    cli::cli_alert_success("Updated NAMESPACE and documentation")
  }

  if (install) {
    cli::cli_h2("Step 6: final install (with R wrappers + NAMESPACE)")
    withr::with_envvar(env_vars, {
      devtools::install(pkg_path, upgrade = "never", quiet = TRUE)
    })
    cli::cli_alert_success("Installed package (final pass)")
  }

  cli::cli_alert_success("Build complete!")
  invisible(TRUE)
}

#' Prepare vendor tarball for CRAN submission
#'
#' Vendors all dependencies and compresses them into `inst/vendor.tar.xz`
#' for offline CRAN builds. This calls [vendor_crates_io()] internally,
#' then strips Cargo.lock checksums and compresses.
#'
#' Run this before `R CMD build` when preparing a CRAN submission.
#'
#' @return Invisibly returns the path to the created tarball.
#' @export
miniextendr_vendor <- function() {
  cli::cli_h1("miniextendr vendor workflow")

  # Step 1: cargo vendor + strip (delegates to vendor_crates_io)
  cli::cli_h2("Step 1: vendor all dependencies")
  vendor_crates_io()

  vendor_dir <- usethis::proj_path("vendor")
  lockfile <- usethis::proj_path("src", "rust", "Cargo.lock")
  inst_dir <- usethis::proj_path("inst")
  tarball <- fs::path(inst_dir, "vendor.tar.xz")

  # Step 2: strip checksums from Cargo.lock (vendored crates have empty checksums)
  if (fs::file_exists(lockfile)) {
    lock_content <- readLines(lockfile, warn = FALSE)
    lock_content <- lock_content[!grepl("^checksum = ", lock_content)]
    writeLines(lock_content, lockfile)
  }

  # Step 3: compress into inst/vendor.tar.xz
  cli::cli_h2("Step 2: compress vendor tarball")
  fs::dir_create(inst_dir)

  # Create staging directory for clean compression
  staging <- fs::path_temp("vendor-compress")
  on.exit(unlink(staging, recursive = TRUE), add = TRUE)
  if (fs::dir_exists(staging)) fs::dir_delete(staging)
  fs::dir_create(staging)
  fs::dir_copy(vendor_dir, fs::path(staging, "vendor"))

  # Truncate .md files (avoids CRAN notes about non-portable content)
  md_files <- fs::dir_ls(fs::path(staging, "vendor"), recurse = TRUE, glob = "*.md")
  for (f in md_files) {
    writeLines(character(), f)
  }

  # Create xz-compressed tarball
  px_result <- processx::run(
    "tar", c("-cJf", tarball, "-C", staging, "vendor"),
    error_on_status = FALSE
  )
  if (px_result$status != 0) {
    abort(c(
      "Failed to create vendor tarball",
      "i" = px_result$stderr
    ))
  }

  size_mb <- round(as.numeric(fs::file_size(tarball)) / 1024 / 1024, 1)
  cli::cli_alert_success("Created {.path inst/vendor.tar.xz} ({size_mb} MB)")
  cli::cli_alert_info("Include this in your CRAN submission (R CMD build will bundle it)")

  invisible(tarball)
}

#' Run R CMD check on a miniextendr package
#'
#' Builds the package tarball and runs R CMD check. Ensures dependencies
#' are vendored so the check works in the isolated temp directory where
#' R CMD check unpacks the tarball.
#'
#' @param args Character vector of extra arguments passed to `R CMD check`.
#'   Defaults to `c("--as-cran", "--no-manual")`.
#' @param error_on Severity level to error on. One of `"error"`, `"warning"`,
#'   or `"note"`. Passed to [rcmdcheck::rcmdcheck()].
#' @param build_args Character vector of extra arguments passed to `R CMD build`.
#' @return The [rcmdcheck::rcmdcheck()] result object, invisibly.
#' @export
miniextendr_check <- function(args = c("--as-cran", "--no-manual"),
                               error_on = "warning",
                               build_args = character()) {
  if (!requireNamespace("rcmdcheck", quietly = TRUE)) {
    abort(c(
      "rcmdcheck is required for miniextendr_check()",
      "i" = 'Install it with: install.packages("rcmdcheck")'
    ))
  }

  cli::cli_h1("miniextendr check workflow")
  pkg_path <- usethis::proj_get()

  cli::cli_h2("Step 1: build (autoconf + configure + install + document)")
  miniextendr_build(install = TRUE, not_cran = TRUE)

  cli::cli_h2("Step 2: R CMD check")
  cli::cli_alert("Running rcmdcheck with args: {.val {args}}")

  result <- rcmdcheck::rcmdcheck(
    pkg_path,
    args = args,
    build_args = build_args,
    error_on = error_on
  )

  invisible(result)
}
