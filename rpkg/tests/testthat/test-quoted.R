# Arguments passed unevaluated (`Quoted`, `Quosure`) and R code evaluated from
# Rust in the caller's R context (`eval_with_handlers`, #1835):
# src/rust/quoted_tests.rs.
#
# Each evaluating fixture holds a Rust value whose destructor counts its runs
# (`quoted_sentinel_drops()`), so an R exit that leaves through the Rust frames
# can be seen to drop them.

drops <- function() miniextendr:::quoted_sentinel_drops()

tbl <- data.frame(a = 1:5, b = c(2, 4, 6, 8, 10))

# region: Quoted

test_that("a Quoted argument is not evaluated and evaluates in the caller's frame", {
  x <- 10
  expect_identical(miniextendr:::quoted_eval(x + 1), 11)
  f <- function() {
    y <- 5
    miniextendr:::quoted_eval(y * 2)
  }
  expect_identical(f(), 10)
  # Never forced by the wrapper: `stop()` here would raise if it were.
  expect_identical(miniextendr:::quoted_expr(stop("not forced")), quote(stop("not forced")))
  expect_identical(miniextendr:::quoted_env(anything), environment())
  g <- function() miniextendr:::quoted_env(anything)
  env <- g()
  expect_false(identical(env, environment()))
  expect_true(is.environment(env))
})

test_that("eval_in() puts the columns in front of the caller's variables", {
  limit <- 5
  expect_identical(miniextendr:::quoted_rows(tbl, b > limit), 3:5)
  # A column shadows a caller variable of the same name.
  a <- 100
  expect_identical(miniextendr:::quoted_rows(tbl, a > 3), 4:5)
  # NA counts as FALSE.
  expect_identical(miniextendr:::quoted_rows(tbl, ifelse(a == 2, NA, a < 4)), c(1L, 3L))
  # An environment is used as is; NULL evaluates in the caller's frame.
  expect_identical(miniextendr:::quoted_rows(list2env(list(b = 1:3)), b > 1L), 2:3)
  keep <- c(TRUE, FALSE)
  expect_identical(miniextendr:::quoted_rows(NULL, keep), 1L)
  e <- tryCatch(miniextendr:::quoted_rows(1:3, a), error = identity)
  expect_match(
    conditionMessage(e),
    "`data` must be a list, a data frame, an environment or NULL, not integer",
    fixed = TRUE
  )
})

test_that("a standalone S3 subset() method takes its predicate unevaluated", {
  t <- structure(tbl, class = c("mx_quoted_tbl", "data.frame"))
  limit <- 5
  out <- subset(t, b > limit & a < 5)
  expect_s3_class(out, "mx_quoted_tbl")
  expect_identical(out$a, 3:4)
  # Through a function: the predicate's variables come from that frame.
  above <- function(x, lim) subset(x, b > lim)
  expect_identical(above(t, 7)$a, 4:5)
})

test_that("as_name() reads a symbol or a string, and nothing else", {
  expect_identical(miniextendr:::quoted_name(TIME), "TIME")
  expect_identical(miniextendr:::quoted_name("TIME"), "TIME")
  expect_identical(miniextendr:::quoted_name(f(TIME)), NA_character_)
  expect_identical(miniextendr:::quoted_name(1), NA_character_)
  expect_identical(miniextendr:::quoted_name(c("a", "b")), NA_character_)
})

test_that("an omitted Quoted argument is a conversion error; Missing<Quoted> gets Absent", {
  e <- tryCatch(miniextendr:::quoted_eval(), error = identity)
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "expr")
  expect_identical(conditionMessage(e), "argument \"expr\" is missing, with no default")
  expect_equal(conditionCall(e), quote(miniextendr:::quoted_eval()))

  expect_identical(miniextendr:::quoted_optional(), "absent")
  expect_identical(miniextendr:::quoted_optional(NULL), "NULL")
  expect_identical(miniextendr:::quoted_optional(TIME), "symbol")
  expect_identical(miniextendr:::quoted_optional(a + b), "language")
  # A missing argument forwarded from another function is still missing.
  fwd <- function(x) miniextendr:::quoted_optional(x)
  expect_identical(fwd(), "absent")
  expect_identical(fwd(TIME), "symbol")
})

test_that("the caller's calling handlers see a warning once, and can muffle it", {
  seen <- 0L
  value <- withCallingHandlers(
    miniextendr:::quoted_eval({
      warning("from the expression")
      42
    }),
    warning = function(w) {
      seen <<- seen + 1L
      expect_identical(conditionMessage(w), "from the expression")
      invokeRestart("muffleWarning")
    }
  )
  expect_identical(value, 42)
  expect_identical(seen, 1L)

  expect_no_warning(
    expect_identical(suppressWarnings(miniextendr:::quoted_eval({
      warning("silenced")
      43
    })), 43)
  )

  heard <- 0L
  withCallingHandlers(
    miniextendr:::quoted_eval(message("hello")),
    message = function(m) {
      heard <<- heard + 1L
      invokeRestart("muffleMessage")
    }
  )
  expect_identical(heard, 1L)
})

