# Tests for cargo wrapper functions

test_that("cargo_new validates vcs against its choices", {
  tmp <- tempfile("bad-vcs-")
  on.exit(unlink(tmp, recursive = TRUE), add = TRUE)

  # An unknown vcs errors immediately: match.arg(vcs) runs before check_rust(),
  # so the diagnostic does not depend on the Rust toolchain being installed.
  expect_error(
    cargo_new(path = tmp, name = "mycrate", vcs = "svn"),
    "should be one of"
  )

  # The default (a missing arg) still resolves to "none": the choices vector
  # lists it first, so match.arg() picks it unchanged.
  expect_identical(eval(formals(cargo_new)$vcs)[[1]], "none")
})
