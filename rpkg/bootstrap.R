# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE)
# Simply runs configure to sync vendor and generate build files

message("Running bootstrap.R...")

pkg_root <- getwd()
configure_script <- file.path(pkg_root, "configure")

# Helper to run a command and check exit status
run_cmd <- function(cmd, args = character()) {
  message(sprintf("Running: %s %s", cmd, paste(args, collapse = " ")))
  result <- system2(cmd, args, stdout = "", stderr = "")
  if (result != 0) {
    stop(sprintf("Command failed with exit code %d: %s %s",
                 result, cmd, paste(args, collapse = " ")))
  }
  invisible(result)
}

if (file.exists(configure_script)) {
  message("Running ./configure...")
  run_cmd(configure_script)
  message("bootstrap.R completed successfully")
} else {
  message("configure script not found - running autoconf first")
  run_cmd("autoconf")
  if (file.exists(configure_script)) {
    run_cmd(configure_script)
    message("bootstrap.R completed successfully")
  } else {
    stop("Failed to generate configure script")
  }
}