test_that("an R error reaches tryCatch() as raised, after the Rust frames drop", {
  before <- drops()
  e <- tryCatch(
    miniextendr:::quoted_eval(stop(errorCondition(
      "boom",
      class = "mx_custom_error",
      call = quote(my_caller(x))
    ))),
    error = identity
  )
  expect_identical(class(e), c("mx_custom_error", "error", "condition"))
  expect_identical(conditionMessage(e), "boom")
  expect_equal(conditionCall(e), quote(my_caller(x)))
  expect_identical(drops() - before, 1L)

  # A plain stop() names the frame it was evaluated from, as a forced
  # promise would.
  e <- tryCatch(miniextendr:::quoted_eval(stop("plain")), error = identity)
  expect_s3_class(e, "simpleError")
  expect_equal(conditionCall(e), quote(miniextendr:::quoted_eval(stop("plain"))))

  # Through eval_in()'s data mask too.
  before <- drops()
  e <- tryCatch(miniextendr:::quoted_rows(tbl, stop("in the mask")), error = identity)
  expect_identical(conditionMessage(e), "in the mask")
  expect_identical(drops() - before, 1L)
})

test_that("an exiting handler and a restart leave through the Rust frames too", {
  before <- drops()
  w <- tryCatch(
    miniextendr:::quoted_eval({
      warning("exit")
      1
    }),
    warning = identity
  )
  expect_s3_class(w, "simpleWarning")
  expect_identical(conditionMessage(w), "exit")
  expect_identical(drops() - before, 1L)

  before <- drops()
  out <- withRestarts(
    miniextendr:::quoted_eval(invokeRestart("mx_restart", 5)),
    mx_restart = function(v) v * 2
  )
  expect_identical(out, 10)
  expect_identical(drops() - before, 1L)
})

test_that("repeated R exits leave the session usable", {
  before <- drops()
  for (i in seq_len(200L)) {
    tryCatch(miniextendr:::quoted_eval(stop("again")), error = identity)
    tryCatch(miniextendr:::quoted_eval(warning("again")), warning = identity)
  }
  expect_identical(drops() - before, 400L)
  x <- 1
  expect_identical(miniextendr:::quoted_eval(x + 1), 2)
})

# endregion

# region: Quosure

test_that("a Quosure argument carries its expression and environment", {
  skip_if_not_installed("rlang")
  q <- miniextendr:::quosure_sexp(a + b)
  expect_true(rlang::is_quosure(q))
  expect_identical(miniextendr:::quosure_expr(a + b), quote(a + b))
  expect_identical(miniextendr:::quosure_env(a + b), environment())
  expect_identical(miniextendr:::quosure_expr(stop("not forced")), quote(stop("not forced")))
})

test_that("eval_tidy(): the column wins over a caller variable of the same name", {
  skip_if_not_installed("rlang")
  a <- 100
  expect_identical(miniextendr:::quosure_eval_tidy(tbl, a), 1:5)
  expect_identical(miniextendr:::quosure_eval_tidy(tbl, a * 2), c(2, 4, 6, 8, 10))
  # The pronouns pick a side.
  expect_identical(miniextendr:::quosure_eval_tidy(tbl, .env$a), 100)
  expect_identical(miniextendr:::quosure_eval_tidy(tbl, .data$a), 1:5)
})

test_that("{{ }} and !! forward an argument from a user-level function", {
  skip_if_not_installed("rlang")
  pull_col <- function(data, col) miniextendr:::quosure_eval_tidy(data, {{ col }})
  a <- 100
  expect_identical(pull_col(tbl, b), c(2, 4, 6, 8, 10))
  # The column still wins through the forwarding.
  expect_identical(pull_col(tbl, a), 1:5)
  # Captured in the user function's caller's frame, not the user function's.
  scaled <- function(data, col) {
    k <- 1000
    pull_col(data, {{ col }})
  }
  k <- 3
  expect_identical(scaled(tbl, a * k), c(3, 6, 9, 12, 15))

  nm <- rlang::sym("b")
  expect_identical(miniextendr:::quosure_eval_tidy(tbl, !!nm), c(2, 4, 6, 8, 10))
  name_of <- function(col) miniextendr:::quosure_name({{ col }})
  expect_identical(name_of(TIME), "TIME")
  expect_identical(miniextendr:::quosure_name(!!nm), "b")
})

