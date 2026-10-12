# Receiver formals of generated wrappers (#1677). Fixture:
# src/rust/receiver_formals_tests.rs. Env methods, Env/R6 trait namespace
# functions and S7 shortcuts bind their receiver as `.mx_self`, so arguments
# named `self`, `x`, `private` and `obj` reach the method. S3/S4 methods on another
# package's generic take that generic's receiver formal and attach to it.

FormalsEnv <- miniextendr:::FormalsEnv
FormalsR6 <- miniextendr:::FormalsR6

test_that("env methods take arguments named self, x and obj", {
  e <- FormalsEnv$new(1L)
  expect_identical(e$echo(self = 2L, x = 3L, obj = 4L), 1:4)
  expect_identical(e$echo(obj = 4L, x = 3L, self = 2L), 1:4)
  # A void method still returns the receiver, not the `self` argument.
  expect_identical(e$add(self = 4L), e)
  expect_identical(e$echo(0L, 0L, 0L), c(5L, 0L, 0L, 0L))
})

test_that("env trait methods take arguments named self, x, private and obj", {
  e <- FormalsEnv$new(1L)
  expect_identical(
    e$FormalNames$echo_formals(self = 2L, x = 3L, private = 4L, obj = 5L),
    1:5
  )
  expect_identical(e$echo_formals(obj = 5L, private = 4L, x = 3L, self = 2L), 1:5)
  expect_identical(
    FormalsEnv$FormalNames$echo_formals(e, self = 2L, x = 3L, private = 4L, obj = 5L),
    1:5
  )
  expect_identical(e$FormalNames$bump(x = 10L), e)
  expect_identical(e$echo(0L, 0L, 0L), c(11L, 0L, 0L, 0L))
})

test_that("R6 trait methods take arguments named self, x, private and obj", {
  r <- FormalsR6$new(1L)
  expect_identical(
    FormalsR6$FormalNames$echo_formals(r, self = 2L, x = 3L, private = 4L, obj = 5L),
    1:5
  )
  expect_identical(FormalsR6$FormalNames$bump(r, x = 5L), r)
  expect_identical(
    FormalsR6$FormalNames$echo_formals(r, 0L, 0L, 0L, 0L),
    c(6L, 0L, 0L, 0L, 0L)
  )
})

test_that("S7 shortcuts take an argument named self", {
  s <- FormalsS7(1L)
  expect_identical(FormalsS7_formals_echo_self(s, self = 2L), c(1L, 2L))
  expect_identical(formals_echo_self(s, self = 2L), c(1L, 2L))
  expect_identical(FormalsS7_formals_add_self(s, self = 3L), s)
  expect_identical(formals_value(s), 4L)
  expect_identical(FormalsS7_shortcut_self(s, self = 5L), c(4L, 5L))
  expect_identical(s7_trait_ShortcutSelf_shortcut_self(s, self = 5L), c(4L, 5L))
})

test_that("S3 methods take the receiver formal of an existing generic", {
  o <- FormalsS3$new(3L)
  expect_identical(
    names(formals(getS3method("all.equal", "FormalsS3"))),
    c("target", "current", "...")
  )
  expect_true(all.equal(o, 3L))
  expect_false(all.equal(target = o, current = 4L))
  expect_identical(names(formals(getS3method("summary", "FormalsS3"))), c("object", "..."))
  expect_identical(summary(object = o), 1L)
  expect_identical(summary(o), 2L)
  # Registered, not only found from inside the namespace: a caller whose
  # environment sees base alone still dispatches to both methods.
  outside <- list2env(list(o = o), parent = baseenv())
  expect_identical(eval(quote(summary(o)), outside), 3L)
  expect_true(eval(quote(all.equal(o, 3L)), outside))
})

test_that("S4 methods attach to show, length and an explicitly named generic", {
  ns <- asNamespace("miniextendr")
  # Neither generic is redefined in the package (#1677).
  expect_false(exists("show", envir = ns, inherits = FALSE))
  expect_false(exists("length", envir = ns, inherits = FALSE))
  expect_identical(methods::getGeneric("show")@package, "methods")

  m <- FormalsS4()
  expect_identical(names(formals(methods::getMethod("show", "FormalsS4"))), "object")
  expect_identical(s4_shown(m), 0L)
  expect_invisible(methods::show(m))
  expect_identical(s4_shown(m), 1L)
  print(m)
  expect_identical(s4_shown(m), 2L)

  expect_identical(length(m), 42L)

  # A generic no other package defines is still created under its name.
  expect_true(exists("formals_fresh_generic", envir = ns, inherits = FALSE))
  expect_identical(formals_fresh_generic(m), 7L)
})

test_that("S4 trait methods attach to show", {
  expect_false(exists("show", envir = asNamespace("miniextendr"), inherits = FALSE))
  t <- FormalsS4Trait()
  expect_identical(names(formals(methods::getMethod("show", "FormalsS4Trait"))), "object")
  methods::show(t)
  expect_identical(s4_reports(t), 1L)
})
