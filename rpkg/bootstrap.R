# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE)
# Runs ./configure so that Makevars and other generated files exist
# before R CMD build creates the source tarball.
#
# Install-mode detection is automatic: if inst/vendor.tar.xz exists
# (created by `minirextendr::miniextendr_vendor()` before `R CMD build`
# when preparing a CRAN submission), configure builds in tarball/offline
# mode. Otherwise, source/network mode is used.

if (.Platform$OS.type == "windows") {
  if (file.exists("configure.ucrt")) {
    system("sh configure.ucrt")
  } else if (file.exists("configure.win")) {
    system("sh configure.win")
  }
} else {
  system2("bash", "./configure")
}
