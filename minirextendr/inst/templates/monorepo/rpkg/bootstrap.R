# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE).
# Invoked by pkgbuild (devtools::build/install, remotes::install_git, pak,
# rcmdcheck, r-lib/actions) in the source directory before R CMD build seals
# the tarball. Two jobs:
#   1. Run ./configure so Makevars and other generated files exist before
#      R CMD build collects them.
#   2. If no inst/vendor.tar.xz is present yet, vendor one with cargo-revendor
#      so the sealed tarball ships self-contained for offline install. Without
#      cargo-revendor, stage only the path dependencies (see below).
#
# Install-mode detection is automatic: minirextendr_vendor() or bootstrap.R
# creates inst/vendor.tar.xz while producing a package tarball; configure only
# consumes it. Otherwise source/network mode is used.

# MINIEXTENDR_BOOTSTRAP=1 tells configure's leaked-tarball guard (#1029) that
# this ./configure was invoked by bootstrap, not directly, so a deliberate
# tarball-in-a-git-tree (produced by the vendor step that precedes
# devtools::build()/check()) is not mistaken for a leak.
if (.Platform$OS.type == "windows") {
  if (file.exists("configure.ucrt")) {
    system2("sh", "configure.ucrt", env = "MINIEXTENDR_BOOTSTRAP=1")
  } else if (file.exists("configure.win")) {
    system2("sh", "configure.win", env = "MINIEXTENDR_BOOTSTRAP=1")
  }
} else {
  system2("bash", "./configure", env = "MINIEXTENDR_BOOTSTRAP=1")
}

# Tarball-production vendoring. minirextendr::miniextendr_vendor() normally seals
# inst/vendor.tar.xz before the build and this block short-circuits via the
# file.exists guard. But git-based / staged installs (remotes::install_git,
# devtools::install, pak, CRAN) never run it and copy the package
# out of the workspace before building — which strands any local
# path-dependency sibling (a core crate at `path = "../../../my-core"`), since
# a path dep is NOT source-replaceable. We vendor here instead, while the
# sibling is still reachable in the source/clone tree. --freeze rewrites the
# sibling to vendor/ so the sealed tarball is self-contained; deps declared
# `git =` stay git and resolve offline via source replacement. Inert for a
# git-only package with no path sibling to rewrite.
# Without cargo-revendor, tools/dev-bootstrap.R stages only the path
# dependencies under src/rust/vendor via `cargo package`, and cleanup swaps in
# a manifest pointing there; registry and git dependencies still resolve over
# the network. A package whose path dependencies all live inside it has
# nothing to stage.
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
    # --freeze rewrites Cargo.toml and normalises Cargo.lock, both fingerprinted
    # by the wrapper provenance record (#1512), while the generated wrappers do
    # not depend on where dependencies are resolved from. A record that is
    # current before the freeze is therefore re-written afterwards, so the sealed
    # tarball keeps the pre-shipped-wrapper fast path (#1022); a stale or absent
    # record stays as it is.
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
