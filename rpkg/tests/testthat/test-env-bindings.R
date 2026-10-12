# REnv::get / REnv::get_inherits / REnv::define (#1997). R 4.5 and later read
# with R_getVarEx, R 4.4 with base::get0(); every expectation here holds on
# both.

env_get <- function(env, name, inherits = FALSE) {
  miniextendr:::expr_env_get(env, name, inherits)
}

test_that("get reads a package namespace's own bindings", {
  # Namespace functions are lazy-loaded promises; get() forces them.
  found <- miniextendr:::expr_namespace_get("miniextendr", "expr_env_get")
  expect_identical(found$value, miniextendr:::expr_env_get)
  expect_null(miniextendr:::expr_namespace_get("miniextendr", "no_such_binding_here"))
  # `sum` is base's, an enclosure of the namespace, not a namespace binding.
  expect_null(miniextendr:::expr_namespace_get("miniextendr", "sum"))
  expect_identical(env_get(asNamespace("miniextendr"), "sum", inherits = TRUE)$value, sum)
  expect_error(
    miniextendr:::expr_namespace_get("noSuchPackageForMiniextendr", "x"),
    class = "packageNotFoundError"
  )
})

test_that("get searches the enclosures only when asked", {
  parent <- new.env(parent = emptyenv())
  parent$x <- "from parent"
  parent$y <- 2
  parent$z <- 1
  child <- new.env(parent = parent)
  child$y <- 1
  child$z <- NULL

  expect_null(env_get(child, "x"))
  expect_identical(env_get(child, "x", inherits = TRUE), list(value = "from parent"))
  # The nearest binding wins, a NULL one included.
  expect_identical(env_get(child, "y"), list(value = 1))
  expect_identical(env_get(child, "y", inherits = TRUE), list(value = 1))
  expect_identical(env_get(child, "z"), list(value = NULL))
  expect_identical(env_get(child, "z", inherits = TRUE), list(value = NULL))
})

test_that("get reports a missing name as no binding", {
  env <- new.env(parent = emptyenv())
  expect_null(env_get(env, "absent"))
  expect_null(env_get(env, "absent", inherits = TRUE))
  expect_null(env_get(new.env(), "no_such_binding_anywhere", inherits = TRUE))
})

test_that("get forces a promise binding once", {
  env <- new.env(parent = emptyenv())
  # The promise's code runs in this frame, so it counts in an environment.
  state <- new.env()
  state$forced <- 0L
  delayedAssign("lazy", {
    state$forced <- state$forced + 1L
    42L
  }, assign.env = env)
  expect_identical(env_get(env, "lazy"), list(value = 42L))
  expect_identical(env_get(env, "lazy", inherits = TRUE), list(value = 42L))
  expect_identical(state$forced, 1L)
})

test_that("an error while reading a binding reaches the caller as R raised it", {
  env <- new.env(parent = emptyenv())
  delayedAssign("bad", stop("lazy failure"), assign.env = env)
  makeActiveBinding("boom", function() stop("active failure"), env)
  expect_error(env_get(env, "bad"), "lazy failure", class = "simpleError")
  expect_error(env_get(env, "boom", inherits = TRUE), "active failure", class = "simpleError")
  # The caller's handlers see conditions signalled while reading.
  makeActiveBinding("noisy", function() {
    warning("read warning")
    "value"
  }, env)
  seen <- NULL
  value <- withCallingHandlers(
    env_get(env, "noisy"),
    warning = function(w) {
      seen <<- conditionMessage(w)
      invokeRestart("muffleWarning")
    }
  )
  expect_identical(value, list(value = "value"))
  expect_identical(seen, "read warning")
})

test_that("define binds in the environment's own frame", {
  parent <- new.env(parent = emptyenv())
  parent$x <- "parent"
  child <- new.env(parent = parent)

  miniextendr:::expr_env_define(child, "x", 1:3)
  miniextendr:::expr_env_define(child, "n", NULL)
  expect_identical(get("x", envir = child, inherits = FALSE), 1:3)
  expect_true(exists("n", envir = child, inherits = FALSE))
  expect_null(child$n)
  expect_identical(parent$x, "parent")

  miniextendr:::expr_env_define(child, "x", "replaced")
  expect_identical(child$x, "replaced")

  # Assigning to an active binding calls its function.
  assigned <- NULL
  makeActiveBinding("active", function(value) assigned <<- value, child)
  miniextendr:::expr_env_define(child, "active", 99L)
  expect_identical(assigned, 99L)
})

test_that("define raises R's error where R refuses the binding", {
  locked <- new.env()
  locked$existing <- 1
  lockEnvironment(locked)
  expect_error(
    miniextendr:::expr_env_define(locked, "new_binding", 1),
    "cannot add bindings to a locked environment"
  )
  # An unlocked binding of a locked environment can still change.
  miniextendr:::expr_env_define(locked, "existing", 2)
  expect_identical(locked$existing, 2)
  lockBinding("existing", locked)
  expect_error(
    miniextendr:::expr_env_define(locked, "existing", 3),
    "cannot change value of locked binding"
  )
  expect_error(
    miniextendr:::expr_env_define(emptyenv(), "x", 1),
    "cannot assign values in the empty environment"
  )
  expect_error(
    miniextendr:::expr_env_define(asNamespace("miniextendr"), "mx_no_new_binding", 1),
    "locked"
  )
})
