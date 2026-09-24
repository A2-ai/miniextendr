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

test_that("base operators are not re-exported or documented as package generics", {
  exports <- getNamespaceExports("miniextendr")
  expect_false("[" %in% exports)
  expect_false("[[" %in% exports)
  expect_false("%*%" %in% exports)
})

test_that("s7(generic = \"%mx_cat%\") defines a new operator and keeps its shortcut", {
  bag <- MxS7Bag(c(1, 2))
  expect_equal(bag %mx_cat% c(3, 4), c(1, 2, 3, 4))
  expect_true(inherits(miniextendr::`%mx_cat%`, "S7_generic"))
  expect_true("%mx_cat%" %in% getNamespaceExports("miniextendr"))
  expect_equal(MxS7Bag_concat(bag, 5), c(1, 2, 5))
})

test_that("`%*%` dispatches on both operands", {
  a <- MxS7Bag(c(1, 2, 3))
  b <- MxS7Bag(c(4, 5, 6))
  expect_equal(a %*% b, 32)
  expect_error(a %*% MxS7Bag(1), "cannot multiply bags of 3 and 1 values")
  expect_equal(MxS7Bag_dot(a, b), 32)
})

test_that("s7(generic = \"generics::tidy\") registers when generics loads", {
  skip_if_not_installed("generics")
  # A fresh session: library(miniextendr) does not load generics, so the
  # method can only reach tidy() through S7's onLoad hook.
  res <- run_isolated({
    before <- isNamespaceLoaded("generics")
    out <- generics::tidy(MxS7Bag(c(10, 20)))
    list(before = before, out = out)
  })
  expect_false(res$before)
  expect_s3_class(res$out, "data.frame")
  expect_equal(res$out$position, 1:2)
  expect_equal(res$out$value, c(10, 20))

  out <- generics::tidy(MxS7Bag(5))
  expect_equal(out$value, 5)
  expect_equal(MxS7Bag_tidy(MxS7Bag(7))$value, 7)
  # The generic belongs to generics: it is neither exported nor documented here.
  expect_false("tidy" %in% getNamespaceExports("miniextendr"))
})
