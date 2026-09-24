# The consumer's `CounterView` converts every argument to a SEXP before it
# calls the producer's vtable shim. Each converted argument has to stay rooted
# while the next one allocates and while the shim reads them back; under
# gctorture an unrooted argument is collected and the shim reads a freed or
# reused node (the wrong class, the wrong message, or a crash).

test_that("View arguments stay rooted across a cross-package vtable call under gctorture", {
  skip_on_cran()
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(1L)
  signal <- function(i) {
    tryCatch(
      {
        counter_raise_condition_classed(counter, "view_arg_signal", "view arg message")
        "not signalled"
      },
      view_arg_signal = function(c) conditionMessage(c),
      error = function(e) paste("error:", conditionMessage(e))
    )
  }
  expect_identical(signal(0L), "view arg message")

  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  results <- vapply(seq_len(5L), signal, character(1L))
  gctorture(FALSE)

  expect_identical(results, rep("view arg message", 5L))
})
