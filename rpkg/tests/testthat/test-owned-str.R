# Box<str> / Rc<str> / Arc<str> conversions and their string ALTREPs.
# Fixtures: src/rust/owned_str_tests.rs.

is_altrep <- function(x) miniextendr:::unsafe_C_altrep_sexp_is_altrep(x)

test_that("scalars and vectors convert like String, NA included", {
  expect_identical(miniextendr:::owned_str_box_scalar("æøå"), "æøå")
  expect_identical(miniextendr:::owned_str_box_scalar(NA_character_), "")
  expect_identical(
    miniextendr:::owned_str_rc_vec(c("one", NA, "")),
    c("one", NA, "")
  )
  expect_identical(miniextendr:::owned_str_rc_vec(character()), character())
  expect_true(miniextendr:::owned_str_arc_is_na(NA_character_))
  expect_false(miniextendr:::owned_str_arc_is_na("x"))
  expect_error(miniextendr:::owned_str_box_scalar(1L))
})

test_that("a string ALTREP keeps NA, takes writes, and copies keep their own", {
  x <- miniextendr:::owned_str_arc_altrep()
  expect_true(is_altrep(x))
  expect_identical(x, c("one", NA, "three"))
  expect_identical(anyNA(x), TRUE)
  y <- x
  y[2] <- "two"
  x[1] <- NA
  expect_identical(x, c(NA, NA, "three"))
  expect_identical(y, c("one", "two", "three"))
})

test_that("an unwritten vector is saved as its class, a written one as a plain vector", {
  r <- miniextendr:::owned_str_rc_altrep()
  expect_identical(r[3], "same")
  restored <- unserialize(serialize(r, NULL))
  expect_true(is_altrep(restored))
  expect_identical(restored, c("same", "other", "same"))
  r[2] <- NA
  expect_identical(unserialize(serialize(r, NULL)), c("same", NA, "same"))
})

test_that("owned str vectors survive writes and copies under gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  x <- miniextendr:::owned_str_arc_altrep()
  first <- x[1]
  y <- x
  y[2] <- "two"
  v <- miniextendr:::owned_str_rc_vec(c("a", NA))
  miniextendr:::gc_stress_owned_str()
  gctorture(FALSE)
  expect_identical(first, "one")
  expect_identical(x, c("one", NA, "three"))
  expect_identical(y, c("one", "two", "three"))
  expect_identical(v, c("a", NA))
})
