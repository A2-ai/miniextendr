#' @keywords internal
"_PACKAGE"

## usethis namespace: start
#' @useDynLib miniextendr, .registration = TRUE
#' @importFrom R6 R6Class
#' @importFrom S7 convert
## usethis namespace: end
NULL

# S7's convert generic body references `properties` (an S7-internal symbol).
# Importing convert via @importFrom brings its body into our namespace,
# causing R CMD check to flag `properties` as an undefined global.
# This is a known S7 interop issue — no way to avoid importing convert
# while keeping S7::method(convert, ...) working during load_all().
utils::globalVariables("properties")

# Register S7 methods on package load
# This is required for S7 method registrations to work properly,
# especially for methods on generics from other packages (like S7::convert)
.onLoad <- function(libname, pkgname) {
  # suppressMessages: S7 re-registers methods that were already registered
  # at source time (from miniextendr-wrappers.R), causing harmless
  # "Overwriting method" messages. This is expected S7 behavior.
  suppressMessages(S7::methods_register())
}
