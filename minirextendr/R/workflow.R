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
#' Runs the complete R package build pipeline: autoconf -> configure ->
#' compile the Rust crate and regenerate `R/<pkg>-wrappers.R` in the source
#' tree -> roxygen2 (`NAMESPACE` + `man/`) -> one `R CMD INSTALL`. This is the
#' high-level workflow for building the entire package; for compiling just the
#' Rust crate, use [cargo_build()] instead.
#'
#' @section Why the wrappers are regenerated before roxygen2 and the install:
#' `R/<pkg>-wrappers.R` is written by the wrapper-gen rule of the package's
#' `Makevars`, which loads the freshly linked shared object and asks it for the
#' R side of every `#[miniextendr]` item. Its roxygen `@export` tags are what
#' [devtools::document()] reads to write `NAMESPACE`, and its doc comments are
#' what `man/` is rendered from. Installing first therefore shipped a
#' `NAMESPACE` and `man/` that were one build behind the Rust sources: a new
#' export was missing from the installed image until a second build (#860), a
#' removed or renamed export made the install's namespace load-test fail with
#' `undefined exports: <old>` before roxygen2 could drop the entry (#1288), and
#' an edited doc comment reached the source `man/` but not the installed help
#' (#1549). Earlier versions worked around the first two with a conditional
#' reinstall and a deferred install failure.
#'
#' The steps now run in dependency order. Step 3 compiles the crate in the
#' source tree with [pkgbuild::compile_dll()] under
#' `MINIEXTENDR_FORCE_WRAPPER_GEN=1`: a libs-only `R CMD INSTALL` into a
#' throwaway library that links `src/<pkg>.so`, regenerates
#' `R/<pkg>-wrappers.R`, `src/rust/wasm_registry.rs` and the wrapper
#' provenance record in place, and never loads the namespace, so a stale
#' `NAMESPACE` cannot fail it. Step 4's `document()` reads the current
#' wrappers; its own `pkgload::load_all()` compile finds the crate built and
#' the record current, and only warns when `NAMESPACE` still lists an export
#' the regenerated wrappers no longer define. Step 5 installs the package
#' once, with `NAMESPACE`, `man/` and the wrappers already in their final
#' form. A brand-new package needs no special case: Step 3 writes its first
#' wrappers file the same way.
#'
#' `document()` loads the package from source through `pkgload::load_all()`
#' and leaves that development namespace registered in the session. Its export
#' table is the `NAMESPACE` as it stood *before* roxygen2 rewrote it, so a
#' `library()` call in the same session would attach an image missing every
#' export the build just added. The workflow unloads it after `document()`;
#' `library()` then loads the installed package (or, with `install = FALSE`,
#' `load_all()` reads the current `NAMESPACE`).
#'
#' @section Development bootstrap:
#' Installs select `MINIEXTENDR_BOOTSTRAP_MODE=dev` unless the caller supplied a
#' mode or a distribution vendor archive already exists. Updated scaffolds
#' package only path-dependency siblings, without xz or registry/Git vendoring.
#' The portable manifest is activated in R CMD build's temporary copy, leaving
#' the checkout's Cargo.toml unchanged. Ordinary bootstrap calls still default
#' to distribution mode. Requires a current cargo-revendor and updated templates.
#'
#' @section Source-tree restore:
#' The install step's `R CMD build` runs `bootstrap.R` in the source tree. In
#' distribution mode that seals `inst/vendor.tar.xz` there by design (the built
#' tarball must carry it) and freezes `src/rust/Cargo.toml`; left in place, the
#' latch would flip the next build into offline tarball mode (#1294).
#' `miniextendr_build()` snapshots the manifest and lockfile on entry and
#' restores them on exit, deleting a latch it created itself. A latch that
#' existed *before* the build is never deleted; a warning is emitted up front
#' instead, and `minirextendr_doctor()` / `miniextendr_clean_vendor_leak()`
#' point at the fix.
#'
#' @param path Path to the R package root, or `NULL` to use the active project.
#' @param install Whether to run the final `R CMD INSTALL`. If `FALSE`, the
#'   workflow stops after roxygen2: autoconf, configure, the compile with
#'   wrapper regeneration and `document()` still run, so the source tree is
#'   current, but nothing is installed.
#' @return Invisibly returns TRUE on success
#' @export
miniextendr_build <- function(path = ".", install = TRUE) {
  with_project(path)
  cli::cli_h1("miniextendr build workflow")
  report_minirextendr_installation()

  pkg_path <- usethis::proj_get()
  has_devtools <- requireNamespace("devtools", quietly = TRUE)

  # The install's `R CMD build` runs the package's bootstrap.R in the source
  # tree, which vendors when no inst/vendor.tar.xz is present: that FREEZES
  # src/rust/Cargo.toml (rewriting a `path = "../../../my-core"` sibling to
  # `vendor/my-core`) and leaves inst/vendor.tar.xz behind -- flipping the tree
  # into tarball mode and stranding the next source-mode build. The dev loop
  # must leave a clean source tree, so snapshot the manifest/lock + tarball
  # presence now and restore on exit (mirrors the `just cran-prep` trap,
  # git-independent). The restore never touches vendor/ or .cargo/: configure
  # owns .cargo/config.toml, and vendor/ may be user-provisioned (offline
  # crates.io deps).
  rust_manifest <- fs::path(pkg_path, "src", "rust", "Cargo.toml")
  rust_lock <- fs::path(pkg_path, "src", "rust", "Cargo.lock")
  rust_prefreeze <- fs::path(pkg_path, "src", "rust", ".Cargo.toml.prefreeze")
  vendor_tarball <- fs::path(pkg_path, "inst", "vendor.tar.xz")
  snap_manifest <- if (fs::file_exists(rust_manifest)) readLines(rust_manifest, warn = FALSE) else NULL
  snap_lock <- if (fs::file_exists(rust_lock)) readLines(rust_lock, warn = FALSE) else NULL
  tarball_preexisting <- fs::file_exists(vendor_tarball)
  # The development loop needs portable path siblings, not a distribution
  # vendor archive. Honor an explicit caller mode or pre-existing release latch.
  if (!tarball_preexisting && !nzchar(Sys.getenv("MINIEXTENDR_BOOTSTRAP_MODE"))) {
    withr::local_envvar(c(MINIEXTENDR_BOOTSTRAP_MODE = "dev"))
  }
  # A snapshot left by an earlier, never-restored freeze (#1509) means the
  # manifest we just read is the FROZEN one; restoring it later would only
  # re-freeze. Point at the fix and leave that snapshot alone.
  prefreeze_preexisting <- fs::file_exists(rust_prefreeze)
  if (prefreeze_preexisting) {
    cli::cli_warn(c(
      "Pre-existing {.path src/rust/.Cargo.toml.prefreeze}: an earlier {.code cargo revendor --freeze} was never restored.",
      "i" = "Run {.code miniextendr_clean_vendor_leak()} first to restore {.path src/rust/Cargo.toml} from that snapshot."
    ))
  }
  if (tarball_preexisting) {
    # A pre-existing latch is never deleted (it may be a deliberate
    # release-prep artifact) -- but with it in place every step runs in
    # offline tarball mode against the sealed archive. Warn loudly up front.
    cli::cli_warn(c(
      "Pre-existing {.path inst/vendor.tar.xz} latch: this tree builds in tarball mode.",
      "i" = "Cargo resolves dependencies from the sealed vendor archive, not from \\
             the workspace, so edits to vendored crates are not compiled.",
      "i" = "Delete {.path inst/vendor.tar.xz} (or run {.code minirextendr_doctor()}, \\
             which detects the stale latch) to resume source-mode development."
    ))
  }
  restore_dev_tree <- function() {
    if (!is.null(snap_manifest)) {
      writeLines(snap_manifest, rust_manifest)
      # bootstrap.R's `cargo revendor --freeze` left a pre-freeze snapshot for
      # out-of-process recovery (#1509); the in-memory snapshot just written
      # back is authoritative, so a snapshot created during this build is now
      # stale. One that pre-existed is somebody else's recovery aid: keep it.
      if (!prefreeze_preexisting && fs::file_exists(rust_prefreeze)) fs::file_delete(rust_prefreeze)
    }
    if (!is.null(snap_lock)) writeLines(snap_lock, rust_lock)
    if (!tarball_preexisting && fs::file_exists(vendor_tarball)) fs::file_delete(vendor_tarball)
    invisible(TRUE)
  }
  on.exit(restore_dev_tree(), add = TRUE)

  cli::cli_h2("Step 1: autoconf")
  miniextendr_autoconf()

  cli::cli_h2("Step 2: configure")
  miniextendr_configure()

  if (!has_devtools) {
    cli::cli_warn("devtools not installed, skipping the compile, roxygen2 and install steps")
    cli::cli_alert_success("Build complete!")
    return(invisible(TRUE))
  }

  cli::cli_h2("Step 3: compile Rust + regenerate R wrappers (source tree)")
  compile_and_generate_wrappers(pkg_path)
  cli::cli_alert_success("Compiled the crate and regenerated {.path R/*-wrappers.R}")

  cli::cli_h2("Step 4: roxygen2 (update NAMESPACE + man pages)")
  devtools::document(pkg_path)
  unload_dev_namespace(pkg_path)
  cli::cli_alert_success("Updated NAMESPACE and documentation")

  if (install) {
    cli::cli_h2("Step 5: install")
    install_pkg(pkg_path)
    cli::cli_alert_success("Installed package")
  }

  cli::cli_alert_success("Build complete!")
  invisible(TRUE)
}

