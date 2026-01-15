# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE)
# Simply runs configure to sync vendor and generate build files

message("Running bootstrap.R...")

pkg_root <- getwd()
is_windows <- .Platform$OS.type == "windows"

# Find bash executable on Windows
find_bash <- function() {
  # Try common Rtools locations
  rtools_versions <- c("44", "43", "42", "40")
  for (ver in rtools_versions) {
    bash_path <- sprintf("C:/rtools%s/usr/bin/bash.exe", ver)
    if (file.exists(bash_path)) return(bash_path)
  }
  # Fallback: try to find bash in PATH
  bash_in_path <- Sys.which("bash")
  if (nzchar(bash_in_path)) return(bash_in_path)
  NULL
}

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

# Choose configure script based on platform
if (is_windows) {
  # On Windows, use configure.win via bash
  configure_script <- file.path(pkg_root, "configure.win")
  if (!file.exists(configure_script)) {
    stop("configure.win not found")
  }

  bash_exe <- find_bash()
  if (is.null(bash_exe)) {
    stop("Could not find bash.exe. Please install Rtools.")
  }

  message(sprintf("Running %s via bash...", basename(configure_script)))
  # Run configure.win through bash with login shell
  result <- system2(bash_exe,
                    args = c("-l", "-c", sprintf("cd '%s' && ./configure.win", pkg_root)),
                    stdout = "", stderr = "")
  if (result != 0) {
    stop(sprintf("configure.win failed with exit code %d", result))
  }
  message("bootstrap.R completed successfully")
} else {
  # Unix: run configure directly
  configure_script <- file.path(pkg_root, "configure")

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
}
