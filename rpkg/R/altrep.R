
# ALTREP wrappers (pure .Call). We pass scalars as integer vectors of length 1.

altrep_compact_int <- function(n, start, step) {
  .Call(C_altrep_compact_int, .call = match.call(),
        as.integer(n), as.integer(start), as.integer(step))
}

altrep_from_doubles <- function(x) {
  .Call(C_altrep_from_doubles, .call = match.call(), as.double(x))
}

altrep_from_strings <- function(x) {
  .Call(C_altrep_from_strings, .call = match.call(), as.character(x))
}
