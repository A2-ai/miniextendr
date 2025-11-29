
# ALTREP wrappers (pure .Call). We pass scalars as integer vectors of length 1.

altrep_compact_int <- function(n, start, step) {
  .Call(rpkg_altrep_compact_int, as.integer(n), as.integer(start), as.integer(step))
}

altrep_from_doubles <- function(x) {
  .Call(rpkg_altrep_from_doubles, as.double(x))
}

altrep_from_strings <- function(x) {
  .Call(rpkg_altrep_from_strings, as.character(x))
}

altrep_from_logicals <- function(x) {
  .Call(rpkg_altrep_from_logicals, as.logical(x))
}

altrep_from_raw <- function(x) {
  .Call(rpkg_altrep_from_raw, as.raw(x))
}

altrep_from_list <- function(x) {
  .Call(rpkg_altrep_from_list, x)
}

# Proc-macro ALTREP test: creates ConstantIntClass instances (always 42, length 10)
altrep_constant_int <- function() {
  .Call(rpkg_constant_int)
}