# Step 4 leaves a development namespace behind: roxygen2 documents through
# pkgload::load_all(), and pkgload fills the namespace's export table from the
# NAMESPACE file as it is at load time, i.e. before roxygen2 rewrote it. In the
# same session, library(<pkg>) attaches an already-loaded namespace as is, so
# it would come up without the exports this build added (a fresh package: with
# none at all). Unload it so library() loads the installed image, or a later
# load_all() reads the current NAMESPACE. Only a pkgload-registered namespace
# is touched; an installed one the user loaded earlier is left alone (#1000).
unload_dev_namespace <- function(pkg_path) {
  pkg_name <- unname(mx_desc_get_field("Package", file = file.path(pkg_path, "DESCRIPTION")))
  if (pkgload::is_dev_package(pkg_name)) {
    pkgload::unload(pkg_name, quiet = TRUE)
  }
  invisible(pkg_name)
}

# Evaluate `expr` with MINIEXTENDR_FORCE_WRAPPER_GEN=1 in the session.
#
# The Makevars wrapper-gen rule reads the variable ([ -n "$$MINIEXTENDR_FORCE_
# WRAPPER_GEN" ]) two subprocess hops down (R CMD build -> R CMD INSTALL ->
# make); a session Sys.setenv() reaches it because pkgbuild's callr children
# inherit the parent environment (#911, pinned by a test). It forces
# regeneration even when the rule would otherwise reuse a wrapper file whose
# provenance record is current, or a pre-shipped one in a latched
# (tarball-mode) tree that would otherwise skip generation and leave
# library() with stale or no functions (#757). The prior value is restored
# afterwards so the override never leaks into the rest of the session, in
# particular not into document(), whose roxygenise compile is meant to reuse
# the wrappers Step 3 just wrote.
with_forced_wrapper_gen <- function(expr) {
  withr::local_envvar(c(MINIEXTENDR_FORCE_WRAPPER_GEN = "1"))
  expr
}

