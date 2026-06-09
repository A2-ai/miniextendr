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
#' autoconf -> configure -> `R CMD INSTALL` (compiles Rust + generates the
#' `R/<pkg>-wrappers.R` file via the cdylib pass) -> roxygen2 -> conditional
#' reinstall. This is the high-level workflow for building the entire package;
#' for compiling just the Rust crate, use [cargo_build()] instead.
#'
#' @section Why a conditional reinstall:
#' The `R/<pkg>-wrappers.R` file is generated *during* install, and its roxygen
#' `@export` tags are what [devtools::document()] reads to write `NAMESPACE`.
#' That creates a chicken-and-egg ordering: `document()` can only see the
#' wrappers after install, but install collates `NAMESPACE` (the export set the
#' installed image actually exposes) *before* `document()` rewrites it. On a
#' first build — or any build that adds or renames an exported function — the
#' freshly-installed image therefore lags the on-disk `NAMESPACE` by one build,
#' and `library(pkg)` exposes nothing new until the package is built a second
#' time.
#'
#' To collapse that into a single pass, this workflow snapshots `NAMESPACE`
#' before and after `document()`. If `document()` changed it (new or renamed
#' exports), the package is reinstalled once so the installed image matches the
#' freshly-written `NAMESPACE`. The reinstall happens at most once: after it,
#' the wrappers and `NAMESPACE` are already in their final form, so a repeat
#' `document()` would be a fixpoint and no further install is needed.
#'
#' @section Fresh-package bootstrap:
#' A brand-new package has no generated `R/<pkg>-wrappers.R` yet. The wrappers
#' are produced by the cdylib pass during a *source-mode* install
#' ([`./configure`][miniextendr_configure] with no `inst/vendor.tar.xz`).
#' A plain `devtools::install(build = TRUE)` cannot bootstrap them: its
#' `R CMD build` step runs `bootstrap.R`, which auto-vendors into
#' `inst/vendor.tar.xz`, and that latch flips `./configure` into offline
#' *tarball* mode — which ships pre-generated wrappers and skips wrapper
#' generation. With no wrappers to ship, the install either fails loudly
#' ("tarball is missing pre-generated wrappers") or, worse, leaves the
#' namespace empty.
#'
#' When `miniextendr_build()` detects that the wrappers file is absent, it
#' first runs an in-place source-mode bootstrap (clear any stale latch ->
#' configure -> `devtools::install(build = FALSE)` -> `devtools::document()`)
#' so the wrappers exist before the normal build path runs. The bootstrap
#' never creates `inst/vendor.tar.xz`, so it does not leak the tarball-mode
#' latch into subsequent dev iteration. Once wrappers are present, the normal
#' build path runs unchanged.
#'
#' @param path Path to the R package root, or `NULL` to use the active project.
#' @param install Whether to run the `R CMD INSTALL` steps. If `FALSE`, only
#'   runs autoconf + configure + roxygen2 (no compile, no reinstall).
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_build <- function(path = ".", install = TRUE) {
  with_project(path)
  cli::cli_h1("miniextendr build workflow")

  pkg_path <- usethis::proj_get()
  has_devtools <- requireNamespace("devtools", quietly = TRUE)

  cli::cli_h2("Step 1: autoconf")
  miniextendr_autoconf()

  cli::cli_h2("Step 2: configure")
  miniextendr_configure()

  if (install) {
    if (!has_devtools) {
      cli::cli_h2("Step 3: install (compile Rust + generate R wrappers)")
      cli::cli_warn("devtools not installed, skipping install step")
    } else {
      # Fresh-package bootstrap (#822). On a brand-new package the generated
      # R/<pkg>-wrappers.R does not exist yet. The normal devtools::install()
      # below uses build = TRUE, whose R CMD build step runs bootstrap.R, which
      # auto-vendors inst/vendor.tar.xz and flips ./configure into tarball mode
      # — a mode that SKIPS wrapper generation. So the very build that should
      # have created the wrappers can't, and library() exposes nothing. Detect
      # this and generate the wrappers first via an in-place source-mode install
      # (build = FALSE), which never touches inst/vendor.tar.xz.
      if (!wrappers_file_exists(pkg_path)) {
        cli::cli_h2("Step 3a: bootstrap wrappers (fresh package, source mode)")
        bootstrap_fresh_wrappers(pkg_path)
      }

      cli::cli_h2("Step 3: install (compile Rust + generate R wrappers)")
      install_pkg(pkg_path)
      cli::cli_alert_success("Installed package")
    }
  }

  cli::cli_h2("Step 4: roxygen2 (update NAMESPACE + man pages)")
  if (!has_devtools) {
    cli::cli_warn("devtools not installed, skipping roxygen2 step")
  } else {
    namespace_path <- fs::path(pkg_path, "NAMESPACE")
    namespace_before <- namespace_digest(namespace_path)

    devtools::document(pkg_path)
    cli::cli_alert_success("Updated NAMESPACE and documentation")

    namespace_after <- namespace_digest(namespace_path)

    # Chicken-and-egg fix: the wrappers file is generated during install and
    # document() reads its @export tags to write NAMESPACE. So the install in
    # Step 3 collated the *previous* NAMESPACE — if document() just added or
    # renamed exports, the installed image is one build behind. Reinstall once
    # so library(pkg) exposes the new exports in a single miniextendr_build()
    # pass. The reinstall is bounded to one pass: after it the wrappers and
    # NAMESPACE are already final, so re-running document() would be a fixpoint.
    if (install && has_devtools && !identical(namespace_before, namespace_after)) {
      cli::cli_h2("Step 5: reinstall (NAMESPACE exports changed)")
      cli::cli_alert_info(
        "{.code document()} changed {.path NAMESPACE}; reinstalling so the \\
         installed image exports the new wrappers."
      )
      install_pkg(pkg_path)
      cli::cli_alert_success("Reinstalled against updated NAMESPACE")
    }
  }

  cli::cli_alert_success("Build complete!")
  invisible(TRUE)
}

