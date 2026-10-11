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

test_that("flat env trait lookup refuses ambiguity and preserves explicit lookup", {
  lookup <- getFromNamespace(".miniextendr_env_method", "miniextendr")
  methods <- new.env(parent = emptyenv())
  methods$Alpha <- new.env(parent = emptyenv())
  methods$Beta <- new.env(parent = emptyenv())
  methods$Alpha$value <- structure(function(x) list("Alpha", x), .__mx_instance__ = TRUE)
  methods$Beta$value <- structure(function(x) list("Beta", x), .__mx_instance__ = TRUE)
  methods$Alpha$limit <- function() 10L
  methods$Beta$limit <- function() 20L
  # An inherited function and a non-function member are not candidates.
  methods$Inherited <- new.env(parent = methods$Alpha)
  methods$Data <- list2env(list(value = 42L), parent = emptyenv())
  `$.AuditEnv` <- function(self, name) lookup(self, name, methods)
  receiver <- structure(list(), class = "AuditEnv")

  error <- tryCatch(receiver$value, error = identity)
  expect_s3_class(error, "miniextendr_ambiguous_method")
  expect_identical(error$method, "value")
  expect_identical(error$traits, c("Alpha", "Beta"))
  expect_match(conditionMessage(error), "obj$Trait", fixed = TRUE)
  expect_identical(conditionCall(error), quote(receiver$value))
  expect_error(receiver$limit, class = "miniextendr_ambiguous_method")
  expect_identical(receiver$Alpha$value(), list("Alpha", receiver))
  expect_identical(receiver$Beta$value(), list("Beta", receiver))
  expect_identical(receiver$Alpha$limit(), 10L)
  expect_identical(receiver$Beta$limit(), 20L)

  # An inherent method wins over both traits.
  methods$value <- function() "inherent"
  expect_identical(receiver$value(), "inherent")
  # With one candidate left, both instance and static flat lookup still work.
  methods$value <- NULL
  methods$Beta$value <- NULL
  methods$Beta$limit <- NULL
  expect_identical(receiver$value(), list("Alpha", receiver))
  expect_identical(receiver$limit(), 10L)
  expect_null(receiver$missing)
})

test_that("generated env methods close over the namespace and bind only self", {
  a <- SimpleCounter$new_counter(10L)
  bound <- a$get_value
  expect_identical(ls(environment(bound), all.names = TRUE), "self")
  expect_identical(environment(bound)$self, a)
  expect_identical(parent.env(environment(bound)), asNamespace("miniextendr"))
  expect_identical(bound(), 10L)
})

# Fixture: src/rust/env_method_lookup_tests.rs. LookupLeft and LookupRight
# both provide `side`, `origin` and `LIMIT`; `label` is also inherent.
test_that("generated env trait members shared by two traits need the trait named", {
  LookupHost <- miniextendr:::LookupHost
  h <- LookupHost$new(5L)

  error <- tryCatch(h$side, error = identity)
  expect_s3_class(error, "miniextendr_ambiguous_method")
  expect_identical(error$method, "side")
  expect_identical(error$traits, c("LookupLeft", "LookupRight"))
  expect_identical(conditionCall(error), quote(h$side))
  error <- tryCatch(h[["side"]], error = identity)
  expect_s3_class(error, "miniextendr_ambiguous_method")
  expect_identical(conditionCall(error), quote(h[["side"]]))
  expect_error(h$origin(), class = "miniextendr_ambiguous_method")
  expect_error(h$LIMIT(), class = "miniextendr_ambiguous_method")

  expect_identical(h$LookupLeft$side(), "left 5")
  expect_identical(h$LookupRight$side(), "right 5")
  expect_identical(LookupHost$LookupLeft$side(h), "left 5")
  expect_identical(LookupHost$LookupRight$side(h), "right 5")
  expect_identical(h$LookupLeft$origin(), "left")
  expect_identical(h$LookupRight$origin(), "right")
  expect_identical(h$LookupLeft$LIMIT(), 10L)
  expect_identical(h$LookupRight$LIMIT(), 20L)

  expect_identical(h$label(), "inherent")
  expect_identical(h$LookupLeft$label(), "left label")
  expect_identical(h$LookupRight$label(), "right label")

  expect_identical(h$left_only(), 6L)
  expect_identical(h[["left_only"]](), 6L)
  expect_null(h$no_such_member)
})
