# `#[r_data(cache)]` fields (#2271): the getter keeps the R value its first
# read converts and returns that same object until the value is borrowed
# exclusively: a `&mut self` method, a `&mut T` argument, an R setter. A
# `&self` method keeps it. R code that modifies a read value copies it first,
# so the kept value stays as it was. A save never carries it: `serialize()`
# writes the same bytes before and after a read, and a restored object
# converts on its first read.
#
# Fixtures: src/rust/rdata_cache_tests.rs, and `gc_stress_rdata_cache()` in
# src/rust/gc_stress_fixtures.rs.

caught <- function(expr) tryCatch(expr, error = identity)
address <- function(x) miniextendr:::rdata_cache_address(x)
reload <- function(x) unserialize(serialize(x, NULL))
reload_rds <- function(x) {
  path <- tempfile(fileext = ".rds")
  on.exit(unlink(path), add = TRUE)
  saveRDS(x, path)
  readRDS(path)
}

# region: env, repeated reads and what drops the kept value

test_that("a cached field returns the same object on every read; an uncached one converts each time", {
  x <- CacheEnv$new(3L)
  first <- x$values
  expect_identical(first, c(1, 2, 3))
  expect_identical(address(x$values), address(first))
  expect_identical(address(x[["values"]]), address(first))
  plain <- x$plain
  expect_identical(x$plain, plain)
  expect_false(address(x$plain) == address(plain))
  # A `&self` method keeps it.
  expect_identical(x$count(), 3L)
  expect_identical(address(x$values), address(first))
})

test_that("a &mut self method, a &mut T argument and an R setter each drop the kept value", {
  x <- CacheEnv$new(3L)
  before <- x$values
  x$push(4)
  after <- x$values
  expect_identical(after, c(1, 2, 3, 4))
  expect_false(address(after) == address(before))
  expect_identical(address(x$values), address(after))

  miniextendr:::rdata_cache_push(x, 5)
  argument <- x$values
  expect_identical(argument, c(1, 2, 3, 4, 5))
  expect_false(address(argument) == address(after))
  expect_identical(address(x$values), address(argument))

  x$values <- c(9, 8)
  set <- x$values
  expect_identical(set, c(9, 8))
  expect_false(address(set) == address(argument))
  # A write of another field is an exclusive borrow too.
  x$plain <- 1
  expect_identical(x$values, c(9, 8))
  expect_false(address(x$values) == address(set))
  # The values a read returned earlier are unchanged.
  expect_identical(before, c(1, 2, 3))
  expect_identical(after, c(1, 2, 3, 4))
})

test_that("a cached read-only field and a cached null_if_none field keep their values too", {
  x <- CacheEnv$new(2L)
  tags <- x$tags
  expect_identical(tags, c("v1", "v2"))
  expect_identical(address(x$tags), address(tags))
  e <- caught(x$tags <- "a")
  expect_s3_class(e, "miniextendr_read_only_field")
  expect_identical(address(x$tags), address(tags))
  x$push(3)
  expect_identical(x$tags, c("v1", "v2", "v3"))
  expect_false(address(x$tags) == address(tags))

  expect_null(x$label)
  expect_null(x$label)
  x$relabel("a")
  label <- x$label
  expect_identical(label, "a")
  expect_identical(address(x$label), address(label))
  x$label <- NULL
  expect_null(x$label)
  x$label <- "b"
  expect_identical(x$label, "b")
})

test_that("modifying a value a read returned copies it, leaving the kept value as it was", {
  x <- CacheEnv$new(3L)
  v <- x$values
  v[1] <- 99
  attr(v, "note") <- "changed"
  expect_identical(x$values, c(1, 2, 3))
  w <- x$values
  w[[2]] <- -1
  names(w) <- c("a", "b", "c")
  expect_identical(x$values, c(1, 2, 3))
  # A complex assignment goes through the setter, which drops the kept value.
  x$values[1] <- 99
  expect_identical(x$values, c(99, 2, 3))
  expect_identical(v, structure(c(99, 2, 3), note = "changed"))
})
# endregion

# region: S3

test_that("S3: `$` and `[[` return the kept object, and a write drops it", {
  x <- new_caches3(4L)
  first <- x$values
  expect_identical(first, c(1, 2, 3, 4))
  expect_identical(address(x[["values"]]), address(first))
  expect_identical(as.list(x), list(values = c(1, 2, 3, 4), plain = c(1, 2, 3, 4)))
  expect_identical(address(x$values), address(first))
  x[["values"]] <- 1
  expect_identical(x$values, 1)
  expect_false(address(x$values) == address(first))
})

test_that("S3: serialize() writes the same bytes before and after a cached read", {
  x <- new_caches3(100000L)
  before <- serialize(x, NULL)
  values <- x$values
  expect_identical(length(values), 100000L)
  expect_identical(serialize(x, NULL), before)
  # A restored pointer has no Rust value, so it reads no kept value either.
  e <- caught(reload(x)$values)
  expect_s3_class(e, "miniextendr_restored")
})
# endregion

# region: R6 and S7

test_that("R6: the active binding returns the kept object until a method or the binding writes", {
  r <- CacheR6$new(3L)
  first <- r$values
  expect_identical(address(r$values), address(first))
  r$push(4)
  expect_identical(r$values, c(1, 2, 3, 4))
  expect_false(address(r$values) == address(first))
  second <- r$values
  r$values <- c(7, 7)
  expect_identical(r$values, c(7, 7))
  expect_false(address(r$values) == address(second))
})

test_that("S7: the properties return the kept object until a property write", {
  s <- CacheS7(3L)
  first <- s@values
  expect_identical(first, c(1, 2, 3))
  expect_identical(address(s@values), address(first))
  expect_null(s@label)
  expect_null(s@label)
  s@values <- c(5, 6)
  expect_identical(s@values, c(5, 6))
  expect_false(address(s@values) == address(first))
  s@label <- "a"
  label <- s@label
  expect_identical(label, "a")
  expect_identical(address(s@label), address(label))
  expect_no_error(S7::validate(s))
})
# endregion

# region: serde, a save carries no kept value

test_that("a serde type's save carries its value, not the object a read kept", {
  for (back_of in list(reload, reload_rds)) {
    r <- SavedCache$new(3L)
    # The bytes differ by the flags R sets on a binding it has called, so
    # the sizes are compared.
    before <- length(serialize(r, NULL))
    kept <- r$values
    expect_identical(length(serialize(r, NULL)), before)
    back <- back_of(r)
    restored <- back$values
    expect_identical(restored, c(1, 2, 3))
    expect_false(address(restored) == address(kept))
    expect_identical(address(back$values), address(restored))
    # The two objects keep their values apart.
    back$push(4)
    expect_identical(back$values, c(1, 2, 3, 4))
    expect_identical(address(r$values), address(kept))
  }
})
# endregion

# region: gctorture

test_that("gc_stress_rdata_cache runs", {
  expect_null(miniextendr:::gc_stress_rdata_cache())
})

test_that("kept field values survive allocation-triggered collection", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  miniextendr:::gc_stress_rdata_cache()
  gctorture(FALSE)
  succeed()
})
# endregion
