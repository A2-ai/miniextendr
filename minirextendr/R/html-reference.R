#' Build the single-page HTML reference manual
#'
#' Runs the package's `tools/build-html-reference.R` (added by
#' [use_miniextendr_html_reference()] and by `upgrade_miniextendr_package()`)
#' through `Rscript`, so the page is built exactly as the shell invocation
#' builds it: every `man/*.Rd` is validated with `tools::checkRd()` and
#' rendered with `tools::pkg2HTML()` (R >= 4.4) into one HTML page next to
#' the rustdoc output. Nothing is installed or loaded.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param out Output directory. `NULL` uses the script's default,
#'   `src/rust/target/doc/r` inside the package.
#' @param strict If `FALSE`, `checkRd()` findings are printed and the page is
#'   still rendered (sets `MINIEXTENDR_HTML_STRICT=0` for the script). The
#'   default refuses to build on any finding.
#' @return Invisibly, the path of the rendered `<package>.html`.
#' @export
miniextendr_html_reference <- function(path = ".", out = NULL, strict = TRUE) {
  pkg_dir <- normalizePath(path, mustWork = TRUE)
  script <- file.path(pkg_dir, "tools", "build-html-reference.R")
  if (!file.exists(script)) {
    cli::cli_abort(c(
      "{.path tools/build-html-reference.R} not found in {.path {pkg_dir}}",
      "i" = "Run {.fn use_miniextendr_html_reference} (or {.fn upgrade_miniextendr_package}) to add it."
    ))
  }
  old <- Sys.getenv("MINIEXTENDR_HTML_STRICT", unset = NA)
  on.exit(
    if (is.na(old)) Sys.unsetenv("MINIEXTENDR_HTML_STRICT") else Sys.setenv(MINIEXTENDR_HTML_STRICT = old),
    add = TRUE
  )
  if (strict) Sys.unsetenv("MINIEXTENDR_HTML_STRICT") else Sys.setenv(MINIEXTENDR_HTML_STRICT = "0")
  status <- system2(
    file.path(R.home("bin"), "Rscript"),
    c(shQuote(script), if (!is.null(out)) shQuote(out))
  )
  if (!identical(status, 0L)) {
    cli::cli_abort("{.path tools/build-html-reference.R} failed with exit status {status}")
  }
  pkg <- unname(read.dcf(file.path(pkg_dir, "DESCRIPTION"), fields = "Package")[1L, "Package"])
  out_dir <- if (is.null(out)) file.path(pkg_dir, "src", "rust", "target", "doc", "r") else out
  invisible(normalizePath(file.path(out_dir, paste0(pkg, ".html")), mustWork = TRUE))
}
