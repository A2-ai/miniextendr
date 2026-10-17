fallback_objects <- function() {
  list(s3 = new_fallbacks3(), s4 = FallbackS4(), env = FallbackEnv$new())
}
fallback_caught <- function(expr) tryCatch(expr, error = identity)

test_that("fallbacks keep field precedence and distinguish None from NULL", {
  for (x in fallback_objects()) {
    expect_identical(x$id, 7L)
    expect_identical(x[["id"]], 7L)
    expect_identical(x$column, 7:8)
    expect_identical(x[["column"]], 7:8)
    expect_null(x$null)
    expect_null(x[["null"]])
    absent_value <- x$missing
    expect_true(missing(absent_value))
    absent_value <- x[["missing"]]
    expect_true(missing(absent_value))
    for (read in alist(x$absent, x[["absent"]], x$col, x[["col", exact = FALSE]])) {
      e <- fallback_caught(eval(read))
      expect_s3_class(e, "miniextendr_no_field")
      expect_s3_class(e, "fallback_field_error")
      expect_match(conditionMessage(e), "No column named '", fixed = TRUE)
      expect_identical(conditionCall(e), read)
    }
  }
})

test_that("positions and collection readers contain fields only", {
  for (x in fallback_objects()) {
    expect_identical(x[[1L]], 7L)
    expect_identical(names(x), "id")
    expect_identical(as.list(x), list(id = 7L))
    for (i in list(0L, 2L, -1L)) {
      expect_s3_class(fallback_caught(x[[i]]), "subscriptOutOfBoundsError")
    }
    for (i in list(1.5, NA_integer_, c("id", "column"), NA_character_)) {
      expect_s3_class(fallback_caught(x[[i]]), "notSubsettableError")
    }
  }
})

test_that("fallback completion filters fields followed by columns", {
  objects <- fallback_objects()
  for (x in objects[c("s3", "s4")]) {
    expect_identical(utils::.DollarNames(x, ""), c("id", "column", "null", "ping"))
    expect_identical(utils::.DollarNames(x, "^(id|col)"), c("id", "column"))
  }
  x <- objects$env
  expect_identical(utils::.DollarNames(x, "^(id|col)"), c("id", "column"))
  expect_identical(x$ping(), 71L)
  expect_identical(x[["ping"]](), 71L)
})

test_that("fallbacks never create writable fields", {
  for (x in fallback_objects()) {
    expect_s3_class(fallback_caught(x$column <- 9L), "miniextendr_no_field")
    expect_s3_class(fallback_caught(x[["column"]] <- 9L), "miniextendr_no_field")
    x$id <- 8L
    expect_identical(x$column, 8:9)
    x[["id"]] <- 9L
    expect_identical(x$column, 9:10)
  }
})

test_that("fallbacks refuse restored values before invoking user functions", {
  for (x in fallback_objects()) {
    x <- unserialize(serialize(x, NULL))
    for (read in alist(x$column, x[["column"]], x$absent, x[[2L]], names(x), as.list(x))) {
      expect_s3_class(fallback_caught(eval(read)), "fallback_saved")
    }
    expect_identical(utils::.DollarNames(x, ""), character())
  }
})

test_that("omitted completion and scalar Option do not invent NA values", {
  x <- new_fallbackscalar()
  expect_identical(x$scalar, 3L)
  expect_identical(x[["scalar"]], 3L)
  expect_s3_class(fallback_caught(x$absent), "miniextendr_no_field")
  expect_s3_class(fallback_caught(x[["absent"]]), "miniextendr_no_field")
  expect_identical(utils::.DollarNames(x, ""), "id")
  # The opt-in fallback does not change lookup=list's numeric/NA semantics.
  expect_identical(x[[1.5]], 3L)
  expect_null(x[[NA]])
  expect_null(x[[NA_character_]])
  x <- unserialize(serialize(x, NULL))
  expect_s3_class(fallback_caught(x$scalar), "miniextendr_restored")
  expect_identical(utils::.DollarNames(x, ""), character())
})

test_that("fallback failures release their borrow and carry the user's call", {
  for (x in fallback_objects()) {
    e <- fallback_caught(x$panic)
    expect_match(conditionMessage(e), "fallback panic", fixed = TRUE)
    expect_identical(conditionCall(e), quote(x$panic))
    expect_identical(x$column, 7:8)
    expect_warning(expect_identical(x$warned, 7L), class = "fallback_warning")
    expect_warning(
      expect_s3_class(fallback_caught(x$absent_warned), "miniextendr_no_field"),
      class = "fallback_warning"
    )
    x$id <- 10L
    expect_identical(x$column, 10:11)
  }
})

test_that("fallback return conversion and completion survive GC torture", {
  skip_gc_stress_if_disabled()
  # Torture the accessors, conversions and condition handling, then assert
  # outside torture so testthat's own reporting allocations do not dominate.
  results <- local({
    gctorture(TRUE)
    on.exit(gctorture(FALSE), add = TRUE)
    gc_stress_field_fallbacks()
    lapply(fallback_objects(), function(x) {
      warning <- NULL
      warned <- withCallingHandlers(x$warned, warning = function(cnd) {
        warning <<- cnd
        invokeRestart("muffleWarning")
      })
      list(
        column = x$column,
        null = x$null,
        absent = fallback_caught(x$absent),
        completions = utils::.DollarNames(x, "^col"),
        warned = warned,
        warning = warning
      )
    })
  })
  for (result in results) {
    expect_identical(result$column, 7:8)
    expect_null(result$null)
    expect_s3_class(result$absent, "miniextendr_no_field")
    expect_identical(result$completions, "column")
    expect_identical(result$warned, 7L)
    expect_s3_class(result$warning, "fallback_warning")
  }
})
