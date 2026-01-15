# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE)
# Simply runs configure to sync vendor and generate build files

message("Running bootstrap.R...")

pkg_root <- getwd()
is_windows <- .Platform$OS.type == "windows"

# Find bash executable on Windows (relies on Rtools being in PATH)
find_bash <- function() {
  bash_in_path <- Sys.which("bash")
  if (nzchar(bash_in_path)) return(bash_in_path)
  NULL
}

# Helper to run a command and check exit status
run_cmd <- function(cmd, args = character()) {
  message(sprintf("Running: %s %s", cmd, paste(args, collapse = " ")))
  # Use stdout/stderr = TRUE to inherit from parent (visible in logs)
  result <- system2(cmd, args, stdout = TRUE, stderr = TRUE)
  exit_status <- attr(result, "status")
  if (!is.null(exit_status) && exit_status != 0) {
    # Print captured output for debugging
    if (length(result) > 0) {
      message("Command output:")
      message(paste(result, collapse = "\n"))
    }
    stop(sprintf("Command failed with exit code %d: %s %s",
                 exit_status, cmd, paste(args, collapse = " ")))
  }
  invisible(0)
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
  message(sprintf("Package root: %s", pkg_root))
  message(sprintf("Bash executable: %s", bash_exe))

  # Run configure.win through bash with login shell
  # Capture output to show in error messages
  cmd_str <- sprintf("cd '%s' && ./configure.win", pkg_root)
  message(sprintf("Bash command: %s", cmd_str))

  output <- system2(bash_exe,
                    args = c("-l", "-c", cmd_str),
                    stdout = TRUE, stderr = TRUE)
  exit_status <- attr(output, "status")

  # Always print output for debugging
  if (length(output) > 0) {
    message("configure.win output:")
    message(paste(output, collapse = "\n"))
  }

  if (!is.null(exit_status) && exit_status != 0) {
    stop(sprintf("configure.win failed with exit code %d", exit_status))
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
