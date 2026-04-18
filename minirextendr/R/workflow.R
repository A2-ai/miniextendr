# Workflow helper functions

#' Run autoconf to generate configure script
#'
#' Runs `autoconf -vif` in the package root to regenerate the configure
#' script from configure.ac. Requires autoconf to be installed.
#'
#' @param path Path to the R package root, or `NULL` to use the active project.
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_autoconf <- function(path = ".") {
  with_project(path)
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
#' @param path Path to the R package root, or `NULL` to use the active project.
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_configure <- function(path = ".") {
  with_project(path)
  configure_path <- usethis::proj_path("configure")

  if (!fs::file_exists(configure_path)) {
    cli::cli_abort(c(
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
    "bash",
    args = c("./configure"),
    log_prefix = "configure",
    wd = usethis::proj_get(),
    env = if (requireNamespace("devtools", quietly = TRUE)) devtools::r_env_vars() else character()
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

#' Full R package build workflow
#'
#' Runs the complete R package build pipeline:
#' autoconf -> configure -> R CMD INSTALL (compiles Rust + generates
#' R wrappers via cdylib) -> roxygen2. This is the high-level workflow
#' for building the entire package; for compiling just the Rust crate,
#' use [cargo_build()] instead.
#'
#' @param path Path to the R package root, or `NULL` to use the active project.
#' @param install Whether to run `R CMD INSTALL` step. If `FALSE`, only
#'   runs autoconf + configure.
#' @param not_cran Logical. If `TRUE` (the default), sets `NOT_CRAN=true`
#'   for configure and install steps.
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_build <- function(path = ".", install = TRUE, not_cran = TRUE) {
  with_project(path)
  cli::cli_h1("miniextendr build workflow")

  env_vars <- if (not_cran) c(NOT_CRAN = "true") else character()
  pkg_path <- usethis::proj_get()

  cli::cli_h2("Step 1: autoconf")
  miniextendr_autoconf()

  cli::cli_h2("Step 2: configure")
  with_envvars(env_vars, miniextendr_configure())

  if (install) {
    cli::cli_h2("Step 3: install (compile Rust + generate R wrappers)")
    if (!requireNamespace("devtools", quietly = TRUE)) {
      cli::cli_warn("devtools not installed, skipping install step")
    } else {
      tryCatch(
        with_envvars(env_vars, {
          devtools::install(pkg_path, upgrade = FALSE, quiet = FALSE)
        }),
        error = function(e) {
          cli::cli_abort(c(
            "Package installation failed",
            "i" = conditionMessage(e)
          ))
        }
      )
      cli::cli_alert_success("Installed package")
    }
  }

  cli::cli_h2("Step 4: roxygen2 (update NAMESPACE + man pages)")
  if (!requireNamespace("devtools", quietly = TRUE)) {
    cli::cli_warn("devtools not installed, skipping roxygen2 step")
  } else {
    devtools::document(pkg_path)
    cli::cli_alert_success("Updated NAMESPACE and documentation")
  }

  cli::cli_alert_success("Build complete!")
  invisible(TRUE)
}

#' Prepare vendor tarball for CRAN submission
#'
#' High-level workflow that vendors all external crate dependencies and
#' compresses them into `inst/vendor.tar.xz` for offline CRAN builds.
#' Calls [vendor_crates_io()] internally, then strips Cargo.lock
#' checksums and compresses.
#'
#' For vendoring the miniextendr workspace crates themselves, see
#' [vendor_miniextendr()]. For syncing vendor/ from a local checkout,
#' see [vendor_sync()].
#'
#' Run this before `R CMD build` when preparing a CRAN submission.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns the path to the created tarball.
#' @export
miniextendr_vendor <- function(path = ".") {
  with_project(path)
  cli::cli_h1("miniextendr vendor workflow")

  # Inform the user about path dependencies. cargo revendor extracts these
  # into vendor/ (unlike plain cargo vendor), but CRAN still requires that
  # the source tree referenced by `path = ...` is reachable at build time
  # — `use_vendor_lib()` handles the packaging side.
  path_deps <- check_path_deps()
  if (nrow(path_deps) > 0) {
    cli::cli_alert_info(
      "Found path dependencies in Cargo.toml (extracted by {.code cargo revendor}):"
    )
    for (i in seq_len(nrow(path_deps))) {
      cli::cli_bullets(c("i" = "{.val {path_deps$crate[i]}} -> {.path {path_deps$path[i]}}"))
    }
    cli::cli_alert_info(
      "If these paths are outside the R package tree, use {.code minirextendr::use_vendor_lib()}"
    )
  }

  # Step 1: cargo revendor + strip (delegates to vendor_crates_io)
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

  # Create xz-compressed tarball.
  # Suppress macOS xattr metadata (AppleDouble `._*` files + LIBARCHIVE.xattr.*
  # PAX headers) that trigger GNU tar warnings on CRAN Linux machines.
  # COPYFILE_DISABLE=1 stops `._*` files; --no-xattrs stops PAX headers.
  old_copyfile <- Sys.getenv("COPYFILE_DISABLE", unset = NA)
  Sys.setenv(COPYFILE_DISABLE = "1")
  on.exit(
    if (is.na(old_copyfile)) Sys.unsetenv("COPYFILE_DISABLE")
    else Sys.setenv(COPYFILE_DISABLE = old_copyfile),
    add = TRUE
  )
  tar_args <- c("-cJf", tarball, "-C", staging, "vendor")
  has_no_xattrs <- identical(
    suppressWarnings(tryCatch(
      system2(
        "tar",
        c("--no-xattrs", "-cf", "/dev/null", "--files-from", "/dev/null"),
        stdout = FALSE, stderr = FALSE
      ),
      error = function(e) 127L
    )),
    0L
  )
  if (has_no_xattrs) {
    tar_args <- c("--no-xattrs", tar_args)
  }
  tar_output <- system2("tar", tar_args, stdout = TRUE, stderr = TRUE)
  if (!is.null(attr(tar_output, "status"))) {
    cli::cli_abort(c(
      "Failed to create vendor tarball",
      "i" = paste(tar_output, collapse = "\n")
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
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param args Character vector of extra arguments passed to `R CMD check`.
#'   Defaults to `c("--as-cran", "--no-manual")`.
#' @param error_on Severity level to error on. One of `"error"`, `"warning"`,
#'   or `"note"`. Passed to [rcmdcheck::rcmdcheck()].
#' @param build_args Character vector of extra arguments passed to `R CMD build`.
#' @return The [rcmdcheck::rcmdcheck()] result object, invisibly.
#' @export
miniextendr_check <- function(path = ".",
                               args = c("--as-cran", "--no-manual"),
                               error_on = "warning",
                               build_args = character()) {
  with_project(path)
  if (!requireNamespace("rcmdcheck", quietly = TRUE)) {
    cli::cli_abort(c(
      "rcmdcheck is required for miniextendr_check()",
      "i" = 'Install it with: install.packages("rcmdcheck")'
    ))
  }

  cli::cli_h1("miniextendr check workflow")
  pkg_path <- usethis::proj_get()

  cli::cli_h2("Step 1: build (autoconf + configure + install + roxygen2)")
  miniextendr_build(install = TRUE, not_cran = TRUE)

  # Check for path dependencies that will fail R CMD check without vendor-lib
  path_deps <- check_path_deps()
  if (nrow(path_deps) > 0) {
    missing <- vapply(path_deps$crate, function(crate) {
      !file.exists(usethis::proj_path("inst", paste0(crate, "-lib.tar.gz")))
    }, logical(1))
    if (any(missing)) {
      missing_crates <- path_deps$crate[missing]
      cli::cli_alert_danger(
        "Path dependencies without vendor-lib tarballs will cause R CMD check to fail:"
      )
      for (crate in missing_crates) {
        dev_path <- path_deps$path[path_deps$crate == crate]
        cli::cli_bullets(c(
          "x" = '{.val {crate}}: run {.code minirextendr::use_vendor_lib("{crate}", dev_path = "{dev_path}")}'
        ))
      }
    }
  }

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
