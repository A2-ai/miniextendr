# S7 methods on operator generics (#1475). Fixtures:
# src/rust/s7_nonsyntactic_tests.rs. The generated wrappers only parse (and the
# package only installs) when these names are backtick-quoted.

test_that("s7(generic = \"[\") dispatches `bag[i]` and keeps its shortcut", {
  bag <- MxS7Bag(c(10, 20, 30))
  expect_equal(bag[2:3], c(20, 30))
  expect_equal(bag[c(1L, 3L)], c(10, 30))
  expect_error(bag[5L], "out of range")
  # The Rust name `subset` is syntactic, so the fast-path shortcut exists.
  expect_true(is.function(MxS7Bag_subset))
  expect_equal(MxS7Bag_subset(bag, 1L), 10)
})

test_that("r_name = \"[[\" dispatches `bag[[i]]` and emits no `MxS7Bag_[[` shortcut", {
  bag <- MxS7Bag(c(10, 20, 30))
  expect_equal(bag[[2L]], 20)
  expect_error(bag[[4L]], "out of range")
  ns <- asNamespace("miniextendr")
  expect_false(exists("MxS7Bag_[[", envir = ns, inherits = FALSE))
})

test_that("r_name = \"%mx_scale%\" defines and exports a package-local operator", {
  bag <- MxS7Bag(c(1, 2, 3))
  expect_equal(bag %mx_scale% 2, c(2, 4, 6))
  expect_true(inherits(miniextendr::`%mx_scale%`, "S7_generic"))
  expect_true("%mx_scale%" %in% getNamespaceExports("miniextendr"))
  expect_false(exists("MxS7Bag_%mx_scale%", envir = asNamespace("miniextendr"), inherits = FALSE))
})
