# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE).
# Invoked by pkgbuild (devtools::build, r-lib/actions/check-r-package) in
# the source directory before R CMD build seals the tarball. Two jobs:
#   1. Run ./configure so Makevars and .cargo/config.toml exist before
#      R CMD build collects them.
#   2. Produce inst/vendor.tar.xz via cargo-revendor, so the sealed
#      tarball ships with vendored sources for offline install. Without
#      cargo-revendor, stage only the path dependencies (see below).
#
# Order matters: configure runs FIRST. In a dev/monorepo checkout (the
# source dir still has a workspace .git ancestor — pkgbuild's
# copy-method defaults to "none"), configure detects the monorepo and
# writes a [patch."<git-url>"] block into src/rust/.cargo/config.toml
# redirecting miniextendr-{api,lint,macros} to the local workspace.
# cargo-revendor then resolves the dependency graph against THOSE local
# sources, so a cross-crate feature/dep rename touching both a framework
# crate and rpkg resolves against the PR's checkout instead of git@main
# (#883). cargo-revendor stamps the canonical `source = "git+<url>#<sha>"`
# attribution back into Cargo.lock itself (the offline source-replacement
# shape), so there is no longer any bare-git pre-resolution dance here.
# src/rust/.cargo is .Rbuildignore'd, so the dev [patch] config is never
# sealed into the tarball — install-time configure rewrites it to the
# [source] replacement form.
#
# Vendoring lives here because bootstrap.R is part of tarball production.
# configure.ac never creates inst/vendor.tar.xz; it only consumes an existing
# tarball to select offline mode. A package artifact built without the vendor
# tarball therefore remains in source mode and is not CRAN-ready.
#
# At install time bootstrap.R does NOT run (Config/build/bootstrap is
# pkgbuild-only). The bundled inst/vendor.tar.xz from step 2 is what
# configure detects to build offline.

# MINIEXTENDR_BOOTSTRAP=1 tells configure's leaked-tarball guard (#1029) that
# this ./configure was invoked by bootstrap, not directly. The `just vendor`
# step that precedes devtools::build()/check() leaves inst/vendor.tar.xz in the
# (git-tracked) source dir on purpose; without this signal configure would
# (correctly, for a *direct* invocation) treat that tarball-in-a-git-tree as a
# leak and abort. We pass it inline to the configure call only, so it never
# leaks into the cargo-revendor step or the surrounding R session.
if (.Platform$OS.type == "windows") {
  if (file.exists("configure.ucrt")) {
    system2("sh", "configure.ucrt", env = "MINIEXTENDR_BOOTSTRAP=1")
  } else if (file.exists("configure.win")) {
    system2("sh", "configure.win", env = "MINIEXTENDR_BOOTSTRAP=1")
  }
} else {
  system2("bash", "./configure", env = "MINIEXTENDR_BOOTSTRAP=1")
}

# A path dependency outside the package (e.g. a core crate at
# `path = "../../../my-core"`) is NOT source-replaceable: R CMD build seals only
# the package directory, and a git/staged install (remotes, pak) copies it out
# of its workspace, so the sibling must travel inside the package. With
# cargo-revendor on PATH the whole graph goes into inst/vendor.tar.xz. Without
# it, tools/dev-bootstrap.R stages only the path dependencies under
# src/rust/vendor via `cargo package`, and cleanup swaps in a manifest pointing
# there; registry and git dependencies still resolve over the network. A package
# whose path dependencies all live inside it has nothing to stage.
bootstrap_mode <- match.arg(Sys.getenv("MINIEXTENDR_BOOTSTRAP_MODE", "dist"), c("dist", "dev"))
source("tools/dev-bootstrap.R", local = TRUE)
if (bootstrap_mode == "dev") {
  prepare_dev_bootstrap()
} else {
  clear_dev_bootstrap()
}

if (bootstrap_mode == "dist" && !file.exists("inst/vendor.tar.xz")) {
  if (!nzchar(Sys.which("cargo-revendor"))) {
    if (prepare_dev_bootstrap(mode = "dist")) {
      message(
        "bootstrap.R: cargo-revendor not on PATH; staged path dependencies under ",
        "src/rust/vendor. Registry and git dependencies resolve over the network at ",
        "install, so this artifact is not CRAN-ready (install cargo-revendor for an ",
        "offline inst/vendor.tar.xz)."
      )
    } else {
      message(
        "bootstrap.R: cargo-revendor not on PATH and no path dependency outside the ",
        "package; building from source (cargo fetches dependencies over the network)."
      )
    }
  } else {
    message("bootstrap.R: generating inst/vendor.tar.xz via cargo-revendor")
    dir.create("inst", showWarnings = FALSE)
    # --freeze rewrites local path-dependency siblings (e.g. a core crate at
    # `path = "../../../my-core"`) to point at vendor/, so the sealed tarball is
    # self-contained: a path dep is NOT source-replaceable, so without --freeze the
    # shipped Cargo.toml would still reference a sibling that does not travel inside
    # the tarball and the offline install would fail to resolve it. Inert for the
    # common git-only package (no local path deps to rewrite, no committed patches),
    # where it only normalises Cargo.lock. cargo-revendor auto-detects the source
    # root from `cargo metadata`, so no --source-root is needed here.
    #
    # Both rewrites land on files the wrapper provenance record fingerprints
    # (#1512), while the generated wrappers do not depend on where dependencies
    # are resolved from. A record that is current before the freeze is therefore
    # re-written afterwards, so the sealed tarball keeps the pre-shipped-wrapper
    # fast path (#1022); a stale or absent record stays as it is.
    source("tools/wrapper-freshness.R", local = TRUE)
    wrappers <- file.path("R", paste0(read.dcf("DESCRIPTION", fields = "Package")[[1L]],
                                      "-wrappers.R"))
    preserve_wrapper_record(".", wrappers, function() {
      status <- system2("cargo", c(
        "revendor",
        "--manifest-path", "src/rust/Cargo.toml",
        "--output", "vendor",
        "--freeze",
        "--compress", "inst/vendor.tar.xz",
        "--blank-md",
        "--source-marker",
        "--force",
        "-v"
      ))
      if (status != 0) {
        stop("bootstrap.R: cargo revendor failed (exit ", status, ")", call. = FALSE)
      }
    })
  }
}
