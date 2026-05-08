# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE)
# Runs ./configure so that Makevars and other generated files exist
# before R CMD build creates the source tarball.
#
# This file is invoked by pkgbuild (devtools::build, r-lib/actions/check-r-package)
# in the build-staging directory before sealing the source tarball.
#
# Vendoring (production of inst/vendor.tar.xz) is handled by configure.ac's
# auto-vendor block — it fires here (staging dir, no .git ancestor) so the
# sealed tarball ships with vendor.tar.xz inside, and again at install time
# if the tarball arrived without one. See rpkg/configure.ac for the
# self-repair contract.

if (.Platform$OS.type == "windows") {
  if (file.exists("configure.ucrt")) {
    system("sh configure.ucrt")
  } else if (file.exists("configure.win")) {
    system("sh configure.win")
  }
} else {
  system2("bash", "./configure")
}