# Install the package via devtools, forcing the cdylib wrapper-gen pass.
#
# MINIEXTENDR_FORCE_WRAPPER_GEN forces regeneration of R/<pkg>-wrappers.R +
# wasm_registry.rs even if an inst/vendor.tar.xz latch has flipped configure
# into tarball mode (which otherwise skips it). Without this, a build run
# against a leaked tarball installs stale wrappers and library() exposes no
# functions. The prior value is restored on exit so the override doesn't leak
# into the rest of the R session.
install_pkg <- function(pkg_path) {
  old_force <- Sys.getenv("MINIEXTENDR_FORCE_WRAPPER_GEN", unset = NA)
  Sys.setenv(MINIEXTENDR_FORCE_WRAPPER_GEN = "1")
  on.exit(
    if (is.na(old_force)) Sys.unsetenv("MINIEXTENDR_FORCE_WRAPPER_GEN")
    else Sys.setenv(MINIEXTENDR_FORCE_WRAPPER_GEN = old_force),
    add = TRUE
  )
  tryCatch(
    devtools::install(pkg_path, upgrade = FALSE, quiet = FALSE),
    error = function(e) {
      cli::cli_abort(c(
        "Package installation failed",
        "i" = conditionMessage(e)
      ))
    }
  )
}

# Stable fingerprint of a NAMESPACE file for diffing before/after document().
# Returns NA_character_ when the file is absent (e.g. a brand-new package),
# which compares unequal to any real content via identical() and so triggers
# the reinstall on first build.
namespace_digest <- function(namespace_path) {
  if (!fs::file_exists(namespace_path)) {
    return(NA_character_)
  }
  paste(readLines(namespace_path, warn = FALSE), collapse = "\n")
}

#' Does the package's generated R wrapper file exist yet?
#'
#' The cdylib pass writes `R/<pkg>-wrappers.R`; its presence is the signal
#' that the package has been bootstrapped at least once. A fresh scaffold
#' has the Rust sources but no wrappers file.
#'
#' @param pkg_path Absolute path to the package root.
#' @return `TRUE` if any `R/*-wrappers.R` file exists.
#' @noRd
wrappers_file_exists <- function(pkg_path) {
  r_dir <- fs::path(pkg_path, "R")
  if (!fs::dir_exists(r_dir)) {
    return(FALSE)
  }
  length(fs::dir_ls(r_dir, glob = "*-wrappers.R", fail = FALSE)) > 0
}

