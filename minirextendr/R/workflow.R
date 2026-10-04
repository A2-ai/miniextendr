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
#' source tree with [pkgbuild::compile_dll()]: a libs-only `R CMD INSTALL` into
#' a throwaway library that links `src/<pkg>.so`, regenerates
#' `R/<pkg>-wrappers.R` and `src/rust/wasm_registry.rs` from it in place, and
#' never loads the namespace, so a stale `NAMESPACE` cannot fail it. Step 4's
#' `document()` reads the current wrappers; its own `pkgload::load_all()`
#' compile finds the library linked and the wrappers generated from it. Step 5
#' installs the package once, in place (`build = FALSE`), with `NAMESPACE`,
#' `man/` and the wrappers already in their final form; it reuses Step 3's
#' Cargo build. A brand-new package needs no special case:
#' Step 3 writes its first wrappers file the same way.
#'
#' `document()` loads the package from source through `pkgload::load_all()`
#' and leaves that development namespace registered in the session. Its export
#' table is the `NAMESPACE` as it stood *before* roxygen2 rewrote it, so a
#' `library()` call in the same session would attach an image missing every
#' export the build just added. The workflow unloads it after `document()`;
#' `library()` then loads the installed package (or, with `install = FALSE`,
#' `load_all()` reads the current `NAMESPACE`).
#'
#' @section Vendoring:
#' Nothing here vendors. Every step works on the source tree in place, where
#' path dependencies resolve as written; registry and Git dependencies resolve
#' over the network. An offline, CRAN-ready tarball comes from
#' [miniextendr_build_tarball()]. A
#' pre-existing `inst/vendor.tar.xz` latch is never deleted, but every step
#' then builds in offline tarball mode, so a warning is emitted up front and
#' `miniextendr_doctor()` / `miniextendr_clean_vendor_leak()` point at the fix.
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
  warn_release_leftovers(pkg_path)

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

#' Unload the development namespace roxygen2 left loaded
#'
#' Step 4 of [miniextendr_build()] leaves a development namespace behind:
#' roxygen2 documents through `pkgload::load_all()`, and pkgload fills the
#' namespace's export table from the NAMESPACE file as it is at load time,
#' i.e. before roxygen2 rewrote it. In the same session, `library(<pkg>)`
#' attaches an already-loaded namespace as is, so it would come up without
#' the exports this build added (a fresh package: with none at all). Unload
#' it so `library()` loads the installed image, or a later `load_all()` reads
#' the current NAMESPACE. Only a pkgload-registered namespace is touched; an
#' installed one the user loaded earlier is left alone (#1000).
#'
#' @param pkg_path Absolute path to the package root.
#' @return The package name, invisibly.
#' @keywords internal
unload_dev_namespace <- function(pkg_path) {
  pkg_name <- unname(mx_desc_get_field("Package", file = file.path(pkg_path, "DESCRIPTION")))
  if (pkgload::is_dev_package(pkg_name)) {
    pkgload::unload(pkg_name, quiet = TRUE)
  }
  invisible(pkg_name)
}