test_that("Quosure as_name() reads a symbol or a string, and nothing else", {
  skip_if_not_installed("rlang")
  expect_identical(miniextendr:::quosure_name(TIME), "TIME")
  expect_identical(miniextendr:::quosure_name("TIME"), "TIME")
  expect_identical(miniextendr:::quosure_name(f(TIME)), NA_character_)
})

test_that("an omitted Quosure argument is a conversion error; Missing<Quosure> gets Absent", {
  skip_if_not_installed("rlang")
  e <- tryCatch(miniextendr:::quosure_name(), error = identity)
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "x")
  expect_identical(conditionMessage(e), "argument \"x\" is missing, with no default")

  expect_identical(miniextendr:::quosure_optional(), "absent")
  expect_identical(miniextendr:::quosure_optional(NULL), "NULL")
  expect_identical(miniextendr:::quosure_optional(TIME), "symbol")
  # `{{ col }}` with `col` missing is missing too.
  fwd <- function(col) miniextendr:::quosure_optional({{ col }})
  expect_identical(fwd(), "absent")
  expect_identical(fwd(TIME), "symbol")
  bare <- function(col) miniextendr:::quosure_name({{ col }})
  expect_identical(tryCatch(bare(), error = function(e) e$kind), "conversion")
})

test_that("eval_tidy() conditions reach the caller's handlers", {
  skip_if_not_installed("rlang")
  seen <- 0L
  out <- withCallingHandlers(
    miniextendr:::quosure_eval_tidy(tbl, {
      warning("tidy warning")
      a
    }),
    warning = function(w) {
      seen <<- seen + 1L
      invokeRestart("muffleWarning")
    }
  )
  expect_identical(out, 1:5)
  expect_identical(seen, 1L)

  before <- drops()
  e <- tryCatch(
    miniextendr:::quosure_eval_tidy(tbl, rlang::abort("tidy error", class = "mx_tidy_error")),
    error = identity
  )
  expect_s3_class(e, "mx_tidy_error")
  expect_identical(drops() - before, 1L)
})

# endregion

# region: calls built in Rust (RCall::eval_with_handlers)

test_that("named arguments reach the function, a call passes as a call", {
  expect_identical(miniextendr:::quoted_call_ns("base", "paste", list("a", "b", sep = "-")), "a-b")
  expect_identical(
    miniextendr:::quoted_call_ns("base", "identity", list(quote(f(x)))),
    quote(f(x))
  )
})

test_that("a deprecation-style warning from a built call keeps its class and call", {
  cond <- warningCondition(
    "`old()` is deprecated",
    class = "lifecycle_warning_deprecated",
    call = quote(old(x))
  )
  seen <- list()
  withCallingHandlers(
    miniextendr:::quoted_call_ns("base", "warning", list(cond)),
    warning = function(w) {
      seen[[length(seen) + 1L]] <<- w
      invokeRestart("muffleWarning")
    }
  )
  expect_length(seen, 1L)
  expect_s3_class(seen[[1]], "lifecycle_warning_deprecated")
  expect_equal(conditionCall(seen[[1]]), quote(old(x)))
  expect_no_warning(suppressWarnings(miniextendr:::quoted_call_ns("base", "warning", list(cond))))
})

test_that("a classed error from a built call keeps its class and call", {
  before <- drops()
  e <- tryCatch(
    miniextendr:::quoted_call_ns("base", "stop", list(errorCondition(
      "bad input",
      class = "mx_custom_error",
      call = quote(my_caller(x))
    ))),
    error = identity
  )
  expect_identical(class(e), c("mx_custom_error", "error", "condition"))
  expect_equal(conditionCall(e), quote(my_caller(x)))
  expect_identical(drops() - before, 1L)
})

test_that("rlang's warn() and abort() through a built call with named arguments", {
  skip_if_not_installed("rlang")
  seen <- 0L
  withCallingHandlers(
    miniextendr:::quoted_call_ns("rlang", "warn", list(
      message = "`old()` is deprecated",
      class = "lifecycle_warning_deprecated"
    )),
    warning = function(w) {
      seen <<- seen + 1L
      expect_s3_class(w, "lifecycle_warning_deprecated")
      invokeRestart("muffleWarning")
    }
  )
  expect_identical(seen, 1L)

  e <- tryCatch(
    miniextendr:::quoted_call_ns("rlang", "abort", list(
      message = "bad input",
      class = "mx_custom_error",
      call = quote(my_caller(x))
    )),
    error = identity
  )
  expect_s3_class(e, "mx_custom_error")
  expect_s3_class(e, "rlang_error")
  expect_equal(conditionCall(e), quote(my_caller(x)))
})

