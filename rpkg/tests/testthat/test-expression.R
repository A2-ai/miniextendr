# Expression subsystem fixtures (RCall, REnv, r_eval_str)

test_that("expr_eval_str evaluates R source and returns the last value", {
  expect_identical(expr_eval_str("1 + 1"), 2)
  expect_identical(expr_eval_str("'a'"), "a")
  # Multiple top-level expressions: all evaluated, last value returned.
  # local() keeps the intermediate assignment out of the global environment.
  expect_identical(expr_eval_str("local({ x <- 2; x * 3 })"), 6)
})

test_that("expr_eval_str returns NULL for empty input", {
  expect_null(expr_eval_str(""))
  expect_null(expr_eval_str("   \n  "))
})

test_that("expr_eval_str surfaces R evaluation errors as R errors, not crashes", {
  # The load-bearing test: stop() inside the evaluated code must come back as
  # a regular R error (R_tryEvalSilent capture), never a longjmp/crash.
  expect_error(expr_eval_str('stop("boom")'), "boom")
})

test_that("expr_eval_str surfaces parse failures as R errors", {
  expect_error(expr_eval_str("1 +"), "incomplete")
  expect_error(expr_eval_str("1 +)"), "syntax error")
})

test_that("expr_call_builder builds sum(x, na.rm = TRUE) via RCall", {
  expect_identical(expr_call_builder(c(1, 2, NA, 4)), 7)
  expect_identical(expr_call_builder(numeric(0)), 0)
})

test_that("expr_call_builder propagates R evaluation errors", {
  # sum() on character input is an R-level error captured by eval()
  expect_error(expr_call_builder("not a number"))
})

test_that("expr_env_lookup resolves base-namespace bindings", {
  expect_true(expr_env_lookup("sum"))
  expect_false(expr_env_lookup("pi")) # resolves, but not a function
  expect_error(expr_env_lookup("no_such_symbol_xyz"))
})
