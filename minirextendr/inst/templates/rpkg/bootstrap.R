# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE).
# Invoked by pkgbuild (devtools::build, r-lib/actions/check-r-package) in
# the source directory before R CMD build seals the tarball. Two jobs:
#   1. Run ./configure so Makevars and .cargo/config.toml exist before
#      R CMD build collects them.
#   2. Produce inst/vendor.tar.xz via cargo-revendor, so the sealed
#      tarball ships with vendored sources for offline install.
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
# Why vendoring lives here, not in configure.ac's auto-vendor block:
# when configure runs in the source dir it walks up to the workspace .git
# and SOURCE_IS_GIT=true — the .git-walk auto-vendor in configure skips.
# We do it here so the tarball gets sealed with vendor.tar.xz inside.
#
# configure.ac's self-repair block still handles the complementary
# install-time case: an end user installs a tarball that arrived
# without inst/vendor.tar.xz (no .git in the extracted dir, so
# configure fires auto-vendor there). That path has no [patch] override,
# so cargo resolves the framework crates straight from the git URL and
# cargo-revendor leaves the natural git source in place (nothing to stamp).
#
# At install time bootstrap.R does NOT run (Config/build/bootstrap is
# pkgbuild-only). The bundled inst/vendor.tar.xz from step 2 is what
# configure detects to build offline.

if (.Platform$OS.type == "windows") {
  if (file.exists("configure.ucrt")) {
    system("sh configure.ucrt")
  } else if (file.exists("configure.win")) {
    system("sh configure.win")
  }
} else {
  system2("bash", "./configure")
}

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
