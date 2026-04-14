# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE)
# Runs ./configure so that Makevars and other generated files exist
# before R CMD build creates the source tarball.

# Default to dev mode if NOT_CRAN is not set.
# bootstrap.R only runs during R CMD build (Config/build/bootstrap: TRUE),
# never on CRAN install, so defaulting to true is safe.
if (!nzchar(Sys.getenv("NOT_CRAN"))) {
  Sys.setenv(NOT_CRAN = "true")
}
# Prevent accidental inheritance of release-prep mode.
if (!nzchar(Sys.getenv("PREPARE_CRAN"))) {
  Sys.setenv(PREPARE_CRAN = "false")
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
