# Tests for issue-345 spike: rust_* class layering from ALTREP RUnwind callbacks.
#
# These tests verify that panics and error!() from ALTREP `r_unwind` guard callbacks
# produce R conditions that match tryCatch(rust_error = h, ...) and
# tryCatch(altrep_specific = h, ...) respectively.
#
# Functions under test (defined in rpkg/src/rust/altrep_condition_tests.rs):
#   altrep_panic_on_elt(n, message)        — PanickingAltrep
#   altrep_classed_error_on_elt(n, error_class, message)  — ClassedErrorAltrep
#   altrep_panic_at_index(n, panic_at)     — LoopStressAltrep

test_that("PanickingAltrep: plain panic produces rust_error condition", {
  x <- altrep_panic_on_elt(5L, "deliberate panic in elt")

  # tryCatch with rust_error handler must catch it
  result <- tryCatch(
    x[1L],
    rust_error = function(e) class(e)
  )

  # The handler was reached and returned the class vector
  expect_true(is.character(result))
  expect_true("rust_error" %in% result)
  expect_true("simpleError" %in% result)
  expect_true("error" %in% result)
  expect_true("condition" %in% result)
})

test_that("PanickingAltrep: class vector is correctly ordered", {
  x <- altrep_panic_on_elt(5L, "ordered class test")

  cls <- tryCatch(
    x[1L],
    rust_error = function(e) class(e)
  )

  expect_equal(cls, c("rust_error", "simpleError", "error", "condition"))
})

test_that("PanickingAltrep: condition message is preserved", {
  x <- altrep_panic_on_elt(5L, "my specific panic message")

  msg <- tryCatch(
    x[1L],
    rust_error = function(e) conditionMessage(e)
  )

  expect_equal(msg, "my specific panic message")
})

test_that("PanickingAltrep: catching as simpleError also works", {
  x <- altrep_panic_on_elt(3L, "simpleError test")

  result <- tryCatch(
    x[1L],
    simpleError = function(e) "caught as simpleError"
  )

  expect_equal(result, "caught as simpleError")
})

test_that("ClassedErrorAltrep: custom class is caught first", {
  x <- altrep_classed_error_on_elt(5L, "altrep_specific", "classed error message")

  # Custom class handler fires before rust_error
  result <- tryCatch(
    x[1L],
    altrep_specific = function(e) "caught by custom class",
    rust_error = function(e) "caught by rust_error"
  )

  expect_equal(result, "caught by custom class")
})

test_that("ClassedErrorAltrep: rust_error handler catches if custom class not listed", {
  x <- altrep_classed_error_on_elt(5L, "altrep_specific", "classed error message")

  result <- tryCatch(
    x[1L],
    rust_error = function(e) "caught by rust_error"
  )

  expect_equal(result, "caught by rust_error")
})

test_that("ClassedErrorAltrep: class vector starts with custom class then rust_error", {
  x <- altrep_classed_error_on_elt(5L, "altrep_specific", "class vector test")

  cls <- tryCatch(
    x[1L],
    altrep_specific = function(e) class(e)
  )

  expect_equal(cls[1], "altrep_specific")
  expect_equal(cls[2], "rust_error")
  expect_true("simpleError" %in% cls)
  expect_true("error" %in% cls)
  expect_true("condition" %in% cls)
})

test_that("ClassedErrorAltrep: message is preserved with custom class", {
  x <- altrep_classed_error_on_elt(5L, "altrep_specific", "my classed error message")

  msg <- tryCatch(
    x[1L],
    altrep_specific = function(e) conditionMessage(e)
  )

  expect_equal(msg, "my classed error message")
})

test_that("LoopStressAltrep: earlier elements succeed, panic at the right index", {
  n <- 50L
  panic_at <- 30L  # 0-indexed: element 31 (1-indexed)
  x <- altrep_panic_at_index(n, panic_at)

  # Elements before panic_at should be accessible without error
  # (0-indexed panic_at = 30 means 1-indexed element 31)
  for (i in seq_len(panic_at)) {
    expect_equal(x[i], i - 1L)
  }
})

test_that("LoopStressAltrep: panics at the correct index and produces rust_error", {
  n <- 50L
  panic_at <- 30L  # 0-indexed
  x <- altrep_panic_at_index(n, panic_at)

  # Element at panic_at+1 (1-indexed) should produce rust_error
  result <- tryCatch(
    x[panic_at + 1L],
    rust_error = function(e) conditionMessage(e)
  )

  expect_match(result, "deliberate panic at index 30")
})

test_that("LoopStressAltrep: R does not crash, loop terminates cleanly", {
  # Stress test: tight loop accessing elements, panicking at index 49 (last element)
  # Addresses issue-345 open question #2: does Rf_eval(stop(...)) from ALTREP
  # context risk re-entering ALTREP dispatch?
  n <- 100L
  panic_at <- 49L  # 0-indexed: element 50 in 1-indexed

  x <- altrep_panic_at_index(n, panic_at)

  # Access all elements up to panic_at without error
  successes <- integer(0)
  for (i in seq_len(panic_at)) {
    successes <- c(successes, x[i])
  }
  expect_equal(length(successes), panic_at)

  # The panic index triggers rust_error — R does not crash
  caught <- tryCatch(
    x[panic_at + 1L],
    rust_error = function(e) "rust_error_caught"
  )
  expect_equal(caught, "rust_error_caught")

  # Further accesses after the error are safe — R session is intact
  expect_equal(x[1L], 0L)
  expect_equal(x[panic_at], panic_at - 1L)
})

test_that("LoopStressAltrep: many sequential panics do not corrupt state", {
  # Each tryCatch catches a fresh error; the ALTREP vector remains usable.
  x <- altrep_panic_at_index(10L, 5L)  # panics at 0-indexed 5 = 1-indexed 6

  for (trial in 1:5) {
    result <- tryCatch(
      x[6L],
      rust_error = function(e) "caught"
    )
    expect_equal(result, "caught")
  }
})