#' Bootstrap a fresh package's R wrappers via a source-mode install
#'
#' On a brand-new package there is no generated `R/<pkg>-wrappers.R`. The
#' wrappers are emitted by the cdylib pass during a *source-mode*
#' `R CMD INSTALL` (no `inst/vendor.tar.xz` latch). A plain
#' `devtools::install(build = TRUE)` can't bootstrap them because its
#' `R CMD build` step runs `bootstrap.R`, which auto-vendors the tarball and
#' flips `./configure` into wrapper-skipping tarball mode.
#'
#' This helper reproduces the proven manual workaround in-process: clear any
#' stale latch so configure stays in source mode, re-run `./configure`, then
#' do an in-place `devtools::install(build = FALSE)` to generate the wrappers,
#' run `devtools::document()` so the NAMESPACE picks up the new exports, and
#' install once more so the namespace-aware install lands. It never creates
#' `inst/vendor.tar.xz`, so the tarball-mode latch does not leak into later
#' dev iteration.
#'
#' @param pkg_path Absolute path to the package root.
#' @return Invisibly `TRUE`.
#' @noRd
bootstrap_fresh_wrappers <- function(pkg_path) {
  cli::cli_alert_info(c(
    "No generated {.path R/*-wrappers.R} found — bootstrapping wrappers ",
    "via a source-mode install before the full build."
  ))

  # Clear any stale tarball-mode latch so ./configure resolves in source mode
  # (where the cdylib wrapper-gen pass runs). The bootstrap install is always
  # source-mode; leaving a latch behind would skip wrapper generation again.
  clear_install_mode_latch(pkg_path)

  # Re-configure in source mode now that the latch is gone, so the
  # tarball-mode .cargo/config.toml (if any) is replaced.
  miniextendr_configure(pkg_path)

  # Generate wrappers via an in-place install. build = FALSE skips R CMD build
  # (and therefore bootstrap.R's auto-vendor), keeping configure in source mode.
  tryCatch(
    devtools::install(pkg_path, build = FALSE, upgrade = FALSE, quiet = FALSE),
    error = function(e) {
      cli::cli_abort(c(
        "Bootstrap install (source mode) failed",
        "i" = conditionMessage(e)
      ))
    }
  )

  if (!wrappers_file_exists(pkg_path)) {
    cli::cli_abort(c(
      "Bootstrap install completed but no {.path R/*-wrappers.R} was generated.",
      "i" = paste(
        "Expected the cdylib wrapper-gen pass to write it. Check that",
        "{.code #[miniextendr]} functions are reachable from {.file src/rust/lib.rs}."
      )
    ))
  }

  # document() so NAMESPACE exports the freshly-generated wrappers, then
  # install once more so the installed package's namespace matches.
  devtools::document(pkg_path)
  tryCatch(
    devtools::install(pkg_path, build = FALSE, upgrade = FALSE, quiet = FALSE),
    error = function(e) {
      cli::cli_abort(c(
        "Bootstrap re-install (after document) failed",
        "i" = conditionMessage(e)
      ))
    }
  )

  cli::cli_alert_success("Bootstrapped R wrappers (source mode)")
  invisible(TRUE)
}

#' Remove the install-mode latch and its source-mode-incompatible siblings
#'
#' `inst/vendor.tar.xz` is the single signal that flips `./configure` into
#' offline tarball mode. The unpacked `vendor/` directory and the
#' tarball-mode `src/rust/.cargo/config.toml` are downstream artifacts of the
#' same mode. Clearing all three guarantees the next `./configure` resolves in
#' source mode. Safe and idempotent — no-op if nothing is present.
#'
#' @param pkg_path Absolute path to the package root.
#' @return Invisibly `TRUE`.
#' @noRd
clear_install_mode_latch <- function(pkg_path) {
  latch <- fs::path(pkg_path, "inst", "vendor.tar.xz")
  if (fs::file_exists(latch)) {
    fs::file_delete(latch)
    cli::cli_alert_info("Cleared stale {.path inst/vendor.tar.xz} latch.")
  }
  vendor_dir <- fs::path(pkg_path, "vendor")
  if (fs::dir_exists(vendor_dir)) {
    fs::dir_delete(vendor_dir)
  }
  cargo_dir <- fs::path(pkg_path, "src", "rust", ".cargo")
  if (fs::dir_exists(cargo_dir)) {
    fs::dir_delete(cargo_dir)
  }
  invisible(TRUE)
}

