# Tests for functional (native-pipe) builder support.
#
# `&mut self -> &mut Self` builder methods on an `#[miniextendr(s3)]` impl
# generate pipe-friendly S3 free functions: the object is the first argument
# and the (same, mutated) object is returned, so the methods compose under R's
# native pipe operator `|>`. The generated S3 generic is named after the Rust
# method (e.g. `set_name`), dispatching on the object's class.

test_that("GreetingBuilder chains under |> and build() returns a String", {
  result <- new_greetingbuilder() |>
    set_name("World") |>
    set_punctuation("!") |>
    build()
  expect_equal(result, "Hello, World!")

  loud <- new_greetingbuilder() |>
    set_name("World") |>
    set_loud(TRUE) |>
    build()
  expect_equal(loud, "HELLO, WORLD.")

  # Defaults: empty name -> "world", default punctuation "."
  expect_equal(build(new_greetingbuilder()), "Hello, world.")
})

test_that("self-returning builder steps preserve object identity (in-place, no clone)", {
  b <- new_greetingbuilder()
  # Each step returns the SAME ExternalPtr handle wrapped in the same S3 object.
  out <- set_name(b, "Ada")
  expect_identical(out, b)
  expect_s3_class(out, "GreetingBuilder")
  # The mutation is visible through the original handle: building from `b`
  # after mutating `out` (same object) reflects the change.
  expect_equal(build(b), "Hello, Ada.")
})

test_that("PipeCounter mutates in place across a |> chain", {
  ctr <- new_pipecounter(1L) |>
    bump(4L) |>
    twice() |>   # (1 + 4) * 2 = 10
    bump(5L)     # 10 + 5 = 15
  expect_s3_class(ctr, "PipeCounter")
  expect_equal(peek(ctr), 15L)
})

test_that("PipeCounter self-ref steps return the same object", {
  ctr <- new_pipecounter(0L)
  expect_identical(bump(ctr, 3L), ctr)
  expect_identical(twice(ctr), ctr)
  # After bump(0, 3) = 3 then twice -> 6
  expect_equal(peek(ctr), 6L)
})

test_that("pipe-builder generics and methods are exported", {
  exports <- getNamespaceExports("miniextendr")
  for (fn in c(
    "new_greetingbuilder", "set_name", "set_punctuation",
    "set_loud", "build", "new_pipecounter", "bump",
    "twice", "peek"
  )) {
    expect_true(fn %in% exports, info = sprintf("`%s` missing from exports", fn))
  }
})

# ---------------------------------------------------------------------------
# Cross-class-system coverage for self-ref builders (#769)
#
# A `&mut self -> &mut Self` builder step plus a terminal accessor must chain
# on every impl-block class system, and must preserve object identity wherever
# the system is reference-semantic. R6/Env chain via `invisible(self)`;
# S4/S7 chain by returning the receiver `x` from the generated generic. The
# critical R6 guarantee: chaining must NOT mint a new R6 wrapper around the same
# pointer (that would break identity) — it returns the *same* environment.
# ---------------------------------------------------------------------------

test_that("R6PipeBuilder chains via invisible(self) and preserves identity", {
  b <- R6PipeBuilder$new()
  # `$add()` returns the same R6 object (invisible(self)), so we can chain and
  # the chain reads through the same wrapper.
  expect_equal(b$add(1L)$add(2L)$total(), 3L)

  # Identity: the value returned by a builder step IS the same R6 environment,
  # not a freshly minted wrapper around the same pointer.
  b2 <- R6PipeBuilder$new()
  stepped <- b2$add(5L)
  expect_identical(stepped, b2)
  # The mutation is visible through the original handle.
  expect_equal(b2$total(), 5L)
})

test_that("S4PipeBuilder chains under |> and preserves identity", {
  total <- S4PipeBuilder() |>
    s4_add(1L) |>
    s4_add(2L) |>
    s4_total()
  expect_equal(total, 3L)

  # Identity: the self-ref step returns the same S4 object (same ExternalPtr).
  b <- S4PipeBuilder()
  stepped <- s4_add(b, 5L)
  expect_identical(stepped, b)
  expect_equal(s4_total(b), 5L)
})

test_that("S7PipeBuilder chains under |> and preserves identity", {
  total <- S7PipeBuilder() |>
    s7_add(1L) |>
    s7_add(2L) |>
    s7_total()
  expect_equal(total, 3L)

  # Identity: the self-ref step returns the same S7 object (same ExternalPtr).
  b <- S7PipeBuilder()
  stepped <- s7_add(b, 5L)
  expect_identical(stepped, b)
  expect_equal(s7_total(b), 5L)
})

test_that("EnvPipeBuilder chains via $ and preserves identity", {
  b <- EnvPipeBuilder$new()
  expect_equal(b$add(1L)$add(2L)$total(), 3L)

  # Identity: the self-ref step returns the same environment.
  b2 <- EnvPipeBuilder$new()
  stepped <- b2$add(5L)
  expect_identical(stepped, b2)
  expect_equal(b2$total(), 5L)
})
