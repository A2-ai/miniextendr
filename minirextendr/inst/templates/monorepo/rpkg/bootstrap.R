# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE)
# Runs ./configure so that Makevars and other generated files exist
# before R CMD build creates the source tarball.
#
# Install-mode detection is automatic: if inst/vendor.tar.xz exists
# (created by `just vendor` from the workspace root, or
# `minirextendr::miniextendr_vendor()` from this package, before
# `R CMD build`), configure builds in tarball/offline mode. Otherwise,
# source/network mode is used.

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
