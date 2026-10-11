test_that("retained env methods bind independent receivers and keep them alive", {
  methods <- local({
    a <- SimpleCounter$new_counter(10L)
    b <- SimpleCounter$new_counter(100L)
    list(
      get_a = a$get_value, get_b = b$get_value,
      flat_a = a$value, flat_b = b$value,
      trait_a = a$Counter, trait_b = b$Counter
    )
  })
  gc(full = TRUE)
  methods$trait_a$increment()
  methods$trait_b$checked_add(7L)
  expect_identical(methods$get_a(), 11L)
  expect_identical(methods$get_b(), 107L)
  expect_identical(methods$flat_a(), 11L)
  expect_identical(methods$flat_b(), 107L)
  expect_identical(methods$trait_a$value(), 11L)
  expect_identical(methods$trait_b$value(), 107L)
  expect_identical(methods$trait_a$default_initial(), 0L)
  expect_identical(methods$trait_b$MAX_VALUE(), .Machine$integer.max)
  expect_identical(environment(SimpleCounter$get_value), asNamespace("miniextendr"))
})

test_that("env method lookup preserves statics, indexing and missing names", {
  a <- SimpleCounter$new_counter(10L)
  b <- PanickyCounter$new_panicky(100L)
  expect_identical(a[["get_value"]](), 10L)
  expect_identical(a$default_initial(), 0L)
  expect_identical(b$default_initial(), 100L)
  expect_identical(a$Counter$MAX_VALUE(), .Machine$integer.max)
  expect_identical(b$Counter$MAX_VALUE(), 1000L)
  expect_null(a$no_such_method)
  expect_null(a[["no_such_method"]])
})

test_that("env method bodies and defaults keep their lexical environment", {
  lookup <- getFromNamespace(".miniextendr_env_method", "miniextendr")
  lexical <- list2env(
    list(name = "package name", obj = "package object", methods = "package methods"),
    parent = baseenv()
  )
  methods <- new.env(parent = emptyenv())
  methods$read <- eval(quote(function(value = name) {
    list(receiver = self, value = value, obj = obj, methods = methods)
  }), lexical)
  receiver <- new.env(parent = emptyenv())

  bound <- lookup(receiver, "read", methods)
  expect_identical(bound(), list(
    receiver = receiver, value = "package name",
    obj = "package object", methods = "package methods"
  ))
  expect_identical(bound("explicit")$value, "explicit")
  expect_identical(environment(methods$read), lexical)
  expect_identical(parent.env(environment(bound)), lexical)
})
