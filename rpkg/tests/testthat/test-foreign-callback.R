# `unwind_protect::call_unwinding` under a frame an unwind must not cross
# (#1946). `foreign_frame_call(f)` runs `f()` in a callback that a stand-in C
# solver calls; an R exit is caught in the callback, carried past the solver
# and continued there. The solver frame is `extern "C"`: an unwind crossing it
# would abort the process.

solver_condition <- function() {
  structure(
    class = c("mx_solver_failed", "error", "condition"),
    list(message = "diverged", call = NULL, step = 7L)
  )
}

foreign_callback_outcomes <- function() {
  cond <- solver_condition()
  list(
    caught = tryCatch(
      miniextendr:::foreign_frame_call(function() stop(cond)),
      mx_solver_failed = identity
    ),
    restarted = withRestarts(
      miniextendr:::foreign_frame_call(function() invokeRestart("mx_skip", 42L)),
      mx_skip = function(v) v
    ),
    muffled = withCallingHandlers(
      miniextendr:::foreign_frame_call(function() {
        warning("slow convergence")
        "converged"
      }),
      warning = function(w) invokeRestart("muffleWarning")
    ),
    value = miniextendr:::foreign_frame_call(function() 1:3)
  )
}

check_foreign_callback_outcomes <- function(out) {
  expect_identical(out$caught, solver_condition())
  expect_identical(out$restarted, 42L)
  expect_identical(out$muffled, "converged")
  expect_identical(out$value, 1:3)
}

test_that("a classed condition crosses the foreign frame to the caller's handler", {
  cond <- solver_condition()
  caught <- tryCatch(
    miniextendr:::foreign_frame_call(function() stop(cond)),
    mx_solver_failed = identity
  )
  expect_identical(caught, cond)
})

test_that("a restart invoked in the callback reaches withRestarts() past the frame", {
  out <- withRestarts(
    miniextendr:::foreign_frame_call(function() invokeRestart("mx_skip", 42L)),
    mx_skip = function(v) v
  )
  expect_identical(out, 42L)
})

test_that("an error the caller does not handle reaches it unchanged", {
  expect_error(
    miniextendr:::foreign_frame_call(function() stop("no root")),
    "no root",
    class = "simpleError"
  )
})

test_that("the caller's calling handlers run and the callback carries on", {
  check_foreign_callback_outcomes(foreign_callback_outcomes())
})

test_that("each path crosses the foreign frame under gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  out <- foreign_callback_outcomes()
  gctorture(FALSE)
  check_foreign_callback_outcomes(out)
})
