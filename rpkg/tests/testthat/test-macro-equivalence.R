# Test macro bidirectional equivalence:
# #[miniextendr] on structs/enums produces working trait implementations.

test_that("#[miniextendr] on multi-field struct creates ExternalPtr", {
  p <- mx_point_new(3.0, 4.0)
  expect_true(is.environment(p) || inherits(p, "externalptr"))
  expect_equal(mx_point_sum(p), 7.0)
})

test_that("#[miniextendr(list)] on struct creates list-convertible type", {
  rec <- mx_record_create("hello", 42L)
  expect_type(rec, "list")
  expect_equal(rec$name, "hello")
  expect_equal(rec$value, 42L)
})

test_that("#[miniextendr(dataframe)] on struct creates data.frame", {
  df <- mx_obs_create()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2)
  expect_equal(df$id, c(1L, 2L))
  expect_equal(df$score, c(0.5, 0.8))
})

test_that("#[miniextendr] on fieldless enum creates factor", {
  s <- mx_season_summer()
  expect_s3_class(s, "factor")
  expect_equal(as.character(s), "Summer")
  expect_equal(levels(s), c("Spring", "Summer", "Autumn", "Winter"))
})

test_that("#[miniextendr] on enum supports roundtrip", {
  s <- mx_season_summer()
  name <- mx_season_name(s)
  expect_equal(name, "Summer")
})

test_that("#[miniextendr(match_arg)] on enum creates match_arg type", {
  v <- factor("Quiet", levels = c("Quiet", "Normal", "Verbose"))
  result <- mx_verbosity_check(v)
  expect_equal(result, "Quiet")
})

test_that("#[derive(Altrep)] on 1-field struct creates ALTREP vector", {
  ints <- mx_derived_ints()
  expect_equal(ints, c(10L, 20L, 30L))
})