#' Compile the crate and regenerate the wrappers in the source tree
#'
#' Step 3 of [miniextendr_build()]. `pkgbuild::compile_dll()` runs a
#' libs-only `R CMD INSTALL --no-test-load` into a throwaway library:
#' configure + make in `src/`, which links `src/<pkg>.so` and runs the
#' Makevars wrapper-gen rule against it, writing `R/<pkg>-wrappers.R` and
#' `src/rust/wasm_registry.rs` in place. `force = TRUE` skips pkgbuild's own
#' source-vs-DLL mtime check; cargo decides what to rebuild, and make
#' regenerates the wrappers whenever the library was relinked or the wrappers
#' file is missing. Nothing here loads the package namespace, so a NAMESPACE
#' that still exports a removed or renamed function cannot fail it (#1288);
#' `document()` reconciles NAMESPACE next. On a brand-new package this is
#' also what writes the first wrappers file (#822).
#'
#' @param pkg_path Absolute path to the package root.
#' @return `TRUE` invisibly. Aborts when the compile fails or leaves no
#'   `R/*-wrappers.R` behind.
#' @keywords internal
compile_and_generate_wrappers <- function(pkg_path) {
  tryCatch(
    pkgbuild::compile_dll(pkg_path, force = TRUE, quiet = FALSE),
    error = function(e) {
      cli::cli_abort(c(
        "Rust compile / wrapper generation failed",
        "i" = conditionMessage(e)
      ))
    }
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

#' Install the package via devtools
#'
#' Step 5 of [miniextendr_build()].
#'
#' `build = FALSE`: install the source tree in place, as `R CMD INSTALL .`
#' does. The install reuses `rust-target/` from Step 3, so Cargo compiles
#' nothing, the library is not relinked and the wrappers stay as Steps 3 and
#' 4 left them. A built tarball would install from a temporary copy instead:
#' a cold Cargo build on every call (unless `CARGO_TARGET_DIR` points outside
#' the package), and a monorepo's local framework crates, found by
#' configure's walk up from the package directory, would resolve from git
#' there.
#'
#' `reload = FALSE`: do NOT reload the freshly-installed package into the
#' building session. The default (`reload = TRUE`) re-registers the package's
#' namespace from its just-written installed image; on R >= 4.6
#' (libdeflate-compressed `.rdb`) a later pkgload `unregister()` of that
#' namespace can fail with "internal error 1 in R_decompress1 with
#' libdeflate" / "lazy-load database is corrupt" (#1000). The build session
#' never needs the package loaded, so skipping the reload is both harmless
#' and the fix.
#'
#' @param pkg_path Absolute path to the package root.
#' @return The value of `devtools::install()`. Aborts when the install fails.
#' @keywords internal
install_pkg <- function(pkg_path) {
  tryCatch(
    devtools::install(pkg_path, build = FALSE, upgrade = FALSE, quiet = FALSE,
                      reload = FALSE),
    error = function(e) {
      cli::cli_abort(c(
        "Package installation failed",
        "i" = conditionMessage(e)
      ))
    }
  )
}

#' Warn about leftovers of an interrupted release build
#'
#' A vendor archive (`inst/vendor.tar.xz`) flips every build into offline
#' tarball mode, and a pre-freeze snapshot
#' (`src/rust/.Cargo.toml.prefreeze`) means `src/rust/Cargo.toml` is still
#' the frozen copy. Each one present gets a warning naming the fix; neither
#' is deleted here.
#'
#' @param pkg_path Absolute path to the package root.
#' @return `TRUE` invisibly; called for its warnings.
#' @keywords internal
warn_release_leftovers <- function(pkg_path) {
  if (fs::file_exists(fs::path(pkg_path, "src", "rust", ".Cargo.toml.prefreeze"))) {
    cli::cli_warn(c(
      "Pre-existing {.path src/rust/.Cargo.toml.prefreeze}: an earlier {.code cargo revendor --freeze} was never restored.",
      "i" = "Run {.code miniextendr_clean_vendor_leak()} first to restore {.path src/rust/Cargo.toml} from that snapshot."
    ))
  }
  if (fs::file_exists(fs::path(pkg_path, "inst", "vendor.tar.xz"))) {
    cli::cli_warn(c(
      "Pre-existing {.path inst/vendor.tar.xz} latch: this tree builds in tarball mode.",
      "i" = "Cargo resolves dependencies from the sealed vendor archive, not from \\
             the workspace, so edits to vendored crates are not compiled.",
      "i" = "Delete {.path inst/vendor.tar.xz} (or run {.code miniextendr_doctor()}, \\
             which detects the stale latch) to resume source-mode development."
    ))
  }
  invisible(TRUE)
}

#' Does the package's generated R wrapper file exist yet?
#'
#' The wrapper-gen pass writes `R/<pkg>-wrappers.R`; its presence is the signal
#' that the package has been built at least once. A fresh scaffold has the
#' Rust sources but no wrappers file.
#'
#' @param pkg_path Absolute path to the package root.
#' @return `TRUE` if any `R/*-wrappers.R` file exists.
#' @keywords internal
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
#' [miniextendr_build_tarball()] runs this before `R CMD build` and restores
#' the source tree afterwards; call it directly only to seal the archive by
#' hand. Nothing else vendors: day-to-day development (`R CMD INSTALL .`,
#' `devtools::install/test/load`, pak) and `bootstrap.R` leave
#' `inst/vendor.tar.xz` absent, and without the file cargo resolves deps over
#' the network.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param revendor_args Character vector of extra arguments passed to
#'   `cargo revendor`. The vendored tree is trimmed with `--strip-all` unless
#'   these include a `--strip-*` flag of their own; see [vendor_crates_io()].
#' @return Invisibly returns the path to the created tarball.
#' @export
miniextendr_vendor <- function(path = ".", revendor_args = character()) {
  with_project(path)
  cli::cli_h1("miniextendr vendor workflow")

  cargo_toml <- usethis::proj_path("src", "rust", "Cargo.toml")
  if (!fs::file_exists(cargo_toml)) {
    cli::cli_abort(c(
      "{.path src/rust/Cargo.toml} not found",
      "i" = "Run {.code miniextendr_configure()} first"
    ))
  }

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
  cli::cli_h2("Step 1: vendor all dependencies into inst/vendor.tar.xz")
  # cargo-revendor compresses the tree itself. Its --blank-md pass empties the
  # vendored .md files (CRAN notes their non-portable content) except those a
  # crate include_str!()s (#828), then recomputes .cargo-checksum.json: a .md
  # file truncated after vendoring fails cargo's offline checksum check.
  # Cargo.lock checksum lines are retained, as in the `just vendor` output.
  inst_dir <- usethis::proj_path("inst")
  tarball <- fs::path(inst_dir, "vendor.tar.xz")
  fs::dir_create(inst_dir)
  vendor_crates_io(tarball = tarball, revendor_args = revendor_args)

  size_mb <- round(as.numeric(fs::file_size(tarball)) / 1024 / 1024, 1)
  cli::cli_alert_success("Created {.path inst/vendor.tar.xz} ({size_mb} MB)")
  cli::cli_alert_info("Include this in your CRAN submission (R CMD build will bundle it)")
  cli::cli_alert_warning(c(
    "{.path inst/vendor.tar.xz} flips {.code ./configure} into offline tarball mode."
  ))
  cli::cli_bullets(c(
    "i" = "{.code miniextendr_build_tarball()} runs this step, builds the tarball and restores the source tree in one call.",
    "i" = "By hand: run {.code R CMD build .} to produce the release tarball, then delete {.path inst/vendor.tar.xz} to resume source-mode dev:",
    " " = "{.code unlink(\"inst/vendor.tar.xz\")}",
    "i" = "If your package has a local path-dependency sibling, vendoring also froze {.path src/rust/Cargo.toml} (and {.path Cargo.lock}) to resolve against {.path vendor/}. After the build, restore source shape:",
    " " = "{.code miniextendr_clean_vendor_leak()} (restores {.path Cargo.toml} from the {.path src/rust/.Cargo.toml.prefreeze} snapshot cargo-revendor left; cargo re-resolves {.path Cargo.lock} on the next build)",
    " " = "or {.code git checkout src/rust/Cargo.toml src/rust/Cargo.lock}"
  ))

  invisible(tarball)
}

#' Build a CRAN-ready package tarball
#'
#' The one workflow that vendors. It runs [miniextendr_build()] without the
#' install, so `R/<pkg>-wrappers.R`, `NAMESPACE` and `man/` match the Rust
#' sources, seals every crate dependency into `inst/vendor.tar.xz` with
#' [miniextendr_vendor()], and builds the tarball with [pkgbuild::build()].
#' That tarball installs offline and is the artifact `R CMD check` and CRAN
#' should see; a tarball from any other build frontend resolves its crates
#' over the network and is a development artifact.
#'
#' The source tree is restored on exit: the vendor archive is deleted, the
#' `src/rust/Cargo.toml` and `Cargo.lock` that `cargo revendor --freeze`
#' rewrote get their original content back, and a `vendor/` directory the
#' vendoring created is removed.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param dest_path Directory to write the tarball to. `NULL` writes it next
#'   to the package directory, as `R CMD build` does.
#' @param args Character vector of extra arguments passed to `R CMD build`.
#' @inheritParams miniextendr_vendor
#' @return The path to the built tarball, invisibly.
#' @seealso [miniextendr_check()] to build and check it in one step.
#' @export
miniextendr_build_tarball <- function(path = ".", dest_path = NULL, args = character(),
                                      revendor_args = character()) {
  with_project(path)
  pkg_path <- usethis::proj_get()
  cli::cli_h1("miniextendr tarball workflow")

  rust_manifest <- fs::path(pkg_path, "src", "rust", "Cargo.toml")
  rust_lock <- fs::path(pkg_path, "src", "rust", "Cargo.lock")
  rust_prefreeze <- fs::path(pkg_path, "src", "rust", ".Cargo.toml.prefreeze")
  vendor_tarball <- fs::path(pkg_path, "inst", "vendor.tar.xz")
  vendor_dir <- fs::path(pkg_path, "vendor")
  if (fs::file_exists(vendor_tarball) || fs::file_exists(rust_prefreeze)) {
    cli::cli_abort(c(
      "The source tree still carries an earlier release build's vendoring.",
      "i" = "Run {.code miniextendr_clean_vendor_leak()} first, then build again."
    ))
  }
  snap_manifest <- readLines(rust_manifest, warn = FALSE)
  snap_lock <- if (fs::file_exists(rust_lock)) readLines(rust_lock, warn = FALSE)
  vendor_preexisting <- fs::dir_exists(vendor_dir)
  on.exit({
    writeLines(snap_manifest, rust_manifest)
    if (!is.null(snap_lock)) writeLines(snap_lock, rust_lock)
    if (fs::file_exists(rust_prefreeze)) fs::file_delete(rust_prefreeze)
    if (fs::file_exists(vendor_tarball)) fs::file_delete(vendor_tarball)
    if (!vendor_preexisting && fs::dir_exists(vendor_dir)) fs::dir_delete(vendor_dir)
  }, add = TRUE)

  cli::cli_h2("Step 1: wrappers, NAMESPACE and man/")
  miniextendr_build(install = FALSE)

  cli::cli_h2("Step 2: vendor")
  miniextendr_vendor(revendor_args = revendor_args)

  cli::cli_h2("Step 3: R CMD build")
  tarball <- pkgbuild::build(pkg_path, dest_path = dest_path, args = args, quiet = FALSE)
  cli::cli_alert_success("Built {.path {tarball}}")
  invisible(tarball)
}

#' Run R CMD check on a miniextendr package
#'
#' Builds the CRAN-ready tarball with [miniextendr_build_tarball()] and runs
#' `R CMD check` on it. Only that tarball carries vendored dependencies, so it
#' is the one whose check means what CRAN's does.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param args Character vector of extra arguments passed to `R CMD check`.
#'   Defaults to `c("--as-cran", "--no-manual")`.
#' @param error_on Severity level to error on. One of `"error"`, `"warning"`,
#'   or `"note"`. Passed to [rcmdcheck::rcmdcheck()].
#' @param build_args Character vector of extra arguments passed to `R CMD build`.
#' @inheritParams miniextendr_vendor
#' @return The [rcmdcheck::rcmdcheck()] result object, invisibly.
#' @seealso [miniextendr_check_static()] for a fast no-compile variant suitable
#'   for un-vendored packages.
#' @export
miniextendr_check <- function(path = ".",
                               args = c("--as-cran", "--no-manual"),
                               error_on = "warning",
                               build_args = character(),
                               revendor_args = character()) {
  with_project(path)
  if (!requireNamespace("rcmdcheck", quietly = TRUE)) {
    cli::cli_abort(c(
      "rcmdcheck is required for miniextendr_check()",
      "i" = 'Install it with: install.packages("rcmdcheck")'
    ))
  }

  cli::cli_h1("miniextendr check workflow")

  cli::cli_h2("Step 1: build the tarball")
  tarball <- miniextendr_build_tarball(dest_path = withr::local_tempdir(), args = build_args,
                                       revendor_args = revendor_args)

  cli::cli_h2("Step 2: R CMD check")
  cli::cli_alert("Running rcmdcheck with args: {.val {args}}")

  result <- rcmdcheck::rcmdcheck(tarball, args = args, error_on = error_on)

  invisible(result)
}
