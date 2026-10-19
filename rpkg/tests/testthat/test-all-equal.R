# `#[derive(AllEqual)]`: the `all.equal()` method of a pointer-backed class
# (#2149). Base R's `all.equal()` compares two such objects only by their
# `data.class()`, so it returned `TRUE` whatever they held. The derived method
# compares the struct field by field, each field with base `all.equal()` on
# its R value, and prefixes each field's messages as `all.equal.list()` does
# (or with the field's `label`).
#
# Fixtures: `AllEqS3`, `AllEqEnv`, `AllEqR6`, `AllEqS4` and `AllEqS7` (one per
# class system, each holding `values`), `AllEqFit` (field options, a nested
# `AllEqModel`, the struct's tolerance), `AllEqHolder` (`AllEqPart`s by value,
# as an optional pointer and in a `Vec`) and `AllEqSidecar` (a `Sidecar<T>`
# and a `Computed` field) in `src/rust/all_equal_tests.rs`.

# The message `all.equal.list()` would give for a differing field `name`.
component <- function(name, target, current, ...) {
  paste0("Component ", dQuote(name), ": ", all.equal(target, current, ...))
}

constructors <- list(
  S3 = new_alleqs3,
  env = AllEqEnv$new,
  R6 = AllEqR6$new,
  S4 = AllEqS4,
  S7 = AllEqS7
)

# region: every class system

for (system in names(constructors)) {
  new_obj <- constructors[[system]]

  test_that(paste0(system, ": equal values are equal, different values are reported"), {
    a <- new_obj(c(1, 2, 3))
    expect_true(all.equal(a, new_obj(c(1, 2, 3))))
    expect_identical(
      all.equal(a, new_obj(c(1, 2, 4))),
      component("values", c(1, 2, 3), c(1, 2, 4))
    )
  })

  test_that(paste0(system, ": the comparison keeps the tolerance"), {
    a <- new_obj(c(1, 2, 3))
    b <- new_obj(c(1, 2, 3 + 1e-10))
    expect_true(all.equal(a, b))
    expect_identical(
      all.equal(a, b, tolerance = 0),
      component("values", c(1, 2, 3), c(1, 2, 3 + 1e-10), tolerance = 0)
    )
  })

  test_that(paste0(system, ": an object of another class is reported, not read"), {
    a <- new_obj(c(1, 2, 3))
    expect_identical(
      all.equal(a, c(1, 2, 3)),
      sprintf("target is %s, current is numeric", data.class(a))
    )
  })

  test_that(paste0(system, ": a restored object is refused"), {
    a <- new_obj(c(1, 2, 3))
    restored <- unserialize(serialize(a, NULL))
    expect_error(all.equal(restored, a), class = "miniextendr_restored_no_value")
    expect_error(all.equal(a, restored), class = "miniextendr_restored_no_value")
  })

  test_that(paste0(system, ": testthat edition 2's expect_equal() uses the method"), {
    local_edition(2)
    a <- new_obj(c(1, 2, 3))
    expect_success(expect_equal(a, new_obj(c(1, 2, 3))))
    expect_failure(expect_equal(a, new_obj(c(1, 2, 4))), "Mean relative difference")
  })

  # Edition 3's expect_equal() compares with waldo::compare(), which never
  # calls all.equal(). It sees the R object, not the Rust value: the pointer
  # of an S3, env, S4 or S7 object, compared by address, so two objects with
  # equal values differ; an R6 object's public fields, which hold no value
  # (the pointer is private), so two R6 objects are equal whatever they hold.
  test_that(paste0(system, ": testthat edition 3's expect_equal() does not use the method"), {
    local_edition(3)
    a <- new_obj(c(1, 2, 3))
    if (system == "R6") {
      expect_success(expect_equal(a, new_obj(c(1, 2, 4))))
    } else {
      expect_failure(expect_equal(a, new_obj(c(1, 2, 3))))
    }
  })
}

# endregion

# region: the field options

new_fit <- function(estimates = c(1, 2), iterations = 10, weights = c(0.5, 0.5),
                    name = "Fit", scratch = 1) {
  new_alleqfit(estimates, iterations, weights, name, scratch)
}

test_that("a skipped field is not compared", {
  other <- new_fit(scratch = c(9, 9, 9))
  expect_identical(scratch_len(other), 3L)
  expect_true(all.equal(new_fit(), other))
})

test_that("a `with` getter's value is compared", {
  expect_true(all.equal(new_fit(name = "Fit"), new_fit(name = "FIT")))
  expect_identical(
    all.equal(new_fit(name = "Fit"), new_fit(name = "Other")),
    component("name", "fit", "other")
  )
})

