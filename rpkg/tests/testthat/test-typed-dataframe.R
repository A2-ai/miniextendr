# =============================================================================
# typed_dataframe! macro tests (#698)
# =============================================================================

theoph_full <- function(n = 6L) {
  data.frame(
    subject = seq_len(n),
    weight = 60 + as.numeric(seq_len(n)),
    dose = rep(320.0, n),
    time = (seq_len(n) - 1L) * 0.5,
    conc = 1.0 + (seq_len(n) - 1L) * 0.1
  )
}

theoph_with_flag <- function(n = 6L) {
  df <- theoph_full(n)
  df$flag <- as.integer(seq_len(n) %% 2L)
  df
}

test_that("typed_df_theoph_nrow returns row count for a valid data.frame", {
  df <- theoph_full(8L)
  expect_equal(typed_df_theoph_nrow(df), 8L)

  df0 <- theoph_full(0L)
  expect_equal(typed_df_theoph_nrow(df0), 0L)
})

test_that("typed_df_theoph_sum_conc reads borrowed slice correctly", {
  df <- theoph_full(5L)
  expect_equal(
    typed_df_theoph_sum_conc(df),
    sum(df$conc),
    tolerance = 1e-12
  )
})

test_that("typed_df_theoph_has_flag respects optional column presence", {
  expect_false(typed_df_theoph_has_flag(theoph_full(3L)))
  expect_true(typed_df_theoph_has_flag(theoph_with_flag(3L)))
})

test_that("typed_df_theoph_flag_sum returns -1 when absent, sum when present", {
  expect_equal(typed_df_theoph_flag_sum(theoph_full(4L)), -1L)
  df <- theoph_with_flag(6L)
  expect_equal(typed_df_theoph_flag_sum(df), as.integer(sum(df$flag)))
})

test_that("typed_dataframe! rejects non-data.frame inputs", {
  expect_error(
    typed_df_theoph_nrow(list(subject = 1L, weight = 60)),
    "expected a data.frame"
  )
  expect_error(
    typed_df_theoph_nrow(1:10),
    "expected a data.frame"
  )
})

test_that("typed_dataframe! batches multiple missing-column errors", {
  # Missing `dose`, `time`, and `conc` — batched into one error.
  df_bad <- data.frame(
    subject = 1:3,
    weight = c(60, 65, 70)
  )
  err <- expect_error(typed_df_theoph_nrow(df_bad))
  msg <- conditionMessage(err)
  expect_match(msg, "missing required column `dose`")
  expect_match(msg, "missing required column `time`")
  expect_match(msg, "missing required column `conc`")
})

test_that("typed_dataframe! reports wrong column types", {
  df <- theoph_full(3L)
  # subject should be integer but make it character
  df$subject <- as.character(df$subject)
  err <- expect_error(typed_df_theoph_nrow(df))
  expect_match(conditionMessage(err), "column `subject`")
})

test_that("typed_dataframe! reports wrong type for optional column", {
  df <- theoph_full(3L)
  df$flag <- c("a", "b", "c") # should be i32 / INTSXP
  err <- expect_error(typed_df_theoph_has_flag(df))
  expect_match(conditionMessage(err), "column `flag`")
})

test_that("typed_dataframe! tolerates extra columns by default", {
  df <- theoph_full(4L)
  df$extra_col <- letters[1:4]
  expect_equal(typed_df_theoph_nrow(df), 4L)
})

test_that("typed_dataframe! @exact; rejects extra columns", {
  df_ok <- data.frame(x = 1:5)
  expect_equal(typed_df_strict_sum_x(df_ok), as.integer(sum(df_ok$x)))

  df_extra <- data.frame(x = 1:5, y = 6:10)
  err <- expect_error(typed_df_strict_sum_x(df_extra))
  expect_match(conditionMessage(err), "extra columns")
  expect_match(conditionMessage(err), "y")
})

test_that("tibbles work (inherit from data.frame)", {
  skip_if_not_installed("tibble")
  tb <- tibble::tibble(
    subject = 1:4,
    weight = c(60, 70, 80, 90),
    dose = rep(160.0, 4L),
    time = c(0, 0.5, 1, 1.5),
    conc = c(1.0, 2.0, 3.0, 4.0)
  )
  expect_equal(typed_df_theoph_nrow(tb), 4L)
  expect_equal(typed_df_theoph_sum_conc(tb), 10.0, tolerance = 1e-12)
})

test_that("gc_stress_typed_dataframe runs without error", {
  # Smoke test — the gctorture nightly sweep exercises this under
  # gctorture(TRUE).
  expect_null(gc_stress_typed_dataframe())
})