#' Prepare vendor tarball for CRAN submission
#'
#' High-level workflow that vendors all crate dependencies and compresses
#' them into `inst/vendor.tar.xz` for offline CRAN install. Wraps
#' [vendor_crates_io()] (which delegates to `cargo-revendor`) plus tarball
#' compression. `cargo-revendor` resolves against the local workspace when a
#' dev `[patch."<git-url>"]` override is present (so a cross-crate rename
#' resolves against the working tree, not git@main) and stamps the canonical
#' `git+url#<sha>` source into `Cargo.lock`; checksum lines are retained.
#'
#' Run this before `R CMD build` when preparing a CRAN submission.
#' Day-to-day development (`R CMD INSTALL .`, `devtools::install/test/load`)
#' does not need it: install mode is auto-detected from
#' `inst/vendor.tar.xz` presence, and without the file cargo resolves deps
#' over the network.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @return Invisibly returns the path to the created tarball.
#' @export
miniextendr_vendor <- function(path = ".") {
  with_project(path)
  cli::cli_h1("miniextendr vendor workflow")

  cargo_toml <- usethis::proj_path("src", "rust", "Cargo.toml")
  if (!fs::file_exists(cargo_toml)) {
    cli::cli_abort(c(
      "{.path src/rust/Cargo.toml} not found",
      "i" = "Run {.code miniextendr_configure()} first"
    ))
  }

  # Step 1: cargo revendor + CRAN-trim (delegates to vendor_crates_io).
  #
  # cargo-revendor resolves the dependency graph with the dev
  # [patch."git+url"] override active (it pins cargo's CWD to the manifest
  # dir, so a monorepo .cargo/config.toml is honoured), then stamps the
  # framework crates' `source = "git+url#<sha>"` attribution back into
  # Cargo.lock — the shape offline source-replacement needs. So a cross-crate
  # feature/dep rename resolves against the local workspace, not git@main
  # (#883), and there is no bare-git "regenerate the lock first" dance: the
  # step that disabled the patch was exactly what broke cross-surface renames.
  # In a standalone package (no [patch] override) cargo resolves the framework
  # crates from their git URL directly and the natural git source is kept.
  cli::cli_h2("Step 1: vendor all dependencies")
  vendor_crates_io()

  vendor_dir <- usethis::proj_path("vendor")
  lockfile <- usethis::proj_path("src", "rust", "Cargo.lock")
  inst_dir <- usethis::proj_path("inst")
  tarball <- fs::path(inst_dir, "vendor.tar.xz")

  # Step 3: compress into inst/vendor.tar.xz
  # Note: Cargo.lock checksum lines are intentionally retained. cargo-revendor
  # (post PR #408) writes valid .cargo-checksum.json entries with real SHA-256s,
  # so stripping `checksum = "..."` from Cargo.lock is no longer needed and
  # would diverge from the `just vendor` reference output.
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
  cli::cli_alert_warning(c(
    "{.path inst/vendor.tar.xz} flips {.code ./configure} into offline tarball mode."
  ))
  cli::cli_bullets(c(
    "i" = "Run {.code R CMD build .} to produce the release tarball, then delete {.path inst/vendor.tar.xz} to resume source-mode dev:",
    " " = "{.code unlink(\"inst/vendor.tar.xz\")}"
  ))

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
#' @seealso [miniextendr_check_static()] for a fast no-compile variant suitable
#'   for un-vendored packages.
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
  miniextendr_build(install = TRUE)

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
