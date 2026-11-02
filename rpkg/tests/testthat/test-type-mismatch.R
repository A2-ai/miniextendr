# TypeMismatchError: what ExternalPtr::wrap_sexp_with_error reports for a
# live pointer, a restored one, one saved by another version, one of another
# type and pointers miniextendr did not build (type_mismatch_tests.rs).

reload <- function(x) unserialize(serialize(x, NULL))

test_that("a live pointer of the type is read", {
  out <- mismatch_probe_read(mismatch_probe_new(7L))
  expect_identical(out, list(variant = "live", value = 7L))
})

test_that("a pointer of another type is Mismatch", {
  out <- mismatch_probe_read(mismatch_other_new())
  expect_identical(out$variant, "mismatch")
  expect_null(out$classes)
  expect_match(out$expected, "MismatchProbe", fixed = TRUE)
  expect_match(out$found, "MismatchOther", fixed = TRUE)
  expect_match(out$message, "type mismatch: expected", fixed = TRUE)
})

test_that("a restored pointer is Restored, with the restored classes", {
  out <- mismatch_probe_read(reload(mismatch_probe_new(7L)))
  expect_identical(out$variant, "restored")
  expect_identical(
    out$classes,
    c("miniextendr_restored_no_value", "miniextendr_restored")
  )
  expect_match(out$type_name, "MismatchProbe", fixed = TRUE)
  expect_match(out$message, "restored from a saved session", fixed = TRUE)
})

test_that("a save by another version is OtherVersion, with both versions", {
  back <- reload(mismatch_probe_new(7L))
  sidecar_rewrite_saved_version(back, "0.0.1")
  out <- mismatch_probe_read(back)
  expect_identical(out$variant, "other_version")
  expect_identical(
    out$classes,
    c("miniextendr_restored_other_version", "miniextendr_restored")
  )
  expect_identical(out$crate_name, "miniextendr")
  expect_identical(out$saved, "0.0.1")
  expect_identical(out$current, as.character(packageVersion("miniextendr")))
  expect_match(out$message, "saved by miniextendr 0.0.1", fixed = TRUE)
})

test_that("a restored pointer of another type is Mismatch", {
  out <- mismatch_probe_read(reload(mismatch_other_new()))
  expect_identical(out$variant, "mismatch")
  expect_match(out$found, "MismatchOther", fixed = TRUE)
})

test_that("pointers miniextendr did not build are NullPointer and InvalidTypeId", {
  out <- mismatch_probe_read(methods::new("externalptr"))
  expect_identical(out$variant, "null_pointer")
  expect_null(out$classes)
  expect_identical(out$message, "external pointer is null")

  # A registered routine's address: a live pointer with no type ID.
  routine <- getDLLRegisteredRoutines("miniextendr")$.Call[[1]]$address
  expect_identical(typeof(routine), "externalptr")
  out <- mismatch_probe_read(routine)
  expect_identical(out$variant, "invalid_type_id")
  expect_identical(out$message, "external pointer has no valid type id")
})

test_that("a value that is no external pointer is refused before the read", {
  expect_error(mismatch_probe_read(1L), "`x` is an external pointer")
})
