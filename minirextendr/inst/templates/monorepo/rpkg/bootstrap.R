# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE).
# Invoked by pkgbuild (devtools::build/install, remotes::install_git, pak,
# rcmdcheck, r-lib/actions) in the source directory before R CMD build seals
# the tarball. Two jobs:
#   1. Run ./configure so Makevars and other generated files exist before
#      R CMD build collects them.
#   2. If no inst/vendor.tar.xz is present yet, vendor one with cargo-revendor
#      so the sealed tarball ships self-contained for offline install.
#
# Install-mode detection is automatic: if inst/vendor.tar.xz exists (created by
# minirextendr::miniextendr_vendor() or by the auto-vendor block below),
# configure builds in tarball/offline mode. Otherwise source/network mode is used.

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

# Auto-vendor fallback. minirextendr::miniextendr_vendor() normally seals
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
if (!file.exists("inst/vendor.tar.xz")) {
  cargo_revendor <- Sys.which("cargo-revendor")
  if (!nzchar(cargo_revendor)) {
    stop(
      "bootstrap.R: cargo-revendor not on PATH. Install with:\n",
      "  cargo install --git https://github.com/A2-AI/miniextendr ",
      "cargo-revendor --locked",
      call. = FALSE
    )
  }

  message("bootstrap.R: generating inst/vendor.tar.xz via cargo-revendor")
  dir.create("inst", showWarnings = FALSE)
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
}
