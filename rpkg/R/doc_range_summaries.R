# Shared help page for the range-summary fixtures in
# src/rust/shared_param_docs.rs (#1590). Each argument is documented once
# here; the Rust functions join the page (`@rdname`, `@describeIn`) or
# inherit the descriptions (`@inheritParams`), and their generated wrappers
# must not add placeholder `@param` lines that would replace these.
#
# roxygen2 reads R/ in alphabetical order and a merged page takes its name
# and title from the first block it reads, so this file sorts before
# R/miniextendr-wrappers.R.

#' Range summaries
#'
#' @description
#' Small summaries of where numeric values sit relative to a range. The
#' functions share this page, and each argument is described once for all
#' of them.
#'
#' @param values A numeric vector.
#' @param lower,upper Lower and upper bound of the range.
#' @param outside What to do with values outside the range: `"clamp"` moves
#'   them onto the nearer bound, `"drop"` removes them.
#' @returns `range_width()` and `range_midpoint()` return a number,
#'   `range_within()` a logical vector with one element per value, and
#'   `range_clamp()` a numeric vector.
#' @examples
#' x <- c(2, 5, 9)
#' range_width(x)
#' range_within(x, 3, 9, inclusive = TRUE)
#' range_clamp(x, 3, 8)
#' range_clamp(x, 3, 8, outside = "drop")
#' range_midpoint(x)
#' @name range_summaries
NULL
