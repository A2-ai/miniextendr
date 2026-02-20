# miniextendr.yml configuration file support

#' Read miniextendr.yml configuration
#'
#' Looks for `miniextendr.yml` in the project root and parses it.
#' Falls back to defaults for any missing keys. The `yaml` package
#' is required to read the file; if unavailable, defaults are returned
#' with a warning.
#'
#' @param path Path to the project root (default: current directory).
#' @return A list of configuration values.
#' @export
mx_config <- function(path = ".") {
  config_path <- file.path(path, "miniextendr.yml")
  defaults <- mx_config_defaults()

  if (!file.exists(config_path)) {
    return(defaults)
  }

  if (!requireNamespace("yaml", quietly = TRUE)) {
    cli::cli_warn(
      "{.pkg yaml} package not installed; using default config.",
      class = "mx_config_no_yaml"
    )
    return(defaults)
  }

  user_config <- tryCatch(
    yaml::read_yaml(config_path),
    error = function(e) {
      cli::cli_warn(
        c("Failed to parse {.path miniextendr.yml}: {conditionMessage(e)}",
          "i" = "Using default config."),
        class = "mx_config_parse_error"
      )
      NULL
    }
  )

  if (is.null(user_config)) {
    return(defaults)
  }

  # Warn about unknown keys
  unknown <- setdiff(names(user_config), names(defaults))
  if (length(unknown) > 0) {
    cli::cli_warn(
      "Unknown key{?s} in {.path miniextendr.yml}: {.val {unknown}}.",
      class = "mx_config_unknown_keys"
    )
  }

  # Merge user config over defaults (only known keys)
  for (key in intersect(names(user_config), names(defaults))) {
    defaults[[key]] <- user_config[[key]]
  }

  defaults
}

#' Create miniextendr.yml in the project root
#'
#' Copies the default `miniextendr.yml` template into the project root.
#' Called automatically by [use_miniextendr()]; can also be run standalone.
#'
#' @param path Path to the project root (default: current project).
#' @return Invisibly returns `TRUE` if the file was created.
#' @keywords internal
use_miniextendr_config <- function(path = ".") {
  with_project(path)
  target <- usethis::proj_path("miniextendr.yml")

  if (fs::file_exists(target)) {
    cli::cli_alert_info("{.path miniextendr.yml} already exists, skipping")
    return(invisible(FALSE))
  }

  src <- system.file("templates", "miniextendr.yml",
                     package = "minirextendr", mustWork = TRUE)
  fs::file_copy(src, target)
  bullet_created("miniextendr.yml")
  invisible(TRUE)
}

#' Default configuration values
#'
#' Returns the default configuration used when no `miniextendr.yml` exists
#' or when keys are missing from the user's file.
#'
#' @return A named list with default configuration values:
#'   \describe{
#'     \item{class_system}{Default class system: `"env"`, `"r6"`, `"s3"`, `"s4"`, or `"s7"`.}
#'     \item{strict}{Strict mode for lossy conversions (logical).}
#'     \item{coerce}{Automatic type coercion (logical).}
#'     \item{features}{Character vector of additional Cargo features to enable.}
#'     \item{rust_version}{Rust toolchain version string.}
#'     \item{vendor}{Whether to vendor dependencies for CRAN (logical).}
#'   }
#' @export
mx_config_defaults <- function() {
  list(
    class_system = "env",
    strict = FALSE,
    coerce = FALSE,
    features = character(0),
    rust_version = "stable",
    vendor = TRUE
  )
}
