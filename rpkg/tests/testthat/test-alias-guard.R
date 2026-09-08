test_that("distinct mutable vectors pass and aliases fail in every build", {
  a <- c(1L, 2L, 3L)
  b <- c(10L, 20L)
  expect_equal(alias_probe(a, b), 5L)
  expect_equal(a, c(2L, 3L, 4L))
  expect_equal(b, c(11L, 21L))
  x <- c(1L, 2L, 3L)
  expect_error(alias_probe(x, x), "aliasing")
  expect_identical(x, c(1L, 2L, 3L))
})

test_that("list/direct mutable and shared conflicts fail before mutation", {
  for (fn in c("alias_list_mut_direct_mut", "alias_list_mut_direct_shared",
               "alias_list_shared_direct_mut", "alias_rng_lists")) {
    x <- c(1L, 2L, 3L)
    err <- tryCatch(get(fn)(list(x), x), error = identity)
    expect_s3_class(err, "error")
    expect_match(conditionMessage(err), "parameters `a` and `b`", fixed = TRUE)
    expect_identical(x, c(1L, 2L, 3L))
  }
})

test_that("list/list conflicts and nested list conflicts are rejected", {
  for (fn in c("alias_list_mut_list_mut", "alias_list_mut_list_shared",
               "alias_list_shared_list_mut")) {
    x <- c(1L, 2L)
    expect_error(get(fn)(list(x), list(x)), "aliasing", info = fn)
    expect_identical(x, c(1L, 2L))
  }
  x <- c(3L, 4L)
  expect_error(alias_nested_lists(list(list(x)), list(x)), "aliasing")
  expect_error(alias_nested_lists(list(list(x), list(x)), list()), "duplicate elements")
  expect_error(alias_list_mut_direct_shared(list(x, x), integer()), "duplicate elements")
  expect_identical(x, c(3L, 4L))
})

test_that("optional leaves, shared aliases, and owned copies remain valid", {
  x <- c(1L, 2L)
  expect_error(alias_optional_lists(list(NULL, x), x), "aliasing")
  expect_error(alias_optional_lists(list(x, NULL, x), NULL), "duplicate elements")
  expect_identical(alias_optional_lists(NULL, NULL), 0L)
  expect_identical(alias_optional_lists(list(NULL, NULL), NULL), 0L)
  expect_identical(alias_list_shared_list_shared(list(x, x), list(x)), 9L)
  expect_identical(alias_owned_and_list(x, list(x)), 8L)
  expect_identical(x, c(2L, 3L))
})

test_that("empty and distinct vectors do not produce false positives", {
  empty <- integer()
  expect_identical(alias_probe(empty, empty), 0L)
  expect_identical(alias_list_mut_direct_mut(list(empty, empty), empty), 0L)
  expect_identical(alias_optional_lists(list(NULL, empty, integer()), empty), 0L)
  expect_identical(alias_list_mut_list_mut(list(), list()), 0L)
  a <- c(1L, 2L)
  b <- c(10L, 20L)
  expect_identical(alias_list_mut_list_shared(list(a), list(b)), 35L)
  expect_identical(a, c(2L, 3L))
  expect_identical(b, c(10L, 20L))
})

test_that("malformed input retains normal conversion errors", {
  for (args in list(list(1L, 1L), list(list("bad"), "bad"),
                    list(new.env(), 1L), list(list(NULL), 1L))) {
    err <- tryCatch(do.call(alias_list_mut_direct_shared, args), error = identity)
    expect_s3_class(err, "error")
    expect_match(conditionMessage(err), "convert parameter|must be an integer vector")
    expect_false(grepl("aliasing", conditionMessage(err), fixed = TRUE))
  }
})

test_that("every conflicting parameter pair appears in one diagnostic", {
  x <- c(1L, 2L)
  err <- tryCatch(alias_three_arguments(list(x), x, list(x)), error = identity)
  for (pair in c("parameters `a` and `b`", "parameters `a` and `c`",
                 "parameters `b` and `c`")) {
    expect_match(conditionMessage(err), pair, fixed = TRUE)
  }
  expect_identical(x, c(1L, 2L))
})

test_that("instance methods use the same pre-conversion guard", {
  probe <- AliasGuardProbe$new()
  x <- c(1L, 2L)
  expect_error(probe$check(list(x), x), "aliasing")
  expect_identical(x, c(1L, 2L))
})

test_that("nested alias checks retain roots under gctorture", {
  skip_gc_stress_if_disabled()
  result <- tryCatch({
    gctorture(TRUE)
    miniextendr:::gc_stress_slice_alias_guard()
  }, finally = gctorture(FALSE))
  expect_null(result)
})

test_that("worker wrappers reject aliases before dispatch", {
  skip_if_not(miniextendr_has_feature("worker-thread") ||
                miniextendr_has_feature("worker-default"), "worker feature not enabled")
  x <- c(1L, 2L)
  expect_error(alias_worker_lists(list(x), x), "aliasing")
  expect_identical(x, c(1L, 2L))
})

test_that("raw wrapper preflight leaves wrong native types to conversion", {
  result <- .Call(miniextendr:::C_miniextendr_alias_list_mut_direct_shared,
                  NULL, list("bad"), "bad")
  expect_identical(result$kind, "conversion")
  expect_match(result$error, "convert parameter")
})


