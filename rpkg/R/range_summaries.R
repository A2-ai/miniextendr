# Shared help page for the range-summary fixtures in
# src/rust/shared_param_docs.rs (#1590). Each argument is documented once
# here; the Rust functions and `RangeBox` methods (inherent and trait-impl)
# join the page (`@rdname`, `@describeIn`) or inherit the descriptions
# (`@inheritParams`), and their generated wrappers must not add `@param`
# lines that would replace these.
#
# roxygen2 reads R/ in alphabetical order and a merged page takes its name
# and title from the first block it reads. This file sorts after
# R/miniextendr-wrappers.R on purpose: the generated blocks that join the
# page carry `@order NaN`, which sorts them after this block anyway, so the
# page is named and titled from here whatever the file order.

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
#' @param x A `RangeBox` object.
#' @param ... Unused; accepted for S3 method compatibility.
#' @returns `range_width()`, `range_midpoint()`, `box_width()` and
#'   `box_midpoint()` return a number, `range_within()` and `box_covers()` a
#'   logical vector with one element per value, `range_clamp()` and
#'   `box_clamp()` a numeric vector, and `rangebox_from_values()` a
#'   `RangeBox`.
#' @examples
#' x <- c(2, 5, 9)
#' range_width(x)
#' range_within(x, 3, 9, inclusive = TRUE)
#' range_clamp(x, 3, 8)
#' range_clamp(x, 3, 8, outside = "drop")
#' range_midpoint(x)
#' box <- rangebox_from_values(x)
#' box_width(box)
#' box_covers(box, c(1, 5))
#' @aliases box_width box_covers
#' @name range_summaries
NULL
