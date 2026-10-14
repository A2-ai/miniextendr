# `ExternalPtr<T>` arguments of a function hold the object's value (#2092).
#
# The C wrapper borrows the value's `RefCell` for each `ExternalPtr<T>` and
# `Option<ExternalPtr<T>>` argument, as a method's wrapper does for its
# receiver: exclusively for a `mut` binding, shared otherwise, through the
# conversion of the result. A body that moves the value out of its pointer
# (`ExternalPtr::into_inner` and co.) takes no borrow. Fixtures:
# `sidecar_env_*()` in `src/rust/rdata_sidecar_tests.rs`.

test_that("one object passed for a mut argument and another argument conflicts", {
  a <- SidecarEnv$new(1L, 0, TRUE, "a")
  b <- SidecarEnv$new(2L, 0, TRUE, "b")
  expect_error(miniextendr:::sidecar_env_pair(a, a), class = "miniextendr_borrow_conflict")
  expect_error(miniextendr:::sidecar_env_pair_rev(a, a), class = "miniextendr_borrow_conflict")
  expect_identical(a$count, 1L)
  expect_identical(miniextendr:::sidecar_env_pair(a, b), 3L)
  # Shared arguments nest.
  expect_identical(miniextendr:::sidecar_env_sum(a, a), 6L)
})

test_that("a callback can't touch an object a function holds as a mut argument", {
  obj <- SidecarEnv$new(1L, 0, TRUE, "a")
  callbacks <- list(
    read = function() obj$count,
    write = function() obj$count <- 5L,
    method = function() obj$double_count(),
    argument = function() miniextendr:::sidecar_env_read_arg(obj, function() NULL)
  )
  for (name in names(callbacks)) {
    expect_error(
      miniextendr:::sidecar_env_bump_arg(obj, callbacks[[name]]),
      class = "miniextendr_borrow_conflict",
      label = name
    )
  }
  expect_identical(miniextendr:::sidecar_env_bump_arg(obj, function() NULL), 2L)
})

test_that("a callback may read, but not write, an object a function holds shared", {
  obj <- SidecarEnv$new(3L, 0, TRUE, "a")
  seen <- NULL
  expect_identical(
    miniextendr:::sidecar_env_read_arg(obj, function() seen <<- obj$count),
    3L
  )
  expect_identical(seen, 3L)
  expect_error(
    miniextendr:::sidecar_env_read_arg(obj, function() obj$count <- 4L),
    class = "miniextendr_borrow_conflict"
  )
  expect_error(
    miniextendr:::sidecar_env_read_arg(obj, function() obj$double_count()),
    class = "miniextendr_borrow_conflict"
  )
  expect_identical(obj$count, 3L)
})

test_that("a method's receiver and an argument of the same object conflict", {
  obj <- SidecarEnv$new(1L, 0, TRUE, "a")
  expect_error(
    obj$bump_around(function() miniextendr:::sidecar_env_read_arg(obj, function() NULL)),
    class = "miniextendr_borrow_conflict"
  )
  expect_identical(
    obj$read_around(function() miniextendr:::sidecar_env_read_arg(obj, function() NULL)),
    1L
  )
})

test_that("an Option<ExternalPtr<T>> argument borrows its object, and NULL is none", {
  obj <- SidecarEnv$new(1L, 0, TRUE, "a")
  expect_error(
    miniextendr:::sidecar_env_bump_opt(obj, function() obj$count),
    class = "miniextendr_borrow_conflict"
  )
  expect_identical(miniextendr:::sidecar_env_bump_opt(obj, function() NULL), 2L)
  expect_identical(miniextendr:::sidecar_env_bump_opt(NULL, function() NULL), NA_integer_)
})

test_that("a function that moves the value out of its argument takes no borrow", {
  obj <- SidecarEnv$new(4L, 0, TRUE, "a")
  expect_identical(miniextendr:::sidecar_env_take(obj), 4L)
})

test_that("try_borrow() / try_borrow_mut() check the value of a handle a function keeps", {
  a <- SidecarEnv$new(1L, 0, TRUE, "a")
  b <- SidecarEnv$new(2L, 0, TRUE, "b")
  expect_identical(miniextendr:::sidecar_env_try_borrow(list(a, b), TRUE), 3L)
  expect_identical(a$count, 3L)
  expect_identical(miniextendr:::sidecar_env_try_borrow(list(a, a), FALSE), 6L)
  expect_error(
    miniextendr:::sidecar_env_try_borrow(list(a, a), TRUE),
    class = "miniextendr_borrow_conflict"
  )
  expect_identical(a$count, 3L)
})

test_that("a class-wrapped argument stays rooted for the call", {
  expect_null(miniextendr:::gc_stress_externalptr_argument_root())
})

test_that("class-wrapped arguments survive gctorture", {
  skip_gc_stress_if_disabled()
  a <- SidecarSlotR6$new(2L)
  b <- SidecarSlotR6$new(4L)
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  for (i in seq_len(3)) {
    sidecar_swap(a, b)
    expect_null(miniextendr:::gc_stress_externalptr_argument_root())
  }
  gctorture(FALSE)
  expect_identical(a$n, 4L)
  expect_identical(b$n, 2L)
})
