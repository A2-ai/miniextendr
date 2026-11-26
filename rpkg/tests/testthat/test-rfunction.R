# `RFunction` (#1947): an R function as an argument. It accepts what
# `is.function()` accepts, refuses anything else as an argument error, reads
# `NULL` as `None` under `Option`, and calls the function with the user's
# handlers in place. Fixtures: rpkg/src/rust/rfunction_tests.rs.

refused <- function(expr) tryCatch(expr, error = identity)

test_that("RFunction accepts a closure, a builtin and a special", {
  expect_identical(miniextendr:::rfunction_kind(function(x) x), "closure")
  expect_identical(miniextendr:::rfunction_kind(sum), "builtin")
  expect_identical(miniextendr:::rfunction_kind(quote), "special")
  expect_identical(miniextendr:::rfunction_kind(mean), "closure")
})

test_that("anything else is refused, by the R check and by the conversion", {
  e <- refused(miniextendr:::rfunction_kind(1L))
  expect_s3_class(e, "rust_error")
  expect_identical(conditionMessage(e), "'f' must be a function")
  expect_identical(e$param, "f")

  # Without the R check the Rust conversion refuses it, in the same words.
  e <- refused(miniextendr:::rfunction_kind_unchecked(1L))
  expect_s3_class(e, "rust_error")
  expect_identical(conditionMessage(e), "'f' must be a function: got integer")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "f")
  expect_identical(miniextendr:::rfunction_kind_unchecked(sum), "builtin")

  expect_identical(
    conditionMessage(refused(miniextendr:::rfunction_kind_unchecked("sum"))),
    "'f' must be a function: got character"
  )
  expect_identical(
    conditionMessage(refused(miniextendr:::rfunction_kind_unchecked(NULL))),
    "'f' must be a function: got NULL"
  )
})

test_that("Option<RFunction> reads NULL as None and refuses NA", {
  expect_false(miniextendr:::rfunction_is_some(NULL))
  expect_true(miniextendr:::rfunction_is_some(function() 1))
  expect_true(miniextendr:::rfunction_is_some(sum))
  expect_identical(
    conditionMessage(refused(miniextendr:::rfunction_is_some(NA))),
    "'f' must be NULL or a function"
  )
  expect_identical(
    conditionMessage(refused(miniextendr:::rfunction_is_some(list()))),
    "'f' must be NULL or a function"
  )
})

test_that("a returned RFunction is the function given", {
  f <- function(x) x + 1
  expect_identical(miniextendr:::rfunction_identity(f), f)
  expect_identical(miniextendr:::rfunction_identity(sum), sum)
})

test_that("calling it gives the function's value", {
  expect_identical(miniextendr:::rfunction_call_on(function(x) x * 2, 3), 6)
  expect_identical(miniextendr:::rfunction_call_on(sqrt, 16), 4)
  expect_identical(miniextendr:::rfunction_call_on(function(x) "text", 1), "text")
})

test_that("the function's error reaches the user with its class", {
  boom <- function(x) stop(errorCondition("boom", class = "mx_callback_error"))
  e <- refused(miniextendr:::rfunction_call_on(boom, 1))
  expect_s3_class(e, "mx_callback_error")
  expect_identical(conditionMessage(e), "boom")
  expect_identical(
    tryCatch(miniextendr:::rfunction_call_on(boom, 1), mx_callback_error = function(e) "caught"),
    "caught"
  )
})

test_that("the function's warnings and messages reach the user's handlers", {
  noisy <- function(x) {
    warning("careful")
    message("hello")
    x
  }
  seen <- character()
  out <- withCallingHandlers(
    miniextendr:::rfunction_call_on(noisy, 5),
    warning = function(w) {
      seen <<- c(seen, conditionMessage(w))
      invokeRestart("muffleWarning")
    },
    message = function(m) {
      seen <<- c(seen, conditionMessage(m))
      invokeRestart("muffleMessage")
    }
  )
  expect_identical(out, 5)
  expect_identical(seen, c("careful", "hello\n"))
  expect_warning(
    suppressMessages(miniextendr:::rfunction_call_on(noisy, 1)),
    "careful"
  )
})
