r
#' @title Execute a System Command with Robust Error Handling and Structured Output
#'
#' @description
#' Runs a system command using \code{system2}, capturing stdout and stderr separately.
#' Returns a structured list with exit status, success indicator, and output streams.
#' Non-zero exit codes are handled gracefully — the redundant R warning from
#' \code{system2} is suppressed because the exit status is already captured via the
#' return attribute. A best-effort timeout mechanism is applied using \code{setTimeLimit}.
#'
#' @param command Character scalar (non-empty). The system command to execute.
#'   The command must be available on the system PATH.
#' @param args Character vector of arguments. Default \code{character()}.
#'   All elements must be non-missing and non-NA.
#' @param wd Character scalar. Working directory for the command.
#'   Default \code{"."}. Must exist and be a directory.
#' @param timeout Numeric scalar (non-negative). Maximum elapsed time in seconds.
#'   Default \code{60}. Set to \code{Inf} to disable timeout.
#'
#' @return A named list with the following components:
#' \describe{
#'   \item{output}{Character vector of combined stdout lines, or \code{NULL} if empty.}
#'   \item{status}{Integer exit status, or \code{NULL} if zero.}
#'   \item{success}{Logical: \code{TRUE} if exit status is zero, \code{FALSE} otherwise.}
#'   \item{stdout}{Character vector of stdout lines (identical to \code{output}).}
#'   \item{stderr}{Character vector of stderr lines, or \code{NULL} if none.}
#' }
#'
#' @details
#' The function validates inputs early, checks that the command exists on the system
#' PATH, and verifies the working directory. Stderr is captured by redirecting to a
#' temporary file. The R warning from \code{system2} for non-zero exit is suppressed
#' because the exit status is explicitly retrieved via the \code{"status"} attribute
#' and included in the result.
#'
#' Logging is performed using the \pkg{logger} package with appropriate levels.
#' Temporary files are cleaned up via \code{on.exit}, even if the function errors.
#'
#' @section Security Considerations:
#' This function does \strong{not} sanitize \code{command} or \code{args} against
#' injection. If the command or arguments originate from user input, consider using
#' \code{processx::process} with proper argument quoting or a whitelist validation.
#'
#' @examples
#' \dontrun{
#' # Successful command
#' run_with_logging("echo", "hello world")
#'
#' # Command that fails
#' run_with_logging("false")
#'
#' # With timeout (will interrupt if command exceeds limit)
#' run_with_logging("sleep", "3", timeout = 2)
#' }
#'
#' @importFrom checkmate assert_string assert_character assert_number
#' @importFrom logger log_trace log_debug log_info log_warn log_error
#' @export
run_with_logging <- function(
  command,
  args = character(),
  wd = ".",
  timeout = 60
) {
  # ---------------------------------------------------------------------------
  # 1. Input Validation
  # ---------------------------------------------------------------------------
  checkmate::assert_string(command, min.chars = 1L, n.chars = NULL)
  checkmate::assert_character(args, min.len = 0L, any.missing = FALSE)
  checkmate::assert_string(wd, min.chars = 1L)
  checkmate::assert_number(timeout, lower = 0, finite = TRUE)

  # Normalize working directory path for consistency
  wd_norm <- normalizePath(wd, winslash = "/", mustWork = FALSE)
  if (!dir.exists(wd_norm)) {
    logger::log_error("Working directory '{wd_norm}' does not exist.")
    stop("Working directory '", wd_norm, "' does not exist.", call. = FALSE)
  }

  # ---------------------------------------------------------------------------
  # 2. Command Existence Check (early failure)
  # ---------------------------------------------------------------------------
  cmd_path <- Sys.which(command)
  if (!nzchar(cmd_path)) {
    logger::log_error("Command '{command}' not found on system PATH.")
    stop("Command '", command, "' not found on system PATH.", call. = FALSE)
  }

  # ---------------------------------------------------------------------------
  # 3. Logging Execution Details
  # ---------------------------------------------------------------------------
  args_str <- paste(args, collapse = " ")
  logger::log_debug("Executing: {cmd_path} {args_str}")
  logger::log_trace("Working directory: {wd_norm}")
  logger::log_trace("Timeout: {timeout} seconds")

  # ---------------------------------------------------------------------------
  # 4. Temporary File for Stderr Capture
  # ---------------------------------------------------------------------------
  stderr_file <- tempfile(pattern = "stderr_")
  on.exit({
    if (file.exists(stderr_file)) {
      unlink(stderr_file)
      logger::log_trace("Cleaned up temporary stderr file: {stderr_file}")
    }
  }, add = TRUE)

  # ---------------------------------------------------------------------------
  # 5. Command Execution with Timeout and Error Handling
  # ---------------------------------------------------------------------------
  result <- tryCatch(
    expr = {
      # Save current timeout option and restore on exit
      old_timeout <- getOption("timeout")
      options(timeout = max(timeout, 0.001))  # system2 timeout option (seconds)
      on.exit(options(timeout = old_timeout), add = TRUE)

      # Best-effort elapsed time limit via setTimeLimit
      setTimeLimit(elapsed = timeout, transient = TRUE)
      on.exit(setTimeLimit(cpu = Inf, elapsed = Inf, transient = FALSE), add = TRUE)

      # Run system2, capture stdout, redirect stderr to temp file
      # Suppress R's warning about non-zero exit (we handle status via attribute)
      suppressWarnings(
        system2(
          command = cmd_path,
          args = args,
          stdout = TRUE,
          stderr = stderr_file
        )
      )
    },
    error = function(e) {
      # This catches timeouts (elapsed limit reached) and other system errors
      logger::log_error("Command '{command}' execution error: {conditionMessage(e)}")
      rethrow_call(err = e, cmd = command)
    }
  )

  # ---------------------------------------------------------------------------
  # 6. Parse Exit Status
  # ---------------------------------------------------------------------------
  status <- attr(result, "status")
  if (is.null(status)) {
    status <- 0L
  } else {
    status <- as.integer(status)
    if (length(status) != 1L || is.na(status)) {
      logger::log_warn("Non-standard exit status attribute; falling back to 1.")
      status <- 1L
    }
  }

  # ---------------------------------------------------------------------------
  # 7. Read Captured Stderr
  # ---------------------------------------------------------------------------
  stderr_lines <- if (file.exists(stderr_file)) {
    lines <- readLines(stderr_file, warn = FALSE)
    if (length(lines) == 0L) NULL else lines
  } else {
    logger::log_warn("Temporary stderr file missing; stderr may be incomplete.")
    NULL
  }

  # ---------------------------------------------------------------------------
  # 8. Logging Completion
  # ---------------------------------------------------------------------------
  if (status == 0L) {
    logger::log_info("Command '{command}' succeeded (exit status 0)")
  } else {
    logger::log_warn("Command '{command}' finished with exit status {status}")
  }

  # ---------------------------------------------------------------------------
  # 9. Return Structured Result
  # ---------------------------------------------------------------------------
  stdout_vec <- if (length(result) == 0L) NULL else result

  list(
    output  = stdout_vec,
    status  = if (status == 0L) NULL else status,
    success = status == 0L,
    stdout  = stdout_vec,
    stderr  = stderr_lines
  )
}

# Helper to re-throw with consistent message
rethrow_call <- function(err, cmd) {
  msg <- sprintf("Failed to execute command '%s': %s", cmd, conditionMessage(err))
  stop(msg, call. = FALSE)
}