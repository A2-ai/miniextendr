# A method that holds its object's Rust value and calls back into R (#1965).
#
# The generated wrappers and field accessors borrow the object's `RefCell`,
# shared (`&self`, field reads) or exclusive (`&mut self`, field writes),
# before they make a Rust reference. A callback that conflicts with the
# running method gets a `miniextendr_borrow_conflict` error instead of a
# second reference. Fixture: `SidecarEnv$bump_around()` / `$read_around()` in
# `src/rust/rdata_sidecar_tests.rs`.

test_that("a callback can't touch an object whose method holds it mutably", {
  obj <- SidecarEnv$new(1L, 0, TRUE, "a")
  callbacks <- list(
    read = function() obj$count,
    write = function() obj$count <- 5L,
    method = function() obj$double_count(),
    shared_method = function() obj$read_around(function() NULL)
  )
  for (name in names(callbacks)) {
    expect_error(
      obj$bump_around(callbacks[[name]]),
      class = "miniextendr_borrow_conflict",
      label = name
    )
  }
  # Each refused call released its borrow and left the value alone.
  expect_identical(obj$bump_around(function() NULL), 2L)
})

test_that("a callback may read, but not write, an object whose method holds it shared", {
  obj <- SidecarEnv$new(3L, 0, TRUE, "a")
  seen <- NULL
  expect_identical(obj$read_around(function() seen <<- obj$count), 3L)
  expect_identical(seen, 3L)
  expect_identical(obj$read_around(function() obj$read_around(function() NULL)), 3L)
  expect_error(
    obj$read_around(function() obj$count <- 4L),
    class = "miniextendr_borrow_conflict"
  )
  expect_error(
    obj$read_around(function() obj$double_count()),
    class = "miniextendr_borrow_conflict"
  )
  expect_identical(obj$count, 3L)
})

test_that("an R error in a callback releases the borrow", {
  obj <- SidecarEnv$new(2L, 0, TRUE, "a")
  expect_error(obj$bump_around(function() stop("boom")), "boom")
  obj$double_count()
  expect_identical(obj$count, 4L)
})

test_that("a borrow of one object doesn't hold another of the same class", {
  a <- SidecarEnv$new(1L, 0, TRUE, "a")
  b <- SidecarEnv$new(1L, 0, TRUE, "b")
  expect_identical(a$bump_around(function() b$double_count()), 2L)
  expect_identical(b$count, 2L)
})
