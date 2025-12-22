
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

# Complex number ALTREP: unit circle (roots of unity)
unit_circle <- function(n) {
  .Call(C_unit_circle, .call = match.call(), as.integer(n))
}

# Lazy materialization ALTREP
lazy_int_seq <- function(from, to, by) {
  .Call(C_lazy_int_seq, .call = match.call(), as.integer(from), as.integer(to), as.integer(by))
}

altrep_lazy_int_seq_is_materialized <- function(x) {
  .Call(rpkg_lazy_int_seq_is_materialized, x)
}

# Box<[T]> ALTREP
boxed_ints <- function(n) {
  .Call(C_boxed_ints, .call = match.call(), as.integer(n))
}

# Static slice ALTREP
static_ints <- function() {
  .Call(C_static_ints, .call = match.call())
}

leaked_ints <- function(n) {
  .Call(C_leaked_ints, .call = match.call(), as.integer(n))
}

static_strings <- function() {
  .Call(C_static_strings, .call = match.call())
}
