# System command execution with stderr/stdout capture

#' Execute system command with stderr/stdout capture
#'
#' Wrapper around system2() that saves stderr/stdout to timestamped log files
#' for later inspection when errors occur.
#'
#' @param command Command to execute
#' @param args Character vector of arguments
#' @param log_prefix Prefix for log file (e.g., "autoconf", "configure")
#' @param wd Working directory (uses current if NULL)
#' @param env Environment variables
#' @param log_dir Directory to save log files (defaults to tempdir())
#' @return List with status, output, and log_file path
#' @keywords internal
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

  # Run command and capture output
  result <- if (is.null(wd)) {
    system2(command, args, stdout = TRUE, stderr = TRUE, env = env)
  } else {
    withr::with_dir(wd, {
      system2(command, args, stdout = TRUE, stderr = TRUE, env = env)
    })
  }

  # Save output to log file
  writeLines(c(
    sprintf("Command: %s %s", command, paste(args, collapse = " ")),
    sprintf("Working directory: %s", wd %||% getwd()),
    sprintf("Timestamp: %s", Sys.time()),
    "",
    "=== Output ===",
    result
  ), log_file)

  # Extract status
  status <- attr(result, "status")

  list(
    status = status,
    output = result,
    log_file = log_file,
    success = is.null(status) || status == 0
  )
}

#' Check command result and throw informative error if failed
#'
#' @param result Result from run_with_logging()
#' @param context Description of what was being done
#' @keywords internal
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
      "i" = "Full output saved to: {result$log_file}"
    ))
  }
  invisible(result)
}