test_that("rlang::eval_tidy() through a built call: column wins, conditions intact", {
  skip_if_not_installed("rlang")
  a <- 100
  expect_identical(
    miniextendr:::quoted_call_ns("rlang", "eval_tidy", list(expr = rlang::quo(a), data = tbl)),
    1:5
  )
  expect_identical(
    miniextendr:::quoted_call_ns("rlang", "eval_tidy", list(expr = rlang::quo(.env$a), data = tbl)),
    100
  )
  before <- drops()
  e <- tryCatch(
    miniextendr:::quoted_call_ns("rlang", "eval_tidy", list(
      expr = rlang::quo(rlang::abort("in the quosure", class = "mx_tidy_error", call = quote(tidy_caller()))),
      data = tbl
    )),
    error = identity
  )
  expect_s3_class(e, "mx_tidy_error")
  expect_equal(conditionCall(e), quote(tidy_caller()))
  expect_identical(drops() - before, 1L)
})

test_that("tidyselect::eval_select() through a built call, error_call the wrapper's call", {
  skip_if_not_installed("rlang")
  skip_if_not_installed("tidyselect")
  expect_identical(miniextendr:::quosure_select(tbl, c(b, a)), c(b = 2L, a = 1L))
  expect_length(miniextendr:::quosure_select(tbl, tidyselect::starts_with("z")), 0L)
  cols_of <- function(data, cols) miniextendr:::quosure_select(data, {{ cols }})
  expect_identical(cols_of(tbl, a), c(a = 1L))

  before <- drops()
  e <- tryCatch(miniextendr:::quosure_select(tbl, c(a, zz)), error = identity)
  expect_s3_class(e, "vctrs_error_subscript_oob")
  expect_equal(conditionCall(e), quote(miniextendr:::quosure_select(tbl, c(a, zz))))
  expect_identical(drops() - before, 1L)

  # allow_rename = FALSE reached eval_select().
  e <- tryCatch(miniextendr:::quosure_select(tbl, c(new = a)), error = identity)
  expect_s3_class(e, "rlang_error")
})

test_that("with_r_thread from a worker body keeps the caller's handlers", {
  seen <- 0L
  out <- withCallingHandlers(
    miniextendr:::quoted_worker_call("base", "warning", "from the worker"),
    warning = function(w) {
      seen <<- seen + 1L
      invokeRestart("muffleWarning")
    }
  )
  expect_identical(out, "from the worker")
  expect_identical(seen, 1L)

  e <- tryCatch(miniextendr:::quoted_worker_call("base", "stop", "worker error"), error = identity)
  expect_s3_class(e, "simpleError")
  expect_identical(conditionMessage(e), "worker error")
  w <- tryCatch(miniextendr:::quoted_worker_call("base", "warning", "exit"), warning = identity)
  expect_s3_class(w, "simpleWarning")
  expect_identical(conditionMessage(w), "exit")
  # The worker answers the next call.
  expect_identical(miniextendr:::quoted_worker_call("base", "toupper", "ok"), "OK")
})

# endregion

# region: GC stress

test_that("unevaluated arguments and R exits survive gctorture", {
  skip_gc_stress_if_disabled()
  t <- structure(tbl, class = c("mx_quoted_tbl", "data.frame"))
  limit <- 5
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  ok <- 0L
  for (i in seq_len(5L)) {
    rows <- miniextendr:::quoted_rows(tbl, b > limit)
    sub <- subset(t, b > limit)
    e <- tryCatch(miniextendr:::quoted_eval(stop("gc")), error = identity)
    w <- tryCatch(miniextendr:::quoted_eval(warning("gc")), warning = identity)
    n <- miniextendr:::quoted_name("x")
    if (identical(rows, 3:5) && identical(sub$a, 3:5) &&
        identical(conditionMessage(e), "gc") && inherits(w, "warning") &&
        identical(n, "x") && identical(miniextendr:::gc_stress_quoted(), 66L)) {
      ok <- ok + 1L
    }
  }
  gctorture(FALSE)
  expect_identical(ok, 5L)
})

test_that("Quosure evaluation survives gctorture", {
  skip_gc_stress_if_disabled()
  skip_if_not_installed("rlang")
  a <- 100
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  ok <- 0L
  for (i in seq_len(3L)) {
    out <- miniextendr:::quosure_eval_tidy(tbl, a * 2)
    e <- tryCatch(miniextendr:::quosure_eval_tidy(tbl, stop("gc")), error = identity)
    if (identical(out, c(2, 4, 6, 8, 10)) && identical(conditionMessage(e), "gc")) {
      ok <- ok + 1L
    }
  }
  gctorture(FALSE)
  expect_identical(ok, 3L)
})

# endregion
