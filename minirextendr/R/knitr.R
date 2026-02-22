# knitr engine integration for miniextendr

#' Set up miniextendr knitr engine
#'
#' Registers a custom knitr language engine named `"miniextendr"`.
#'
#' When `path` is provided (default `"."`), runs [miniextendr_sync()] to
#' ensure an existing package is built before knitting. Chunks are then
#' evaluated as R code.
#'
#' When `path = NULL`, enables **inline mode**: each `{miniextendr}` chunk
#' is compiled as standalone Rust code via [rust_source()], and compiled
#' functions are loaded into the knitr global environment. This is useful
#' for self-contained vignettes that don't require an existing package.
#'
#' @param path Path to the R package root, `"."` to use the current directory,
#'   or `NULL` for inline compilation mode.
#' @param ... Additional arguments passed to [miniextendr_sync()] (package mode)
#'   or [rust_source()] (inline mode).
#' @return Invisibly returns `TRUE`.
#' @export
#'
#' @examples
#' \dontrun{
#' # Package mode (in a package vignette):
#' miniextendr_knitr_setup()
#'
#' # Inline mode (self-contained vignette):
#' miniextendr_knitr_setup(path = NULL)
#' }
miniextendr_knitr_setup <- function(path = ".", ...) {
  if (!requireNamespace("knitr", quietly = TRUE)) {
    cli::cli_abort(c(
      "{.pkg knitr} is required for miniextendr_knitr_setup()",
      "i" = 'Install it with: install.packages("knitr")'
    ))
  }

  if (is.null(path)) {
    # Inline mode: register engine that compiles chunks via rust_source()
    inline_args <- list(...)
    knitr::knit_engines$set(miniextendr = function(options) {
      eng_miniextendr_inline(options, extra_args = inline_args)
    })
  } else {
    # Package mode: sync the package before knitting
    miniextendr_sync(path = path, ...)
    knitr::knit_engines$set(miniextendr = eng_miniextendr)
  }

  invisible(TRUE)
}

#' miniextendr knitr engine (package mode)
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
    cli::cli_abort("{.pkg knitr} is required")
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

#' miniextendr knitr engine (inline mode)
#'
#' Compiles each `{miniextendr}` chunk as standalone Rust code via
#' [rust_source()] and loads the functions into the knitr global environment.
#'
#' Supported chunk options:
#' - `quiet`: suppress build output (logical, default TRUE)
#' - `features`: comma-separated cargo features to enable
#'
#' @param options Chunk options list (provided by knitr)
#' @param extra_args Additional arguments from miniextendr_knitr_setup()
#' @return Engine output (via [knitr::engine_output()])
#' @noRd
eng_miniextendr_inline <- function(options, extra_args = list()) {
  if (!requireNamespace("knitr", quietly = TRUE)) {
    cli::cli_abort("{.pkg knitr} is required")
  }

  code <- paste(options$code, collapse = "\n")

  out <- ""
  if (options$eval) {
    quiet <- options$quiet %||% TRUE
    features <- options$features %||% character()
    if (is.character(features) && length(features) == 1 && grepl(",", features)) {
      features <- trimws(strsplit(features, ",")[[1]])
    }

    args <- c(
      list(code = code, env = knitr::knit_global(), quiet = quiet,
           features = features),
      extra_args
    )

    out <- tryCatch(
      {
        do.call(rust_source, args)
        ""
      },
      error = function(e) conditionMessage(e)
    )
  }

  knitr::engine_output(options, code, out)
}
