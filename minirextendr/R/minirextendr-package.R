#' @keywords internal
"_PACKAGE"

## usethis namespace: start
## usethis namespace: end
NULL

# Null-coalescing operator (inlined from rlang)
`%||%` <- function(x, y) {
  if (is.null(x)) y else x
}
# Use base R version when available (R >= 4.4.0)
if (exists("%||%", envir = baseenv())) {
  `%||%` <- get("%||%", envir = baseenv())
}