test_that("a labelled field's messages start with its label", {
  expect_identical(
    all.equal(new_fit(estimates = c(1, 2)), new_fit(estimates = c(1, 3))),
    paste0("Estimates: ", all.equal(c(1, 2), c(1, 3)))
  )
})

test_that("the struct's tolerance is the default, which the call replaces", {
  close <- new_fit(estimates = c(1, 2 + 1e-8))
  # 5e-9 is within the struct's 1e-6, not within R's default 1.5e-8 ...
  expect_true(all.equal(new_fit(), close))
  expect_false(isTRUE(all.equal(c(1, 2), c(1, 2 + 1e-8), tolerance = 1e-10)))
  # ... and the call's tolerance replaces the struct's.
  expect_identical(
    all.equal(new_fit(), close, tolerance = 1e-10),
    paste0("Estimates: ", all.equal(c(1, 2), c(1, 2 + 1e-8), tolerance = 1e-10))
  )
  # A third positional argument is the tolerance, as for numbers.
  expect_identical(
    all.equal(new_fit(), close, 1e-10),
    all.equal(new_fit(), close, tolerance = 1e-10)
  )
})

test_that("a field's own tolerance ignores the call's", {
  expected <- component("iterations", 10, 10 + 1e-9, tolerance = 0)
  expect_identical(all.equal(new_fit(), new_fit(iterations = 10 + 1e-9)), expected)
  expect_identical(
    all.equal(new_fit(), new_fit(iterations = 10 + 1e-9), tolerance = 1),
    expected
  )
})

test_that("a nested class is compared by its own method, with both labels", {
  expect_true(all.equal(new_fit(), new_fit()))
  expect_identical(
    all.equal(new_fit(), new_fit(weights = c(0.5, 0.6))),
    paste0(
      "Component ", dQuote("model"), ": ",
      component("weights", c(0.5, 0.5), c(0.5, 0.6), tolerance = 1e-6)
    )
  )
})

test_that("a class held by value, as an optional pointer and in a Vec uses its own method", {
  new_holder <- function(first = 1, spare = 2, rest = c(3, 4)) {
    new_alleqholder(first, spare, rest)
  }
  weight <- function(target, current) component("weight", target, current)
  expect_true(all.equal(new_holder(), new_holder()))
  expect_true(all.equal(new_holder(spare = NULL), new_holder(spare = NULL)))
  expect_identical(
    all.equal(new_holder(), new_holder(first = 5)),
    paste0("Component ", dQuote("first"), ": ", weight(1, 5))
  )
  expect_identical(
    all.equal(new_holder(), new_holder(spare = 3)),
    paste0("Component ", dQuote("spare"), ": ", weight(2, 3))
  )
  expect_identical(
    all.equal(new_holder(), new_holder(spare = NULL)),
    paste0("Component ", dQuote("spare"), ": target is AllEqPart, current is NULL")
  )
  expect_identical(
    all.equal(new_holder(), new_holder(rest = c(3, 5))),
    paste0("Component ", dQuote("rest"), ": Component 2: ", weight(4, 5))
  )
})

test_that("every differing field is reported, in field order", {
  result <- all.equal(
    new_fit(),
    new_fit(estimates = c(1, 3), iterations = 11, weights = c(0.5, 0.6), name = "x")
  )
  expect_length(result, 4L)
  expect_true(all(startsWith(
    result,
    c("Estimates: ", paste0("Component ", dQuote(c("iterations", "model", "name")), ": "))
  )))
})

test_that("a restored object is refused with the call as written", {
  a <- new_fit()
  restored <- unserialize(serialize(a, NULL))
  err <- expect_error(all.equal(restored, a), class = "miniextendr_restored_no_value")
  expect_identical(conditionCall(err), quote(all.equal(restored, a)))
})

# endregion

# region: sidecar fields

test_that("a Sidecar field reads its slot, a Computed field its getter", {
  s <- new_alleqsidecar(1:3, c(1L, 2L))
  # `values` is skipped; its total is compared.
  expect_true(all.equal(s, new_alleqsidecar(1:3, 3L)))
  expect_identical(
    all.equal(s, new_alleqsidecar(1:4, 3L)),
    component("keys", 1:3, 1:4)
  )
  expect_identical(
    all.equal(s, new_alleqsidecar(1:3, 4L)),
    component("total", 3L, 4L)
  )
})

test_that("two classes with derived methods are reported as different classes", {
  expect_identical(
    all.equal(new_fit(), new_alleqsidecar(1:3, 3L)),
    "target is AllEqFit, current is AllEqSidecar"
  )
})

# endregion
