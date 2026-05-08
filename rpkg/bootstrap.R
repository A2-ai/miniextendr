# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE).
# Invoked by pkgbuild (devtools::build, r-lib/actions/check-r-package) in
# the source directory before R CMD build seals the tarball. Two jobs:
#   1. Produce inst/vendor.tar.xz via cargo-revendor, so the sealed
#      tarball ships with vendored sources for offline install.
#   2. Run ./configure so Makevars and other generated files exist
#      before R CMD build collects them.
#
# Why vendoring lives here, not in configure.ac's auto-vendor block:
# pkgbuild::build_setup_source uses callr::rscript against the original
# source dir (Config/build/copy-method defaults to "none"), so when
# configure runs it walks up to the workspace .git and SOURCE_IS_GIT=true
# — the .git-walk auto-vendor in configure skips. We do it here, before
# ./configure is called, so the tarball gets sealed with vendor.tar.xz
# inside.
#
# configure.ac's self-repair block still handles the complementary
# install-time case: an end user installs a tarball that arrived
# without inst/vendor.tar.xz (no .git in the extracted dir, so
# configure fires auto-vendor there).
#
# At install time bootstrap.R does NOT run (Config/build/bootstrap is
# pkgbuild-only). The bundled inst/vendor.tar.xz from step 1 is what
# configure detects to build offline.

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

if (.Platform$OS.type == "windows") {
  if (file.exists("configure.ucrt")) {
    system("sh configure.ucrt")
  } else if (file.exists("configure.win")) {
    system("sh configure.win")
  }
} else {
  system2("bash", "./configure")
}
