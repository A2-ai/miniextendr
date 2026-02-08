# System command execution with stderr/stdout capture

#' Execute system command with stderr/stdout capture
#'
#' Wrapper around processx::run() that saves stderr/stdout to timestamped log
#' files for later inspection when errors occur.
#'
#' @param command Command to execute
#' @param args Character vector of arguments
#' @param log_prefix Prefix for log file (e.g., "autoconf", "configure")
#' @param wd Working directory (uses current if NULL)
#' @param env Environment variables
#' @param log_dir Directory to save log files (defaults to tempdir())
#' @return List with status, output, and log_file path
#' @export
run_with_logging <- function(command, args = character(),
                              log_prefix = "command",
                              wd = NULL,
                              env = character(),
                              log_dir = tempdir()) {
  # Create timestamp for log file
  timestamp <- format(Sys.time(), "%Y-%m-%d-%H-%M-%S")
  log_file <- file.path(log_dir, sprintf("%s-%s.log", timestamp, log_prefix))

  # Ensure log directory exists
  if (!dir.exists(log_dir)) {
    dir.create(log_dir, recursive = TRUE, showWarnings = FALSE)
  }

  # Convert named vector to "NAME=value" format
  if (length(env) > 0 && !is.null(names(env))) {
    env <- paste0(names(env), "=", env)
  }

  # Build processx env argument: processx expects a named character vector
  # where names are variable names and values are variable values.
  # Our `env` is in "NAME=value" format. Convert for processx, and merge
  # with the current environment so child processes inherit it.
  px_env <- NULL
  if (length(env) > 0) {
    # Parse "NAME=value" strings into a named vector
    eq_pos <- regexpr("=", env, fixed = TRUE)
    env_names <- substr(env, 1, eq_pos - 1)
    env_values <- substr(env, eq_pos + 1, nchar(env))
    extra_env <- stats::setNames(env_values, env_names)

    # Merge with current environment (extra vars override)
    px_env <- c(Sys.getenv(), extra_env)
  }

  # Run command via processx
  px_result <- processx::run(
    command,
    args = args,
    wd = wd,
    env = px_env,
    error_on_status = FALSE
  )

  # Combine stdout and stderr into a single character vector (like system2)
  combined <- c(px_result$stdout, px_result$stderr)
  output <- strsplit(combined, "\n", fixed = TRUE)
  output <- unlist(output, use.names = FALSE)
  # Remove trailing empty string from final newline
  if (length(output) > 0 && output[length(output)] == "") {
    output <- output[-length(output)]
  }

  # Save output to log file
  writeLines(c(
    sprintf("Command: %s %s", command, paste(args, collapse = " ")),
    sprintf("Working directory: %s", wd %||% getwd()),
    sprintf("Timestamp: %s", Sys.time()),
    "",
    "=== Output ===",
    output
  ), log_file)

  status <- px_result$status

  list(
    status = if (status == 0) NULL else status,
    output = output,
    log_file = log_file,
    success = status == 0
  )
}

#' Check command result and throw informative error if failed
#'
#' @param result Result from run_with_logging()
#' @param context Description of what was being done
#' @export
check_result <- function(result, context) {
  if (!result$success) {
    cli::cli_alert_danger("{context} failed (status: {result$status %||% 'unknown'})")
    cli::cli_alert_info("Log saved to: {.path {result$log_file}}")

    # Show last 20 lines of output
    tail_output <- utils::tail(result$output, 20)
    if (length(tail_output) > 0) {
      cli::cli_alert_info("Last 20 lines of output:")
      cli::cli_verbatim(paste(tail_output, collapse = "\n"))
    }

    abort(c(
      paste0(context, " failed"),
      "i" = paste0("Full output saved to: ", result$log_file)
    ))
  }
  invisible(result)
}

#' Run a command and capture output
#'
#' Internal helper that runs a command via processx and returns the combined
#' stdout+stderr as a character vector, with the exit status as an attribute
#' (mimicking system2's return convention).
#'
#' @param command Command to execute
#' @param args Character vector of arguments
#' @param wd Working directory (NULL for current)
#' @return Character vector of output lines. The exit status is stored as
#'   the \code{"status"} attribute (NULL when 0, matching system2 convention).
#' @noRd
run_command <- function(command, args = character(), wd = NULL) {
  px_result <- processx::run(
    command,
    args = args,
    wd = wd,
    error_on_status = FALSE
  )

  # Combine stdout and stderr into a character vector (like system2)
  combined <- c(px_result$stdout, px_result$stderr)
  output <- strsplit(combined, "\n", fixed = TRUE)
  output <- unlist(output, use.names = FALSE)
  # Remove trailing empty string from final newline
  if (length(output) > 0 && output[length(output)] == "") {
    output <- output[-length(output)]
  }

  # system2 returns NULL status on success, numeric on failure
  if (px_result$status != 0) {
    attr(output, "status") <- px_result$status
  }

  output
}
