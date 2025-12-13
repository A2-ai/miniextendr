# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE)
# Simply runs configure to sync vendor and generate build files

message("Running bootstrap.R...")

pkg_root <- getwd()
configure_script <- file.path(pkg_root, "configure")

if (file.exists(configure_script)) {
  message("Running ./configure...")
  result <- system2(configure_script, stdout = TRUE, stderr = TRUE)
  cat(result, sep = "\n")
  message("bootstrap.R completed successfully")
} else {
  message("configure script not found - running autoconf first")
  system2("autoconf", stdout = TRUE, stderr = TRUE)
  if (file.exists(configure_script)) {
    result <- system2(configure_script, stdout = TRUE, stderr = TRUE)
    cat(result, sep = "\n")
    message("bootstrap.R completed successfully")
  } else {
    stop("Failed to generate configure script")
  }
}
