#' @keywords internal
"_PACKAGE"

## usethis namespace: start
#' @useDynLib miniextendr, .registration = TRUE
#' @importFrom R6 R6Class
#' @importFrom S7 convert
## usethis namespace: end
NULL

# Register S7 methods on package load
# This is required for S7 method registrations to work properly,
# especially for methods on generics from other packages (like S7::convert)
.onLoad <- function(libname, pkgname) {
  S7::methods_register()
}
