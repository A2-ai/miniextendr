# Regression tests for #1114: an `#[miniextendr(s7)]` impl whose method names
# collide with plain (non-generic) base/stats closures (`var`, `get`) must
# install and dispatch. The S7 codegen shadows the base closure with a
# package-local generic and registers a `class_any` fallback delegating to the
# masked function, so ordinary calls keep working while S7 objects dispatch.
#
# The fixture (`S7NonGenericCollision`) is an *explicit* `#[miniextendr(s7)]`
# class, so this collision path is covered on default-feature builds, not only
# the `s7-default` feature leg.

test_that("S7 class with base-name-colliding methods installs and constructs", {
  # Reaching this test at all proves R CMD INSTALL succeeded: before #1114 the
  # generated `S7::method(<plain closure>, ...) <-` errored at package load.
  g <- S7NonGenericCollision(values = c(2, 4, 6))
  expect_true(S7::S7_inherits(g, S7NonGenericCollision))
})

test_that("colliding generic dispatches to the S7 method for our objects", {
  g <- S7NonGenericCollision(values = c(2, 4, 6))
  # `var` returns the sum (12), deliberately != stats::var, so a pass here can
  # only mean our method ran (not the masked stats::var fallback).
  expect_equal(var(g), 12)
  expect_equal(get(g), 2)
})

test_that("fast-path shortcuts dispatch to the S7 method", {
  g <- S7NonGenericCollision(values = c(10, 20, 30))
  expect_equal(S7NonGenericCollision_var(g), 60)
  expect_equal(S7NonGenericCollision_get(g), 10)
})

test_that("the masking generic falls back to the shadowed base/stats function", {
  # `var` masks stats::var when the package is attached, but the class_any
  # fallback must delegate ordinary inputs to it.
  expect_equal(var(1:10), stats::var(1:10))
  expect_equal(var(c(2, 4, 4, 4, 5, 5, 7, 9)), stats::var(c(2, 4, 4, 4, 5, 5, 7, 9)))

  # `get` masks base::get; the fallback forwards through `...`. It resolves
  # names against its own scope up to the search path and honours an explicit
  # `envir=`/`mode=` (NB: masking re-roots the default environment, so a bare
  # `get("x")` for a *caller-frame local* is the one behaviour that differs
  # from base::get — pass `envir=` for locals. See the PR notes.).
  env <- new.env()
  assign("marker", 4242L, envir = env)
  expect_identical(get("marker", envir = env), 4242L)
  expect_true(is.function(get("mean", mode = "function")))
})

test_that("dispatch failures stay loud (no silent mis-dispatch)", {
  # An input with no applicable S7 method and no valid fallback must error, not
  # silently return a wrong value. If the classifier ever mis-judged a plain
  # closure as usable, S7::method<- would instead fail loudly at load.
  expect_error(var(list(1, 2, 3)))
})
