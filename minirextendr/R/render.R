# rmarkdown and Quarto integration for miniextendr

# =============================================================================
# rmarkdown wrapper formats
# =============================================================================

#' miniextendr-aware HTML document format
#'
#' Wraps [rmarkdown::html_document()] with a `pre_knit` hook that calls
#' [miniextendr_sync()] before knitting. Use this as the output format
#' in your vignette YAML header to ensure the Rust package is built.
#'
#' @param ... Arguments passed to [rmarkdown::html_document()].
#' @param sync_args Named list of arguments passed to [miniextendr_sync()].
#' @return An rmarkdown output format.
#' @export
#'
#' @examples
#' \dontrun{
#' # In YAML header:
#' # output: minirextendr::miniextendr_html_document
#' }
miniextendr_html_document <- function(..., sync_args = list()) {
  if (!requireNamespace("rmarkdown", quietly = TRUE)) {
    abort(c(
      "{.pkg rmarkdown} is required for miniextendr_html_document()",
      "i" = 'Install it with: install.packages("rmarkdown")'
    ))
  }

  fmt <- rmarkdown::html_document(...)
  fmt$pre_knit <- make_pre_knit(sync_args)
  fmt
}

#' miniextendr-aware PDF document format
#'
#' Wraps [rmarkdown::pdf_document()] with a `pre_knit` hook that calls
#' [miniextendr_sync()] before knitting.
#'
#' @param ... Arguments passed to [rmarkdown::pdf_document()].
#' @param sync_args Named list of arguments passed to [miniextendr_sync()].
#' @return An rmarkdown output format.
#' @export
#'
#' @examples
#' \dontrun{
#' # In YAML header:
#' # output: minirextendr::miniextendr_pdf_document
#' }
miniextendr_pdf_document <- function(..., sync_args = list()) {
  if (!requireNamespace("rmarkdown", quietly = TRUE)) {
    abort(c(
      "{.pkg rmarkdown} is required for miniextendr_pdf_document()",
      "i" = 'Install it with: install.packages("rmarkdown")'
    ))
  }

  fmt <- rmarkdown::pdf_document(...)
  fmt$pre_knit <- make_pre_knit(sync_args)
  fmt
}

#' miniextendr-aware Word document format
#'
#' Wraps [rmarkdown::word_document()] with a `pre_knit` hook that calls
#' [miniextendr_sync()] before knitting.
#'
#' @param ... Arguments passed to [rmarkdown::word_document()].
#' @param sync_args Named list of arguments passed to [miniextendr_sync()].
#' @return An rmarkdown output format.
#' @export
#'
#' @examples
#' \dontrun{
#' # In YAML header:
#' # output: minirextendr::miniextendr_word_document
#' }
miniextendr_word_document <- function(..., sync_args = list()) {
  if (!requireNamespace("rmarkdown", quietly = TRUE)) {
    abort(c(
      "{.pkg rmarkdown} is required for miniextendr_word_document()",
      "i" = 'Install it with: install.packages("rmarkdown")'
    ))
  }

  fmt <- rmarkdown::word_document(...)
  fmt$pre_knit <- make_pre_knit(sync_args)
  fmt
}

#' Create a pre_knit hook that runs miniextendr_sync
#'
#' @param sync_args Named list of arguments for miniextendr_sync()
#' @return A function suitable for use as `pre_knit` in an rmarkdown format
#' @noRd
make_pre_knit <- function(sync_args = list()) {
  function(input, ...) {
    # Resolve package root from input file location
    pkg_path <- find_root_with_file("DESCRIPTION", dirname(input)) %||%
      dirname(input)
    args <- c(list(path = pkg_path), sync_args)
    do.call(miniextendr_sync, args)
  }
}

# =============================================================================
# Quarto integration
# =============================================================================

#' Quarto pre-render hook for miniextendr
#'
#' Entry point for Quarto's `project.pre-render` script. Runs
#' [miniextendr_sync()] to ensure the package is built before rendering.
#'
#' @param path Path to the R package root, or `"."` to use the current directory.
#' @param ... Additional arguments passed to [miniextendr_sync()].
#' @return Invisibly returns the result of [miniextendr_sync()].
#' @export
#'
#' @examples
#' \dontrun{
#' # In _quarto.yml:
#' # project:
#' #   pre-render: Rscript -e 'minirextendr::miniextendr_quarto_pre_render()'
#' }
miniextendr_quarto_pre_render <- function(path = ".", ...) {
  miniextendr_sync(path = path, ...)
}
