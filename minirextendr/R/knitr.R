# knitr engine integration for miniextendr

#' Set up miniextendr knitr engine
#'
#' Registers a custom knitr language engine named `"miniextendr"` and runs
#' [miniextendr_sync()] to ensure the package is built before knitting.
#' Call this in a setup chunk at the top of your vignette.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param ... Additional arguments passed to [miniextendr_sync()].
#' @return Invisibly returns `TRUE`.
#' @export
#'
#' @examples
#' \dontrun{
#' # In an Rmd/qmd setup chunk:
#' miniextendr_knitr_setup()
#' }
miniextendr_knitr_setup <- function(path = ".", ...) {
  if (!requireNamespace("knitr", quietly = TRUE)) {
    abort(c(
      "{.pkg knitr} is required for miniextendr_knitr_setup()",
      "i" = 'Install it with: install.packages("knitr")'
    ))
  }

  # Sync the package before knitting
  miniextendr_sync(path = path, ...)

  # Register the engine
  knitr::knit_engines$set(miniextendr = eng_miniextendr)

  invisible(TRUE)
}

#' miniextendr knitr engine
#'
#' Processes `miniextendr` chunks. Chunks are evaluated as R code
#' (after the package has been synced by [miniextendr_knitr_setup()]).
#'
#' Supported chunk options:
#' - `mode`: passed to [miniextendr_sync()] (`"if_stale"`, `"always"`, `"never"`)
#' - `stage`: passed to [miniextendr_sync()] (`"install"`, `"wrappers"`, `"build"`)
#' - `quiet`: suppress sync messages (logical)
#'
#' @param options Chunk options list (provided by knitr)
#' @return Engine output (via [knitr::engine_output()])
#' @noRd
eng_miniextendr <- function(options) {
  if (!requireNamespace("knitr", quietly = TRUE)) {
    abort("{.pkg knitr} is required")
  }

  # Extract miniextendr-specific options
  mode <- options$mode %||% "if_stale"
  stage <- options$stage %||% "install"
  quiet <- options$quiet %||% TRUE

  # Re-sync if chunk requests it
  if (!identical(mode, "never")) {
    miniextendr_sync(mode = mode, stage = stage, quiet = quiet)
  }

  # Evaluate the chunk as R code
  code <- paste(options$code, collapse = "\n")
  out <- if (options$eval) {
    tryCatch(
      utils::capture.output(eval(parse(text = code), envir = knitr::knit_global())),
      error = function(e) conditionMessage(e)
    )
  } else {
    ""
  }

  knitr::engine_output(options, code, out)
}