test_that("RNG cleanup keeps alias conditions alive with a shared seed", {
  skip_gc_stress_if_disabled()
  withr::local_preserve_seed()
  functions <- "alias_rng_lists"
  if (miniextendr_has_feature("worker-thread") ||
      miniextendr_has_feature("worker-default")) {
    functions <- c(functions, "alias_worker_rng_lists")
  }
  for (fn in functions) {
    set.seed(1252)
    saved_seed <- .Random.seed
    x <- c(1L, 2L)
    err <- tryCatch({
      gctorture(TRUE)
      get(fn)(list(x), x)
    }, error = identity, finally = gctorture(FALSE))
    expect_s3_class(err, "error")
    expect_match(conditionMessage(err), "aliasing")
    expect_identical(x, c(1L, 2L))
    expect_identical(.Random.seed, saved_seed)
  }
})


test_that("native scalar aliases report every pair before mutation", {
  inputs <- list(integer = 2L, real = 2.5, raw = as.raw(2L),
                 logical = TRUE, complex = 2 + 3i)
  prefixes <- "alias_scalar_"
  if (miniextendr_has_feature("worker-thread") ||
      miniextendr_has_feature("worker-default")) {
    prefixes <- c(prefixes, "alias_worker_scalar_")
  }
  for (prefix in prefixes) {
    for (type in names(inputs)) {
      fn <- get(paste0(prefix, type))
      x <- inputs[[type]]
      saved <- serialize(x, NULL)
      err <- tryCatch(fn(x, x, x), error = identity)
      expect_s3_class(err, "error")
      for (pair in c("parameters `a` and `b`", "parameters `a` and `c`",
                     "parameters `b` and `c`")) {
        expect_match(conditionMessage(err), pair, fixed = TRUE)
      }
      expect_identical(serialize(x, NULL), saved)
      # Serialized copies are distinct R vectors with identical values.
      expect_identical(fn(unserialize(saved), unserialize(saved), x), 1L)
    }
  }
})

test_that("scalar, slice, list, boxed, and wrapped borrows share one guard", {
  x <- 7L
  for (fn in c("alias_scalar_slice", "alias_slice_scalar", "alias_wrapped_scalars")) {
    expect_error(get(fn)(x, x), "aliasing", info = fn)
  }
  expect_error(alias_scalar_lists(list(x), list(x)), "aliasing")
  expect_error(alias_scalar_lists(list(x, x), list()), "duplicate elements")
  expect_error(alias_boxed_scalars(list(NULL, x), list(x)), "aliasing")
  expect_error(alias_boxed_scalars(list(x, NULL, x), list()), "duplicate elements")
  expect_error(alias_boxed_slices(list(x), x), "aliasing")
  expect_error(alias_boxed_slices(list(x, x), 9L), "duplicate elements")
  expect_error(alias_nested_scalars(list(list(x)), x), "aliasing")
  expect_error(alias_nested_scalars(list(list(x), list(x)), 9L), "duplicate elements")
  expect_error(alias_newtype_scalars(list(x), x), "aliasing")
  expect_error(alias_newtype_scalars(list(x, NULL, x), 9L), "duplicate elements")
  expect_identical(x, 7L)
  expect_identical(alias_scalar_shared(x, x), 14L)
  expect_identical(alias_nested_scalars(NULL, x), 7L)
  expect_identical(alias_boxed_scalars(list(NULL, NULL), list(NULL)), 0L)
  expect_identical(alias_boxed_slices(list(integer(), integer()), x), 7L)
  a <- 3L
  b <- c(4L, 5L)
  expect_identical(alias_boxed_scalars(list(a), list(b)), 9L)
  expect_identical(a, 9L)
  if (miniextendr_has_feature("worker-thread") ||
      miniextendr_has_feature("worker-default")) {
    expect_error(alias_worker_boxed_scalars(list(x), list(x)), "aliasing")
    expect_error(alias_worker_boxed_scalars(list(x, NULL, x), list()), "duplicate elements")
    expect_identical(alias_worker_boxed_scalars(list(NULL), list(NULL)), 0L)
    expect_identical(x, 7L)
  }
})

test_that("scalar preflight requires length one but treats NA as borrowed storage", {
  for (x in list(integer(), c(1L, 2L), "wrong", NULL)) {
    result <- .Call(miniextendr:::C_miniextendr_alias_scalar_integer, NULL, x, x, x)
    expect_identical(result$kind, "conversion")
    expect_false(grepl("aliasing", result$error, fixed = TRUE))
    result <- .Call(miniextendr:::C_miniextendr_alias_boxed_scalars,
                    NULL, list(x), list(x))
    if (is.null(x)) {
      expect_identical(result, 0L)
    } else {
      expect_identical(result$kind, "conversion")
    }
  }
  for (case in list(list(fn = alias_scalar_integer, x = NA_integer_),
                    list(fn = alias_scalar_real, x = NA_real_),
                    list(fn = alias_scalar_logical, x = NA),
                    list(fn = alias_scalar_complex, x = NA_complex_))) {
    expect_error(case$fn(case$x, case$x, case$x), "aliasing")
  }
})
