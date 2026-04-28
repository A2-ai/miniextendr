# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE)
# Runs ./configure so that Makevars and other generated files exist
# before R CMD build creates the source tarball.

if (.Platform$OS.type == "windows") {
  if (file.exists("configure.ucrt")) {
    system("sh configure.ucrt")
  } else if (file.exists("configure.win")) {
    system("sh configure.win")
  }
} else {
  system2("bash", "./configure")
}