# Step 3: compile the crate and regenerate the wrappers in the source tree.
#
# pkgbuild::compile_dll() runs a libs-only `R CMD INSTALL --no-test-load` into
# a throwaway library: configure + make in `src/`, which links `src/<pkg>.so`
# and runs the Makevars wrapper-gen rule against it, writing
# R/<pkg>-wrappers.R, src/rust/wasm_registry.rs and tools/wrapper-inputs.rds
# in place. force = TRUE skips pkgbuild's own source-vs-DLL mtime check; cargo
# decides what to rebuild. Nothing here loads the package namespace, so a
# NAMESPACE that still exports a removed or renamed function cannot fail it
# (#1288); document() reconciles NAMESPACE next. On a brand-new package this
# is also what writes the first wrappers file (#822).
compile_and_generate_wrappers <- function(pkg_path) {
  with_forced_wrapper_gen(
    tryCatch(
      pkgbuild::compile_dll(pkg_path, force = TRUE, quiet = FALSE),
      error = function(e) {
        cli::cli_abort(c(
          "Rust compile / wrapper generation failed",
          "i" = conditionMessage(e)
        ))
      }
    )
  )
  if (!wrappers_file_exists(pkg_path)) {
    cli::cli_abort(c(
      "The compile step completed but no {.path R/*-wrappers.R} was generated.",
      "i" = paste(
        "Expected the wrapper-gen pass to write it. Check that",
        "{.code #[miniextendr]} functions are reachable from {.file src/rust/lib.rs}."
      )
    ))
  }
  invisible(TRUE)
}

# Step 5: install the package via devtools, forcing the wrapper-gen pass.
#
# build = TRUE (the default): `R CMD build` runs bootstrap.R and the install
# proceeds from the tarball, the same path an end user's install takes. The
# wrappers regenerated in that copy are identical to the ones Step 3 wrote
# and Step 4 documented, so the namespace load-test sees a reconciled
# NAMESPACE.
#
# reload = FALSE: do NOT reload the freshly-installed package into the building
# session. The default (reload = TRUE) re-registers the package's namespace from
# its just-written installed image; on R >= 4.6 (libdeflate-compressed .rdb) a
# later pkgload unregister() of that namespace can fail with "internal error 1
# in R_decompress1 with libdeflate" / "lazy-load database is corrupt" (#1000).
# The build session never needs the package loaded, so skipping the reload is
# both harmless and the fix.
install_pkg <- function(pkg_path) {
  with_forced_wrapper_gen(
    tryCatch(
      devtools::install(pkg_path, upgrade = FALSE, quiet = FALSE, reload = FALSE),
      error = function(e) {
        cli::cli_abort(c(
          "Package installation failed",
          "i" = conditionMessage(e)
        ))
      }
    )
  )
}

#' Does the package's generated R wrapper file exist yet?
#'
#' The wrapper-gen pass writes `R/<pkg>-wrappers.R`; its presence is the signal
#' that the package has been built at least once. A fresh scaffold has the
#' Rust sources but no wrappers file.
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
  # Cargo.lock -- the shape offline source-replacement needs. So a cross-crate
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
    " " = "{.code unlink(\"inst/vendor.tar.xz\")}",
    "i" = "If your package has a local path-dependency sibling, vendoring also froze {.path src/rust/Cargo.toml} (and {.path Cargo.lock}) to resolve against {.path vendor/}. After the build, restore source shape:",
    " " = "{.code miniextendr_clean_vendor_leak()} (restores {.path Cargo.toml} from the {.path src/rust/.Cargo.toml.prefreeze} snapshot cargo-revendor left; cargo re-resolves {.path Cargo.lock} on the next build)",
    " " = "or {.code git checkout src/rust/Cargo.toml src/rust/Cargo.lock}"
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
