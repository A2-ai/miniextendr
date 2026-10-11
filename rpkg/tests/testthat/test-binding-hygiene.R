# Parameters named like the bindings that generated wrappers use internally
# (#1677). Fixture: src/rust/binding_hygiene_tests.rs. Before the fix each of
# these compiled and bound the wrong value.

HygieneHost <- miniextendr:::HygieneHost

test_that("a parameter named arg_1 leaves the next argument's slot alone", {
  h <- HygieneHost$new(1L)
  expect_identical(h$second("first", "second"), "second")
  expect_identical(h$second(arg_1 = 1L, value = 2L), 2L)
})

test_that("a parameter named self_sexp leaves the returned receiver alone", {
  h <- HygieneHost$new(1L)
  # The R wrapper returns its own receiver, so call the C wrapper directly to
  # see which SEXP it hands back.
  out <- .Call(miniextendr:::C_miniextendr_HygieneHost__store, NULL, h, 42L)
  expect_identical(out, h)
  expect_identical(h$value(), 42L)
  expect_identical(h$store(5L), h)
  expect_identical(h$value(), 5L)
})

test_that("a trait parameter named self_ref is not the receiver", {
  a <- HygieneHost$new(3L)
  b <- HygieneHost$new(10L)
  expect_identical(a$ShimNames$pair(b), c(3L, 10L))
  expect_identical(HygieneHost$ShimNames$pair(b, self_ref = a), c(10L, 3L))
})

test_that("trait parameters named like shim bindings keep their values", {
  h <- HygieneHost$new(7L)
  # Through the trait's C wrapper ...
  expect_identical(
    h$ShimNames$shim_args(argc = 1L, data = 10L, argv = 100L, self_ref = 1000L),
    c(7L, 1L, 10L, 100L, 1000L)
  )
  # ... and through the View and the vtable shim, where `argc` used to
  # replace the shim's argument count.
  expect_identical(
    miniextendr:::shim_names_through_view(),
    c(7L, 1L, 10L, 100L, 1000L)
  )
})
