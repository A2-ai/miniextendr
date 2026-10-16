test_that("struct as_factor produces one factor column in every assembly shape", {
  expected_names <- list(
    named = c("id", "stage", "label"),
    tuple = c("stage", "_1"),
    nested = c("id", "inner_q", "level"),
    expanded = c("level", "values_1", "values_2"),
    tagged = c("kind", "level")
  )
  for (shape in names(expected_names)) {
    for (empty in c(FALSE, TRUE)) {
      df <- df_struct_factor(shape, empty)
      expected <- expected_names[[shape]]
      if (empty && shape == "expanded") expected <- "level"
      expect_identical(names(df), expected)
      expect_identical(nrow(df), if (empty) 0L else 3L)
      column <- if (shape %in% c("named", "tuple")) "stage" else "level"
      expect_identical(class(df[[column]]), "factor")
      expect_identical(levels(df[[column]]), c("Low", "High"))
      expect_identical(as.integer(df[[column]]), if (empty) integer() else c(1L, 2L, 1L))
    }
  }
})

test_that("a struct factor inside an enum variant stays a factor, NA where absent", {
  for (empty in c(FALSE, TRUE)) {
    df <- df_struct_factor("enum", empty)
    expect_identical(names(df), c("kind", "id", "step_id", "step_stage", "step_label"))
    expect_identical(class(df$step_stage), "factor")
    expect_identical(levels(df$step_stage), c("Low", "High"))
    expect_identical(as.integer(df$step_stage), if (empty) integer() else c(1L, 2L, NA))
  }
})

test_that("struct factor readers round-trip owned values and empty columns", {
  for (shape in c("named", "nested", "expanded", "enum")) {
    for (empty in c(FALSE, TRUE)) {
      df <- df_struct_factor(shape, empty)
      expect_identical(df_struct_factor_roundtrip(df, shape), df)
    }
  }
})

test_that("parallel struct factor readers need neither Clone nor Default", {
  skip_if_not(exists("df_struct_factor_roundtrip_par", mode = "function"),
              "rayon feature not compiled")
  for (shape in c("named", "nested", "expanded", "enum")) {
    for (empty in c(FALSE, TRUE)) {
      df <- df_struct_factor(shape, empty)
      expect_identical(df_struct_factor_roundtrip_par(df, shape), df)
    }
  }
})

test_that("struct factor readers validate levels, type, length and all invalid codes", {
  df <- df_struct_factor("named", FALSE)
  bad <- df
  bad$stage <- as.character(bad$stage)
  expect_error(df_struct_factor_roundtrip(bad, "named"), "column `stage`.*expected a factor")
  bad$stage <- factor(df$stage, levels = c("High", "Low"))
  expect_error(df_struct_factor_roundtrip(bad, "named"), "level 1: expected 'Low', got 'High'")
  bad$stage <- NULL
  expect_error(df_struct_factor_roundtrip(bad, "named"), "column `stage` is missing")
  bad <- unclass(df)
  bad$stage <- df$stage[1:2]
  class(bad) <- "data.frame"
  expect_error(df_struct_factor_roundtrip(bad, "named"), "column `stage` has length 2.*3 rows")
  bad <- df
  bad$stage <- structure(c(NA_integer_, 0L, 3L), levels = c("Low", "High"), class = "factor")
  err <- tryCatch(df_struct_factor_roundtrip(bad, "named"), error = identity)
  expect_s3_class(err, "error")
  msg <- conditionMessage(err)
  expect_match(msg, "NA is not allowed")
  expect_match(msg, "code 0 is not a level")
  expect_match(msg, "code 3 is not a level")
})

test_that("struct factor construction and readers retain roots under GC pressure", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  expect_null(gc_stress_struct_factor())
})
