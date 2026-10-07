# `#[miniextendr(call_arg)]` (#1834): an exported function takes a `.call`
# argument, and its conditions name the call passed there. Fixtures in
# src/rust/call_attribution_demo.rs; docs/CALL_ATTRIBUTION.md, "An exported
# function: `call_arg`".

# Composes `call_arg_verb()` the way an `update()` method applies verbs: by
# name, through `do.call()`, passing its own frame as `.call`.
compose_verb <- function(x, mode = "Fast") {
  do.call("call_arg_verb", list(x, mode, .call = environment()))
}

test_that("`call_arg` adds `.call = NULL` after the formals", {
  expect_equal(names(formals(call_arg_verb)), c("x", "mode", ".call"))
  expect_null(formals(call_arg_verb)$.call)
  expect_equal(names(formals(call_arg_joined)), c("x", ".call"))
  expect_equal(names(formals(call_arg_topic)), "x")
  expect_equal(call_arg_verb(3L), 3L)
  expect_equal(compose_verb(3L, "Safe"), 3L)
})

test_that("a Rust error names the composing function's call", {
  e <- tryCatch(compose_verb(-1L), error = identity)
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e), "x must be non-negative, got -1", fixed = TRUE)
  expect_equal(conditionCall(e), quote(compose_verb(-1L)))
})

test_that("an R-side check names the composing function's call", {
  e <- tryCatch(compose_verb(1.5), error = identity)
  expect_s3_class(e, "rust_error")
  expect_equal(e$param, "x")
  expect_equal(conditionCall(e), quote(compose_verb(1.5)))
})

test_that("a `match_arg` refusal names the composing function's call", {
  e <- tryCatch(compose_verb(1L, mode = "Slow"), error = identity)
  expect_s3_class(e, "rust_error")
  expect_equal(e$param, "mode")
  expect_equal(conditionCall(e), quote(compose_verb(1L, mode = "Slow")))
})

test_that("a warning deferred from Rust names the composing function's call", {
  w <- tryCatch(compose_verb(0L), warning = identity)
  expect_match(conditionMessage(w), "x is zero", fixed = TRUE)
  expect_equal(conditionCall(w), quote(compose_verb(0L)))
  expect_equal(suppressWarnings(compose_verb(0L)), 0L)
})

test_that("without `.call` the wrapper names its own call", {
  e <- tryCatch(call_arg_verb(-1L), error = identity)
  expect_equal(conditionCall(e), quote(call_arg_verb(-1L)))
  e <- tryCatch(call_arg_verb(1.5), error = identity)
  expect_equal(conditionCall(e), quote(call_arg_verb(1.5)))
  w <- tryCatch(call_arg_verb(0L), warning = identity)
  expect_equal(conditionCall(w), quote(call_arg_verb(0L)))
  # Not its caller's: unlike `call = caller`, NULL is this call.
  via <- function(value) call_arg_verb(value)
  e <- tryCatch(via(-1L), error = identity)
  expect_equal(conditionCall(e), quote(call_arg_verb(value)))
})

test_that("`.call` takes a call object as is", {
  e <- tryCatch(
    call_arg_verb(-1L, .call = quote(update(a, verb = -1L))),
    error = identity
  )
  expect_equal(conditionCall(e), quote(update(a, verb = -1L)))
  # Through `do.call()` a call object needs `quote = TRUE`.
  e <- tryCatch(
    do.call(call_arg_verb, list(-1L, .call = quote(update(a))), quote = TRUE),
    error = identity
  )
  expect_equal(conditionCall(e), quote(update(a)))
})

test_that("an environment that is no closure's frame names the wrapper's own call", {
  e <- tryCatch(call_arg_verb(-1L, .call = globalenv()), error = identity)
  expect_equal(conditionCall(e), quote(call_arg_verb(-1L)))
  # From a closure too: the fallback is the wrapper's own call, not the caller's.
  via <- function(value) call_arg_verb(value, .call = globalenv())
  e <- tryCatch(via(-1L), error = identity)
  expect_equal(conditionCall(e), quote(call_arg_verb(value)))
})

test_that("anything else passed as `.call` is an argument error on `.call`", {
  e <- tryCatch(call_arg_verb(1L, .call = "update"), error = identity)
  expect_s3_class(e, "rust_error")
  expect_equal(e$param, ".call")
  expect_equal(conditionMessage(e), "'.call' must be NULL, an environment or a call")
  # The wrapper's own call, as written.
  expect_equal(conditionCall(e), quote(call_arg_verb(1L, .call = "update")))
})

test_that("a `Call` body receives the call `.call` resolves to", {
  expect_equal(
    miniextendr:::call_arg_marker_impl(1L),
    quote(miniextendr:::call_arg_marker_impl(1L))
  )
  expect_equal(miniextendr:::call_arg_marker_impl(1L, .call = quote(f(x))), quote(f(x)))
  own <- function(value) miniextendr:::call_arg_marker_impl(value, .call = environment())
  expect_equal(own(1L), quote(own(1L)))
})

test_that("`call_arg` functions document `.call`, on a page they join too", {
  rd_db <- tryCatch(tools::Rd_db("miniextendr"), error = function(e) NULL)
  skip_if(is.null(rd_db), "tools::Rd_db('miniextendr') unavailable — package not installed")
  rd_text <- function(page) {
    paste(capture.output(print(rd_db[[page]])), collapse = "\n")
  }
  verb <- rd_text("call_arg_verb.Rd")
  expect_match(verb, "call_arg_verb(x, mode = c(\"Fast\", \"Safe\", \"Debug\"), .call = NULL)", fixed = TRUE)
  expect_match(verb, "\\item{.call}{", fixed = TRUE)
  # `call_arg_joined()` joins `call_arg_topic()`'s page, whose own block has no
  # `.call`: the joined block's line is kept, once.
  topic <- rd_text("call_arg_topic.Rd")
  expect_match(topic, "call_arg_joined(x, .call = NULL)", fixed = TRUE)
  expect_equal(lengths(regmatches(topic, gregexpr("\\item{.call}{", topic, fixed = TRUE))), 1L)
})
