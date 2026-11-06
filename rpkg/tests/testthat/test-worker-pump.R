# WorkerPump: a worker thread computes in pure Rust and sends progress
# messages; the pump runs on R's main thread for each one and is the only
# side that calls R (worker_pump_tests.rs).

sum_squares <- function(n) sum(as.numeric(seq_len(n))^2)

test_that("the worker's result comes back and the pump sees every message", {
  for (capacity in c(0L, 1L, 64L)) {
    out <- worker_pump_sum_squares(100L, capacity, NULL)
    expect_identical(out$total, sum_squares(100L))
    expect_identical(out$ticks, 100L)
    # The worker runs off R's main thread; the pump on it.
    expect_false(out$worker_on_main)
    expect_true(out$pump_on_main)
  }
})

test_that("no work is no message", {
  out <- worker_pump_sum_squares(0L, 0L, NULL)
  expect_identical(out$total, 0)
  expect_identical(out$ticks, 0L)
})

test_that("the pump calls R for each message, in order", {
  seen <- list()
  record <- function(i, partial) seen[[length(seen) + 1L]] <<- c(i, partial)
  out <- worker_pump_sum_squares(5L, 0L, record)
  expect_identical(out$ticks, 5L)
  expect_identical(
    seen,
    list(c(1, 1), c(2, 5), c(3, 14), c(4, 30), c(5, 55))
  )
})

test_that("the caller's handlers see conditions signalled in the pump", {
  shown <- 0L
  out <- withCallingHandlers(
    worker_pump_sum_squares(4L, 1L, function(i, partial) message("term ", i)),
    message = function(m) {
      shown <<- shown + 1L
      invokeRestart("muffleMessage")
    }
  )
  expect_identical(shown, 4L)
  expect_identical(out$total, sum_squares(4L))
})

test_that("an R error in the pump stops the worker and reaches the caller", {
  stop_at_3 <- function(i, partial) if (i == 3L) stop("pump stopped at ", i)
  # Capacity 0: the worker is blocked in send() when the pump fails, and
  # must see the closed channel for the join to return.
  for (capacity in c(0L, 64L)) {
    expect_error(worker_pump_sum_squares(1000L, capacity, stop_at_3), "pump stopped at 3")
  }
  # A tryCatch() exit is an unwind too.
  halted <- tryCatch(
    worker_pump_sum_squares(1000L, 0L, function(i, partial) {
      if (i == 2L) signalCondition(simpleCondition("halt", call = NULL))
    }),
    condition = function(c) conditionMessage(c)
  )
  expect_identical(halted, "halt")
  # The pump is reusable afterwards.
  expect_identical(worker_pump_sum_squares(10L, 0L, NULL)$total, sum_squares(10L))
})

test_that("a failing worker is an error after the pump has drained its messages", {
  expect_error(
    worker_pump_failing(3L, FALSE),
    "the worker gave up after 3 messages (the pump got 3)",
    fixed = TRUE
  )
  expect_error(
    worker_pump_failing(0L, FALSE),
    "the worker gave up after 0 messages (the pump got 0)",
    fixed = TRUE
  )
  expect_error(
    worker_pump_failing(2L, TRUE),
    "WorkerPump worker panicked (the pump got 2)",
    fixed = TRUE
  )
})

test_that("arguments are checked before any thread starts", {
  expect_error(worker_pump_sum_squares(-1L, 0L, NULL), "`n` must be non-negative, not -1", fixed = TRUE)
  expect_error(
    worker_pump_sum_squares(1L, -2L, NULL),
    "`capacity` must be non-negative, not -2",
    fixed = TRUE
  )
})

test_that("a pump growing an R list survives gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE))
  log <- gc_stress_worker_pump()
  gctorture(FALSE)
  expect_identical(names(log), paste0("tick", 0:31))
  expect_identical(log$tick31, rep(31L, 8L))
})
