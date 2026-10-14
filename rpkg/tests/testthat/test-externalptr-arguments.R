# Borrowed arguments of a function hold the object's value (#2092, #2106).
#
# The C wrapper borrows the value's `RefCell` for each `&T` / `&mut T`,
# `ExternalPtrRef` / `ExternalPtrMut` and `Option` of those arguments, as a
# method's wrapper does for its receiver: exclusively for `&mut T` /
# `ExternalPtrMut`, shared otherwise, through the conversion of the result.
# An `ExternalPtr<T>` argument is a handle the wrapper doesn't borrow; its
# `borrow()` / `borrow_mut()` check the value where the body reads it.
# Fixtures: `sidecar_env_*()` in `src/rust/rdata_sidecar_tests.rs`.

test_that("one object passed for a &mut argument and another argument conflicts", {
  a <- SidecarEnv$new(1L, 0, TRUE, "a")
  b <- SidecarEnv$new(2L, 0, TRUE, "b")
  expect_error(miniextendr:::sidecar_env_pair(a, a), class = "miniextendr_borrow_conflict")
  expect_error(miniextendr:::sidecar_env_pair_rev(a, a), class = "miniextendr_borrow_conflict")
  expect_identical(a$count, 1L)
  expect_identical(miniextendr:::sidecar_env_pair(a, b), 3L)
  # Shared arguments nest.
  expect_identical(miniextendr:::sidecar_env_sum(a, a), 6L)
})

test_that("a callback can't touch an object a function holds as a &mut argument", {
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

test_that("an Option<&mut T> argument borrows its object, and NULL is none", {
  obj <- SidecarEnv$new(1L, 0, TRUE, "a")
  expect_error(
    miniextendr:::sidecar_env_bump_opt(obj, function() obj$count),
    class = "miniextendr_borrow_conflict"
  )
  expect_identical(miniextendr:::sidecar_env_bump_opt(obj, function() NULL), 2L)
  expect_identical(miniextendr:::sidecar_env_bump_opt(NULL, function() NULL), NA_integer_)
})

test_that("an ExternalPtrMut argument borrows its object and reaches the R object", {
  obj <- SidecarEnv$new(1L, 0, TRUE, "a")
  expect_error(
    miniextendr:::sidecar_env_bump_view(obj, function() obj$count),
    class = "miniextendr_borrow_conflict"
  )
  f <- function() NULL
  back <- miniextendr:::sidecar_env_bump_view(obj, f)
  expect_identical(typeof(back), "externalptr")
  expect_identical(obj$count, 2L)
  expect_identical(SidecarEnv_get_count(back), 2L)
  expect_identical(SidecarEnv_get_raw_slot(obj), f)
})

test_that("an ExternalPtr<T> argument takes no borrow", {
  obj <- SidecarEnv$new(4L, 0, TRUE, "a")
  # A running call holds the value, so it can't move out.
  expect_error(
    obj$read_around(function() miniextendr:::sidecar_env_take(obj)),
    class = "miniextendr_borrow_conflict"
  )
  expect_identical(obj$count, 4L)
  expect_identical(miniextendr:::sidecar_env_take(obj), 4L)
})

test_that("borrow() / borrow_mut() check the value of a handle a function keeps", {
  a <- SidecarEnv$new(1L, 0, TRUE, "a")
  b <- SidecarEnv$new(2L, 0, TRUE, "b")
  expect_identical(miniextendr:::sidecar_env_borrow_handles(list(a, b), TRUE), 3L)
  expect_identical(a$count, 3L)
  expect_identical(miniextendr:::sidecar_env_borrow_handles(list(a, a), FALSE), 6L)
  expect_error(
    miniextendr:::sidecar_env_borrow_handles(list(a, a), TRUE),
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
