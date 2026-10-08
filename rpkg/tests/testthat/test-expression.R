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
  # a regular R error (caught as an REvalError), never a longjmp/crash.
  expect_error(expr_eval_str('stop("boom")'), "boom")
})

test_that("a returned REvalError raises the caught message and specific classes", {
  e <- tryCatch(
    expr_eval_str('stop(errorCondition("bad thing", class = "my_class"))'),
    error = function(e) e
  )
  expect_identical(conditionMessage(e), "bad thing")
  expect_identical(
    class(e),
    c("my_class", "rust_error", "simpleError", "error", "condition")
  )
  # A plain stop() has no specific class: a bare rust_error.
  e <- tryCatch(expr_eval_str('stop("boom")'), error = function(e) e)
  expect_identical(conditionMessage(e), "boom")
  expect_identical(class(e), c("rust_error", "simpleError", "error", "condition"))
})

test_that("expr_eval_str surfaces parse failures as R errors", {
  expect_error(expr_eval_str("1 +"), "incomplete")
  expect_error(expr_eval_str("1 +)"), "syntax error")
})

test_that("RCall::eval returns a classed condition with its class and message", {
  check_input <- function(x) {
    stop(errorCondition("bad input", class = "my_input_error", call = sys.call()))
  }
  res <- miniextendr:::expr_catch_error(function() check_input(1L))
  expect_identical(res$message, "bad input")
  expect_identical(res$classes, c("my_input_error", "error", "condition"))
  expect_identical(res$specific_classes, "my_input_error")
  expect_identical(res$call, quote(check_input(1L)))
  expect_s3_class(res$condition, c("my_input_error", "error", "condition"), exact = TRUE)
  expect_identical(conditionMessage(res$condition), "bad input")
})

test_that("RCall::eval's message has no 'Error in' prefix and no 'Calls:' line", {
  # showErrorCalls = TRUE is what Rscript sets; R's printed error would read
  # "Error in g() : boom\nCalls: <anonymous> -> f -> g".
  withr::local_options(showErrorCalls = TRUE)
  f <- function() g()
  g <- function() stop("boom")
  res <- miniextendr:::expr_catch_error(function() f())
  expect_identical(res$message, "boom")
  expect_identical(res$classes, c("simpleError", "error", "condition"))
  expect_identical(res$specific_classes, character())
  expect_identical(res$call, quote(g()))
})

test_that("RCall::eval's message has no 'Calls:' line under Rscript", {
  skip_on_cran()
  skip_on_os("windows")
  skip_if_not_installed("callr")
  script <- withr::local_tempfile(fileext = ".R")
  writeLines(c(
    "f <- function() g()",
    "g <- function() stop(\"boom\")",
    "res <- miniextendr:::expr_catch_error(function() f())",
    "cat(res$message, deparse(res$call), sep = \"\\n\")"
  ), script)
  out <- callr::rscript(script, libpath = .libPaths(), show = FALSE)
  expect_identical(out$stdout, "boom\ng()\n")
})

test_that("a stop() at the top of the evaluated source has no call", {
  # R would give it the call of the tryCatch() frame the evaluation runs in.
  res <- miniextendr:::expr_catch_error_str('stop("top")')
  expect_identical(res$message, "top")
  expect_null(res$call)
  expect_identical(res$classes, c("simpleError", "error", "condition"))
})

test_that("a parse failure comes back as a simpleError without a call", {
  res <- miniextendr:::expr_catch_error_str("1 +)")
  expect_identical(res$message, "R syntax error while parsing: 1 +)")
  expect_identical(res$classes, c("simpleError", "error", "condition"))
  expect_null(res$call)
  expect_s3_class(res$condition, c("simpleError", "error", "condition"), exact = TRUE)
  expect_identical(conditionMessage(res$condition), res$message)
  expect_identical(miniextendr:::expr_catch_error_str("1L + 1L"), list(value = 2L))
})

test_that("RCall::eval hides the evaluation from the caller's handlers", {
  # warn = -1 keeps R's default warning handler quiet: the warning reaches
  # no handler of the caller's, so the default handler is all that sees it.
  withr::local_options(warn = -1)
  warns <- function() {
    warning("from the evaluated code")
    "value"
  }
  seen <- 0L
  res <- withCallingHandlers(
    miniextendr:::expr_catch_error(warns),
    warning = function(w) seen <<- seen + 1L
  )
  expect_identical(res, list(value = "value"))
  expect_identical(seen, 0L)

  res <- tryCatch(
    miniextendr:::expr_catch_error(warns),
    warning = function(w) "the caller's handler ran"
  )
  expect_identical(res, list(value = "value"))
})

test_that("an REvalError re-raised with reraise_class keeps the caught classes", {
  vctrs_like <- function() {
    stop(errorCondition("subscript out of bounds", class = c("vctrs_error_x", "rlang_error")))
  }
  e <- tryCatch(miniextendr:::expr_reraise(vctrs_like), error = function(e) e)
  expect_identical(conditionMessage(e), "subscript out of bounds")
  expect_identical(
    class(e),
    c(
      "mx_reraised", "vctrs_error_x", "rlang_error",
      "rust_error", "simpleError", "error", "condition"
    )
  )
  # A plain stop() keeps only the package's own class.
  e <- tryCatch(miniextendr:::expr_reraise(function() stop("plain")), error = function(e) e)
  expect_identical(conditionMessage(e), "plain")
  expect_identical(
    class(e),
    c("mx_reraised", "rust_error", "simpleError", "error", "condition")
  )
  expect_identical(miniextendr:::expr_reraise(function() 42L), 42L)
})

test_that("expr_call_builder builds sum(x, na.rm = TRUE) via RCall", {
  expect_identical(expr_call_builder(c(1, 2, NA, 4)), 7)
  expect_identical(expr_call_builder(numeric(0)), 0)
})

test_that("expr_call_builder propagates R evaluation errors", {
  # sum() on character input is an R-level error captured by eval()
  expect_error(expr_call_builder("not a number"))
})

test_that("RCall keeps freshly allocated inline arguments reachable", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  ok <- 0L
  for (i in seq_len(20L)) {
    if (identical(miniextendr:::expr_call_inline_arguments(), 1:8)) {
      ok <- ok + 1L
    }
  }
  gctorture(FALSE)
  expect_equal(ok, 20L)
})

test_that("expr_env_lookup resolves base-namespace bindings", {
  expect_true(expr_env_lookup("sum"))
  expect_false(expr_env_lookup("pi")) # resolves, but not a function
  expect_error(expr_env_lookup("no_such_symbol_xyz"))
})
