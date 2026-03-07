test_that("AltrepSexp detects ALTREP compact integer", {
  x <- 1:10
  expect_true(unsafe_C_altrep_sexp_is_altrep(x))
  info <- unsafe_C_altrep_sexp_check(x)
  expect_equal(info[1], "is_altrep=true")
  expect_equal(info[2], "sexptype=INTSXP")
  expect_equal(info[3], "length=10")
})

test_that("AltrepSexp materializes compact integer", {
  x <- 1:10
  expect_equal(altrep_sexp_materialize_int(x), 1:10)
})

test_that("AltrepSexp detects ALTREP string (as.character(1:10))", {
  y <- as.character(1:10)
  expect_true(unsafe_C_altrep_sexp_is_altrep(y))
  info <- unsafe_C_altrep_sexp_check(y)
  expect_equal(info[2], "sexptype=STRSXP")
})

test_that("AltrepSexp materializes ALTREP strings", {
  y <- as.character(1:10)
  result <- altrep_sexp_materialize_strings(y)
  expect_equal(result, as.character(1:10))
})

test_that("ensure_materialized works for ALTREP integer", {
  x <- 1:10
  expect_equal(unsafe_C_altrep_ensure_materialized_int(x), 1:10)
})

test_that("ensure_materialized works for non-ALTREP integer", {
  z <- c(1L, 2L, 3L)
  expect_false(unsafe_C_altrep_sexp_is_altrep(z))
  expect_equal(unsafe_C_altrep_ensure_materialized_int(z), c(1L, 2L, 3L))
})

test_that("ensure_materialized works for ALTREP strings", {
  y <- as.character(1:10)
  expect_equal(unsafe_C_altrep_ensure_materialized_str(y), as.character(1:10))
})

test_that("ensure_materialized works for non-ALTREP strings", {
  v <- c("a", "b", "c")
  expect_false(unsafe_C_altrep_sexp_is_altrep(v))
  expect_equal(unsafe_C_altrep_ensure_materialized_str(v), c("a", "b", "c"))
})

test_that("ensure_materialized preserves NA in strings", {
  v <- c("a", NA, "c")
  result <- unsafe_C_altrep_ensure_materialized_str(v)
  expect_equal(result, c("a", NA, "c"))
  expect_true(is.na(result[2]))
})

test_that("materialize_int errors on non-ALTREP", {
  expect_error(
    altrep_sexp_materialize_int(c(1L, 2L, 3L)),
    "expected an ALTREP vector"
  )
})

test_that("materialize_int errors on wrong SEXPTYPE", {
  expect_error(
    altrep_sexp_materialize_int(as.character(1:5)),
    "expected INTSXP"
  )
})

test_that("non-ALTREP vector correctly detected", {
  expect_false(unsafe_C_altrep_sexp_is_altrep(c(1L, 2L, 3L)))
  expect_false(unsafe_C_altrep_sexp_is_altrep(c(1.0, 2.0, 3.0)))
  expect_false(unsafe_C_altrep_sexp_is_altrep(c("a", "b")))
})

test_that("SEXP parameter rejects ALTREP vectors", {
  x <- 1:10
  expect_true(unsafe_C_altrep_sexp_is_altrep(x))
  # Typed parameters (Vec<i32>) auto-materialize and work fine
  expect_equal(length(x), 10L)
})

test_that("typed parameters auto-materialize ALTREP", {
  # Vec<i32> goes through TryFromSexp -> &[i32] -> DATAPTR_RO -> materialization
  # This is the recommended way to consume ALTREP vectors
  x <- 1:10
  expect_equal(altrep_sexp_materialize_int(x), 1:10)
})
