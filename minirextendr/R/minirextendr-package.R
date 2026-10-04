#' @keywords internal
"_PACKAGE"

## usethis namespace: start
## usethis namespace: end
NULL

#' Default value for `NULL`
#'
#' Null-coalescing operator, inlined from rlang. Base R has had `%||%` since
#' 4.4.0; minirextendr declares no R version floor, so it keeps this local
#' definition and replaces it with base R's when that exists.
#'
#' @param x,y If `x` is `NULL`, return `y`; otherwise return `x`.
#' @return `y` when `x` is `NULL`, else `x`.
#' @name op-null-default
#' @aliases %||%
#' @keywords internal
`%||%` <- function(x, y) {
  if (is.null(x)) y else x
}
# Use base R version when available (R >= 4.4.0)
if (exists("%||%", envir = baseenv())) {
  `%||%` <- get("%||%", envir = baseenv())
}
