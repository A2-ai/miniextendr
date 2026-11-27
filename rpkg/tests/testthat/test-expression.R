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

test_that("a returned REvalError raises R's condition object as it is", {
  e <- tryCatch(
    expr_eval_str('stop(errorCondition("bad thing", class = "my_class", data = 1:2))'),
    error = function(e) e
  )
  expect_identical(conditionMessage(e), "bad thing")
  expect_identical(class(e), c("my_class", "error", "condition"))
  expect_identical(e$data, 1:2)
  # A stop() at the top of the evaluated source: R's simpleError, without the
  # call of the tryCatch() frame the evaluation ran in.
  e <- tryCatch(expr_eval_str('stop("boom")'), error = function(e) e)
  expect_identical(conditionMessage(e), "boom")
  expect_identical(class(e), c("simpleError", "error", "condition"))
  expect_null(conditionCall(e))
})

test_that("a returned REvalError keeps R's fields and call, after the queued warning", {
  skip_if_not_installed("vctrs")
  seen <- character()
  e <- withCallingHandlers(
    tryCatch(
      miniextendr:::expr_return_err(function() vctrs::vec_slice(1:3, 5)),
      error = function(e) {
        seen <<- c(seen, "error")
        e
      }
    ),
    warning = function(w) {
      seen <<- c(seen, conditionMessage(w))
      invokeRestart("muffleWarning")
    }
  )
  expect_identical(seen, c("queued before the error", "error"))
  expect_s3_class(e, "vctrs_error_subscript_oob")
  expect_false(inherits(e, "rust_error"))
  expect_identical(e$i, 5L)
  expect_identical(conditionCall(e), quote(vctrs::vec_slice(1:3, 5)))
  expect_identical(suppressWarnings(miniextendr:::expr_return_err(function() 42L)), 42L)
})

test_that("a returned REvalError keeps a class's own conditionMessage() method", {
  # The message is computed from a field, by a method for the class: only the
  # object R raised has the field.
  conditionMessage.mx_lazy_error <- function(c) paste("detail: row", c$row)
  lazy <- function() {
    stop(structure(
      class = c("mx_lazy_error", "error", "condition"),
      list(message = "lazy", call = NULL, row = 3L)
    ))
  }
  msg <- suppressWarnings(tryCatch(
    miniextendr:::expr_return_err(lazy),
    mx_lazy_error = function(e) conditionMessage(e)
  ))
  expect_identical(msg, "detail: row 3")
})

test_that("an REvalError Rust built raises its message as the function's error", {
  # A parse failure raised no R condition: the queued warning, then
  # rust_error with the message and the function's call.
  seen <- character()
  e <- withCallingHandlers(
    tryCatch(miniextendr:::expr_return_err_str("1 +)"), error = function(e) {
      seen <<- c(seen, "error")
      e
    }),
    warning = function(w) {
      seen <<- c(seen, conditionMessage(w))
      invokeRestart("muffleWarning")
    }
  )
  expect_identical(seen, c("queued before the error", "error"))
  expect_identical(conditionMessage(e), "R syntax error while parsing: 1 +)")
  expect_identical(class(e), c("rust_error", "simpleError", "error", "condition"))
  expect_identical(conditionCall(e), quote(miniextendr:::expr_return_err_str("1 +)")))
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

test_that("an REvalError raised as the package's own error takes R's message", {
  vctrs_like <- function() {
    stop(errorCondition("subscript out of bounds", class = c("vctrs_error_x", "rlang_error")))
  }
  e <- tryCatch(miniextendr:::expr_raise_own(vctrs_like), error = function(e) e)
  expect_identical(conditionMessage(e), "subscript out of bounds")
  expect_identical(class(e), c("mx_reraised", "rust_error", "simpleError", "error", "condition"))
  expect_identical(conditionCall(e), quote(miniextendr:::expr_raise_own(vctrs_like)))
  expect_identical(miniextendr:::expr_raise_own(function() 42L), 42L)
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

test_that("a returned REvalError keeps R's condition reachable under gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  ok <- 0L
  for (i in seq_len(5L)) {
    e <- tryCatch(
      expr_eval_str('stop(errorCondition("gc", class = "mx_gc", data = 1:3))'),
      error = identity
    )
    cond <- miniextendr:::gc_stress_reval_error_object()
    if (identical(class(e), c("mx_gc", "error", "condition")) && identical(e$data, 1:3) &&
        is.null(conditionCall(e)) && identical(conditionMessage(cond), "gc") &&
        is.null(conditionCall(cond))) {
      ok <- ok + 1L
    }
  }
  gctorture(FALSE)
  expect_equal(ok, 5L)
})

test_that("expr_env_lookup resolves base-namespace bindings", {
  expect_true(expr_env_lookup("sum"))
  expect_false(expr_env_lookup("pi")) # resolves, but not a function
  expect_error(expr_env_lookup("no_such_symbol_xyz"))
})

test_that("RSymbol::new interns the symbol R's as.name() gives", {
  expect_identical(expr_symbol("x"), quote(x))
  expect_true(is.symbol(expr_symbol("x")))
  # Interned: the same name is the same object.
  expect_identical(expr_symbol("x"), expr_symbol("x"))
  # A name that isn't syntactic, and one that isn't ASCII.
  expect_identical(expr_symbol("my var"), as.name("my var"))
  expect_identical(expr_symbol("café"), as.name("café"))
  # R refuses an empty name with its own error.
  expect_error(expr_symbol(""), "zero-length variable name")
})

test_that("an RSymbol call argument is a variable reference", {
  env <- new.env()
  assign("answer", 42L, envir = env)
  assign("my var", "spaced", envir = env)
  expect_identical(expr_symbol_value(env, "answer"), 42L)
  expect_identical(expr_symbol_value(env, "my var"), "spaced")
  # Looked up from `env` outward, as R evaluates a symbol.
  expect_identical(expr_symbol_value(env, "pi"), pi)
  expect_error(expr_symbol_value(env, "no_such_variable_xyz"), "not found")
})
