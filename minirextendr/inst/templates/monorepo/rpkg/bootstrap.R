# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE)
# Mimics R's internal configure handling from src/library/tools/R/install.R

message("Running bootstrap.R...")

pkg_root <- getwd()

# Get R CMD config value for an environment variable
get_r_config <- function(var) {
  if (.Platform$OS.type == "windows") {
    system2(file.path(R.home("bin"), "Rcmd.exe"),
            c("config", var), stdout = TRUE)
  } else {
    system2(file.path(R.home("bin"), "R"),
            c("CMD", "config", var), stdout = TRUE)
  }
}

# Run configure with proper environment setup (matching R's install.R)
run_configure <- function() {
  # Environment variables to set from R CMD config
  ev <- c("CC", "CFLAGS", "CXX", "CXXFLAGS", "CPPFLAGS",
          "LDFLAGS", "FC", "FCFLAGS")

  # Skip any which are already set
  ev <- ev[!nzchar(Sys.getenv(ev))]

  # Get values from R CMD config
  ev_values <- sapply(ev, get_r_config, USE.NAMES = TRUE)

  # Filter out empty values (possible for CXX on some systems)
  ev_values <- ev_values[nzchar(ev_values)]

  # Set environment variables
  if (length(ev_values) > 0) {
    do.call(Sys.setenv, as.list(ev_values))
  }

  on.exit({
    # Unset environment variables we set (safe since we skipped already-set ones)
    if (length(ev_values) > 0) {
      Sys.unsetenv(names(ev_values))
    }
  })

  if (.Platform$OS.type == "windows") {
    # Windows: try configure.ucrt, then configure.win
    if (file.exists("configure.ucrt")) {
      f <- "configure.ucrt"
    } else if (file.exists("configure.win")) {
      f <- "configure.win"
    } else if (file.exists("configure")) {
      message("\n",
              "   **********************************************\n",
              "   WARNING: this package has a configure script\n",
              "         It probably needs manual configuration\n",
              "   **********************************************\n\n")
      return(invisible(FALSE))
    } else {
      stop("No configure script found")
    }

    message(sprintf("Running: sh %s", f))
    res <- system(paste("sh", f))
    if (res != 0) {
      stop(sprintf("configuration failed (exit code %d)", res))
    }

  } else {
    # Unix: run ./configure
    configure_script <- file.path(pkg_root, "configure")

    # Set NOT_CRAN=true for dev builds if not already set
    # This ensures configure runs in dev mode during devtools workflows
    if (!nzchar(Sys.getenv("NOT_CRAN"))) {
      Sys.setenv(NOT_CRAN = "true")
      message("Setting NOT_CRAN=true for dev build")
    }
    # Prevent accidental inheritance of release-prep mode
    if (!nzchar(Sys.getenv("PREPARE_CRAN"))) {
      Sys.setenv(PREPARE_CRAN = "false")
    }

    if (!file.exists(configure_script)) {
      # Try running autoconf first
      message("configure script not found - running autoconf first")
      res <- system2("autoconf", stdout = "", stderr = "")
      if (res != 0) {
        stop("autoconf failed")
      }
    }

    if (!file.exists(configure_script)) {
      stop("Failed to generate configure script")
    }

    # Check if configure is executable
    if (!file_test("-x", configure_script)) {
      stop("'configure' exists but is not executable -- see the 'R Installation and Administration Manual'")
    }

    # Build command with env vars (matching R's approach)
    # _R_SHLIB_BUILD_OBJECTS_SYMBOL_TABLES_=false in case configure calls SHLIB
    ev_args <- paste0(names(ev_values), "=", shQuote(ev_values))
    cmd <- paste(c("_R_SHLIB_BUILD_OBJECTS_SYMBOL_TABLES_=false",
                   ev_args,
                   "./configure"),
                 collapse = " ")

    message(sprintf("Running: %s", cmd))
    res <- system(cmd)
    if (res != 0) {
      stop(sprintf("configuration failed (exit code %d)", res))
    }
  }

  invisible(TRUE)
}

run_configure()
message("bootstrap.R completed successfully")
