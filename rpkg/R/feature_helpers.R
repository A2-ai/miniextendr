#' Check if a feature is enabled
#'
#' Check if a specific optional feature was compiled into the package.
#'
#' @param name Character string naming the feature to check.
#' @return Logical `TRUE` if the feature is enabled, `FALSE` otherwise.
#' @examples
#' rpkg_has_feature("uuid")
#' rpkg_has_feature("time")
#' @export
rpkg_has_feature <- function(name) {
  name %in% rpkg_enabled_features()
}

#' Skip test if feature is missing
#'
#' For use in testthat tests to skip tests when an optional feature is not enabled.
#'
#' @param name Character string naming the required feature.
#' @return Invisibly returns `NULL`. Called for its side effect of skipping tests.
#' @examples
#' \dontrun{
#' test_that("uuid feature works", {
#'   skip_if_missing_feature("uuid")
#'   # ... test code ...
#' })
#' }
#' @export
skip_if_missing_feature <- function(name) {
  if (!rpkg_has_feature(name)) {
    testthat::skip(paste("feature not enabled:", name))
  }
}
