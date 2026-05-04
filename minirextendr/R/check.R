# Static R CMD check (no Rust compile)

#' Run R CMD check without triggering a Rust compile
#'
#' Wraps [rcmdcheck::rcmdcheck()] with `--install=fake` so that
#' `./configure` (and therefore `cargo`) is never invoked. This makes it
#' suitable for fast iteration on un-vendored packages, sandboxed
#' environments (e.g. Claude Code, CRAN incoming `--no-internet`), or any
#' context where `cargo` cannot reach the network.
#'
#' All static phases still run. Phases that require a loaded package
#' are automatically skipped by R CMD check when `--install=fake` is active.
#'
#' **Catches:**
#' - DESCRIPTION fields (Authors\@R, Version, License, dependency syntax)
#' - NAMESPACE (export consistency, unresolved imports)
#' - R code syntax and parse errors
#' - Rd file parsing, cross-references, and metadata
#' - code-usage analysis (`codetools` -- undefined globals, etc.)
#' - sub-directory checks (`man/`, `tests/`, `R/`)
#' - top-level file checks (LICENSE presence, etc.)
#' - ASCII / encoding checks
#'
#' **Does not catch:**
#' - Rust compile errors
#' - `#[miniextendr]` macro expansion errors
#' - ABI / linkage issues
#' - anything in `tests/testthat/*` (tests are skipped)
#' - examples in Rd files
#' - vignettes
#'
#' The `--install=fake` flag is honoured by R CMD check since at least R 3.5
#' (r-svn `tools/R/install.R:2424`, `tools/R/check.R:7373`).
#'
#' @param path Path to the R package root, or `"."` to use the current
#'   directory.
#' @param args Character vector of arguments passed to `R CMD check`.
#'   Must contain `--install=fake`; a warning is emitted if a different
#'   `--install=` value is detected.
#' @param build_args Character vector of arguments passed to `R CMD build`.
#' @param error_on Severity level at which to error. One of `"error"`,
#'   `"warning"`, or `"note"`. Passed to [rcmdcheck::rcmdcheck()].
#' @param roxygenize If `TRUE` (the default) and `roxygen2` is installed,
#'   call `roxygen2::roxygenize(path)` before running the check. This
#'   removes false-positive stale-documentation warnings without requiring
#'   cargo. Silently skipped if `roxygen2` is not installed. Pass
#'   `roxygenize = FALSE` in tests that intentionally exercise broken-Rd
#'   code paths.
#' @param ... Additional arguments forwarded to [rcmdcheck::rcmdcheck()].
#'
#' @return The [rcmdcheck::rcmdcheck()] result object, invisibly.
#'
#' @note Run [miniextendr_check()] for full validation (including Rust
#'   compilation) before a CRAN submission.
#'
#' @seealso [miniextendr_check()] for the full check including Rust
#'   compilation.
#'
#' @export
miniextendr_check_static <- function(path = ".",
                                     args = c("--install=fake", "--no-manual"),
                                     build_args = c("--no-build-vignettes"),
                                     error_on = "warning",
                                     roxygenize = TRUE,
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

  # Optional roxygenize step: updates NAMESPACE + Rd without cargo.
  # Removes false-positive stale-doc warnings that would otherwise appear
  # when the user's Rd files are out-of-date relative to their R source.
  if (isTRUE(roxygenize)) {
    if (requireNamespace("roxygen2", quietly = TRUE)) {
      cli::cli_alert_info("Running roxygen2::roxygenize() to sync Rd files")
      roxygen2::roxygenize(usethis::proj_get())
    } else {
      cli::cli_alert_info(
        "roxygen2 not installed -- skipping roxygenize step (pass {.code roxygenize = FALSE} to silence)"
      )
    }
  }

  cli::cli_alert_info("Skipped phases: install (configure/cargo), examples, tests, vignettes")
  cli::cli_alert_info("Running phases: DESCRIPTION, NAMESPACE, R code, Rd, subdirs")

  result <- rcmdcheck::rcmdcheck(
    usethis::proj_get(),
    args = args,
    build_args = build_args,
    error_on = error_on,
    ...
  )

  invisible(result)
}
