test_that("foreign callbacks return normally and clean up Rust and C frames", {
  before <- foreign_callback_counts()
  expect_identical(foreign_callback_roundtrip(6L * 7L), 42L)
  expect_identical(foreign_callback_counts() - before, rep(1L, 4L))
})

test_that("foreign callbacks resume the original classed condition", {
  before <- foreign_callback_counts()
  condition <- errorCondition("foreign callback failed", class = "solver_error",
                              call = quote(solver()), detail = list(code = 42L))
  observed <- NULL
  caught <- tryCatch(
    withCallingHandlers(
      foreign_callback_roundtrip(stop(condition)),
      solver_error = function(c) observed <<- c
    ),
    error = identity
  )
  expect_identical(observed, condition)
  expect_identical(caught, condition)
  expect_identical(foreign_callback_counts() - before, rep(1L, 4L))
  expect_identical(foreign_callback_roundtrip(7L), 7L)
})

test_that("a restart crosses the foreign callback as a saved payload", {
  before <- foreign_callback_counts()
  value <- withRestarts(
    foreign_callback_roundtrip(invokeRestart("finish", list(result = 23L))),
    finish = identity
  )
  expect_identical(value, list(result = 23L))
  expect_identical(foreign_callback_counts() - before, rep(1L, 4L))
  expect_identical(foreign_callback_roundtrip(8L), 8L)
})

test_that("foreign callbacks preserve calling and exiting warning handlers", {
  before <- foreign_callback_counts()
  seen <- NULL
  value <- withCallingHandlers(
    foreign_callback_roundtrip({ warning("muffled callback"); 9L }),
    warning = function(c) {
      seen <<- conditionMessage(c)
      invokeRestart("muffleWarning")
    }
  )
  expect_identical(seen, "muffled callback")
  expect_identical(value, 9L)
  expect_identical(foreign_callback_counts() - before, rep(1L, 4L))

  before <- foreign_callback_counts()
  caught <- tryCatch(
    foreign_callback_roundtrip(warning("exiting callback")),
    warning = identity
  )
  expect_s3_class(caught, "warning")
  expect_identical(conditionMessage(caught), "exiting callback")
  expect_identical(foreign_callback_counts() - before, rep(1L, 4L))
})

test_that("foreign callback continuation values survive GC torture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  gc_stress_foreign_callback()
  condition <- tryCatch(
    foreign_callback_roundtrip(stop(errorCondition(
      "fresh condition", class = "fresh_solver_error", detail = list(value = 12L)
    ))),
    error = identity
  )
  expect_s3_class(condition, "fresh_solver_error")
  expect_identical(condition$detail, list(value = 12L))
  expect_identical(withRestarts(
    foreign_callback_roundtrip(invokeRestart("finish", list(value = 13L))),
    finish = identity
  ), list(value = 13L))
})
