# Static R CMD check (no Rust compile)

#' Run R CMD check without triggering a Rust compile
#'
#' Wraps [rcmdcheck::rcmdcheck()] with `--install=fake` so that
#' `./configure` (and therefore `cargo`) is never invoked.  All static phases
#' still run: DESCRIPTION, NAMESPACE, R code syntax, Rd parsing, code-usage
#' analysis, sub-dirs, etc.  Phases that require a loaded package (examples,
#' tests, vignettes) are automatically skipped by R CMD check when
#' `--install=fake` is active.
#'
#' Use this in offline, sandboxed, or no-network environments where cargo
#' cannot fetch dependencies, or as a fast pre-vendor sanity check.  Run
#' [miniextendr_check()] for full validation before a CRAN submission.
#'
#' The `--install=fake` flag is honoured by R CMD check since at least R 3.5
#' (r-svn `tools/R/install.R:2424`, `tools/R/check.R:7373`).
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param args Character vector of arguments passed to `R CMD check`.
#'   Must contain `--install=fake`; a warning is emitted if a different
#'   `--install=` value is detected.
#' @param build_args Character vector of arguments passed to `R CMD build`.
#' @param error_on Severity level at which to error. One of `"error"`,
#'   `"warning"`, or `"note"`. Passed to [rcmdcheck::rcmdcheck()].
#' @param ... Additional arguments forwarded to [rcmdcheck::rcmdcheck()].
#' @return The [rcmdcheck::rcmdcheck()] result object, invisibly.
#' @seealso [miniextendr_check()] for the full check including Rust compilation.
#' @export
miniextendr_check_static <- function(path = ".",
                                     args = c("--install=fake", "--no-manual"),
                                     build_args = c("--no-build-vignettes"),
                                     error_on = "warning",
                                     ...) {
  with_project(path)
  if (!requireNamespace("rcmdcheck", quietly = TRUE)) {
    cli::cli_abort(c(
      "rcmdcheck is required for miniextendr_check_static()",
      "i" = 'Install it with: install.packages("rcmdcheck")'
    ))
  }

  # Guarantee --install=fake is in args (and warn if the user overrode it).
  if (!any(grepl("^--install=", args))) {
    args <- c(args, "--install=fake")
  } else if (!any(grepl("^--install=fake$", args))) {
    cli::cli_warn(c(
      "miniextendr_check_static() requires --install=fake to skip Rust compile",
      "i" = "Override detected in {.arg args}; check may trigger cargo"
    ))
  }

  cli::cli_h1("miniextendr static check (no Rust compile)")
  cli::cli_alert_info("Skipped phases: install, examples, tests, vignettes")
  cli::cli_alert_info("Running phases: DESCRIPTION, NAMESPACE, R code, Rd, subdirs")

  rcmdcheck::rcmdcheck(
    usethis::proj_get(),
    args = args,
    build_args = build_args,
    error_on = error_on,
    ...
  )
}
