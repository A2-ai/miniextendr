# A computed NA is still R's NA.
#
# R decides "is this NA" for a double with `R_IsNA`: a NaN whose low 32-bit word
# is 1954. Arithmetic on `NA_real_` quiets the NaN, so `NA_real_ * 1` has other
# high bits than `NA_real_` itself while `is.na()` and `ISNA()` still say NA.
# Every conversion path that decides "is this NA" must agree; a plain `NaN` stays
# a value there. The R round trip alone cannot tell the two bit patterns apart,
# so each test compares bytes or reads the answer on the Rust side.

computed_na <- function() NA_real_ * 1

bytes_of <- function(x) writeBin(x, raw())

# The tests only discriminate when the platform really produces a different bit
# pattern for a computed NA.
skip_unless_computed_na_differs <- function() {
  x <- computed_na()
  if (!is.na(x) || is.nan(x) || identical(bytes_of(x), bytes_of(NA_real_))) {
    skip("NA_real_ * 1 has the same bits as NA_real_ on this platform")
  }
}

test_that("the computed NA fixture differs from NA_real_ but is NA", {
  skip_unless_computed_na_differs()
  expect_true(is.na(computed_na()))
  expect_false(is.nan(computed_na()))
})

test_that("Option<f64> and Vec<Option<f64>> read a computed NA as None", {
  skip_unless_computed_na_differs()
  expect_identical(conv_opt_f64_is_some(computed_na()), 0L)
  expect_identical(conv_opt_f64_is_some(NaN), 1L)

  # A value read as None comes back as canonical NA_real_ bytes; a value read
  # as Some(NaN) would keep the computed NA's bytes.
  out <- box_slice_option_f64_roundtrip(c(1, computed_na(), NaN))
  expect_identical(bytes_of(out[2]), bytes_of(NA_real_))
  expect_true(is.nan(out[3]))
})

test_that("DataFrameRow readers see a computed NA as None", {
  skip_unless_computed_na_differs()
  df <- df_option_scalar_rows()
  df$weight[1] <- computed_na()
  expect_true(is.na(df$weight[1]))
  # Row 2 is None in all four optional columns; row 1 now adds the weight.
  expect_identical(df_option_scalar_none_count(df), 5L)
  df$weight[1] <- NaN
  expect_identical(df_option_scalar_none_count(df), 4L)
})

test_that("ALTREP no_na counts NaN and computed NAs, as anyNA() requires", {
  skip_unless_computed_na_differs()
  expect_false(anyNA(vec_real_altrep_from(c(1, 2, 3))))
  expect_true(anyNA(vec_real_altrep_from(c(1, NaN, 3))))
  expect_true(anyNA(vec_real_altrep_from(c(1, computed_na(), 3))))
})

test_that("ALTREP sum / min / max return NA, not NaN, for a computed NA", {
  skip_unless_computed_na_differs()
  x <- vec_real_altrep_from(c(2, computed_na(), 1))
  for (f in list(sum, min, max)) {
    r <- f(x)
    expect_true(is.na(r))
    expect_false(is.nan(r))
  }
  expect_identical(sum(x, na.rm = TRUE), 3)
  expect_identical(min(x, na.rm = TRUE), 1)
  expect_identical(max(x, na.rm = TRUE), 2)

  # NA wins over NaN in either order, as in base R's min() / max().
  for (v in list(c(NaN, 1, computed_na()), c(computed_na(), NaN, 1))) {
    y <- vec_real_altrep_from(v)
    expect_identical(bytes_of(min(y)), bytes_of(min(v)))
    expect_identical(bytes_of(max(y)), bytes_of(max(v)))
  }
  nan_only <- vec_real_altrep_from(c(3, NaN, 1))
  expect_true(is.nan(min(nan_only)))
  expect_true(is.nan(max(nan_only)))
})

test_that("serde reads a computed NA as None", {
  skip_if_missing_feature("serde")
  skip_unless_computed_na_differs()
  # `None` comes back as canonical NA_real_ bytes; a missed NA would come back
  # as `Some(NaN)` with the computed NA's bytes.
  out <- serde_r_deserialize_option_f64(computed_na())
  expect_identical(bytes_of(out), bytes_of(NA_real_))
  expect_true(is.nan(serde_r_deserialize_option_f64(NaN)))
})

test_that("JSON conversion takes the NA branch for a computed NA", {
  skip_if_missing_feature("serde_json")
  skip_unless_computed_na_differs()
  # NA becomes JSON null by default; a plain NaN is an error.
  expect_identical(json_roundtrip(computed_na()), json_roundtrip(NA_real_))
  expect_error(json_roundtrip(NaN), "NaN")
})

test_that("Arrow nulls follow R_IsNA", {
  skip_unless_computed_na_differs()
  v <- c(1, computed_na(), NaN)
  expect_identical(arrow_na_f64_null_count(v), 1L)
  expect_identical(arrow_na_f64_null_positions(v), c(FALSE, TRUE, FALSE))
})

test_that("Arrow dates and times treat NaN as missing, as R does", {
  # R reads a NaN date or time as NA (`is.na(as.Date(NaN))`); casting it to
  # Arrow's integer storage would give the epoch instead.
  d <- arrow_date_roundtrip(as.Date(c(19000, NaN, NA)))
  expect_identical(is.na(d), c(FALSE, TRUE, TRUE))
  t <- arrow_posixct_roundtrip(.POSIXct(c(0, NaN, NA), tz = "UTC"))
  expect_identical(is.na(t), c(FALSE, TRUE, TRUE))
})

test_that("num-complex treats a computed NA part as NA", {
  skip_if_missing_feature("num-complex")
  skip_unless_computed_na_differs()
  expect_error(complex_roundtrip(complex(real = computed_na(), imaginary = 1)))
  expect_error(complex_roundtrip(complex(real = 1, imaginary = computed_na())))
  z <- complex(real = NaN, imaginary = 1)
  expect_true(is.nan(Re(complex_roundtrip(z))))
})
