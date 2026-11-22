# Skip the calling test when this build cannot create custom connections.
# `RCustomConnection::build` (and the `RConnectionIo` builders on top of it)
# needs `R_new_custom_connection`, which R lists as API only from R 4.6. Built
# against an older R without the `nonapi` feature, it raises an R error
# instead (docs/CONNECTIONS.md, "R versions before 4.6").
skip_if_no_custom_connections <- function() {
  if (getRversion() < "4.6.0" && !miniextendr_has_feature("nonapi")) {
    skip("custom connections need R >= 4.6 or the nonapi feature")
  }
}
