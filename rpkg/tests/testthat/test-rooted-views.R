# Borrowed StrVec / RCow parameters, and Cow and shared-reference
# parameters, bind to a root that lives for the call (#1427). Fixture:
# src/rust/rooted_view_tests.rs.

RootedViewHost <- miniextendr:::RootedViewHost

test_that("free functions read borrowed views and refuse other storage", {
  expect_identical(miniextendr:::rooted_strvec_first(c("rooted", "later")), "rooted")
  expect_identical(miniextendr:::rooted_strvec_first(NA_character_), NA_character_)
  expect_identical(miniextendr:::rooted_strvec_first(character()), NA_character_)
  x <- c(1L, NA_integer_, 3L)
  expect_identical(miniextendr:::rooted_rcow_identity(x), x)
  expect_error(miniextendr:::rooted_strvec_first(1L))
  expect_error(miniextendr:::rooted_rcow_identity("wrong storage"))
})

test_that("free functions read borrowed Cow and shared-reference parameters", {
  describe <- miniextendr:::rooted_refs_describe
  expect_identical(
    describe(c(1L, 2L), 0.5, "lbl", "txt", c("a", NA, "c")),
    "lbl:1,2:txt:0.5:a NA c"
  )
  expect_identical(describe(integer(), 2, NULL, "", character()), "NA:::2:")
  expect_error(describe(1.5, 0.5, "lbl", "txt", "a"))
  expect_error(describe(1L, c(1, 2), "lbl", "txt", "a"))
  expect_error(describe(1L, 0.5, 1L, "txt", "a"))
  expect_error(describe(1L, 0.5, "lbl", c("x", "y"), "a"))
  expect_error(describe(1L, 0.5, "lbl", "txt", 1L))
})

test_that("inherent methods bind borrowed views to their call", {
  host <- RootedViewHost$new()
  expect_identical(host$first(c("method", "later")), "method")
  x <- c(4L, 5L)
  expect_identical(host$echo(x), x)
  expect_error(host$first(1L))
  expect_error(host$echo(1.5))
  expect_identical(host$describe(c(1L, 2L), "lbl", "txt"), "lbl:1,2:txt")
  expect_identical(host$describe(integer(), NULL, "txt"), "NA::txt")
  expect_error(host$describe(1.5, "lbl", "txt"))
  expect_error(host$describe(1L, "lbl", 1L))
})

test_that("trait methods bind borrowed views in the C wrapper and the vtable shim", {
  host <- RootedViewHost$new()
  expect_identical(host$RootedViews$view_first(c("trait", "later")), "trait")
  expect_identical(host$RootedViews$view_values(c(7L, NA_integer_)), c(7L, NA_integer_))
  expect_error(host$RootedViews$view_first(1L))
  expect_error(host$RootedViews$view_values("wrong storage"))
  expect_identical(host$RootedViews$view_describe(3L, NULL, "trait"), "NA:3:trait")
  expect_error(host$RootedViews$view_describe("3", NULL, "trait"))
  expect_identical(
    miniextendr:::rooted_views_through_view(),
    c("through view", "1,2,3", "NA:4,5:shim")
  )
})

test_that("uncached ALTSTRING elements survive generated argument conversion", {
  x <- miniextendr:::rooted_uncached_strings()
  expect_true(miniextendr:::rooted_uncached_strings_check(x, x, x, x, x))
  expect_null(miniextendr:::gc_stress_uncached_string_borrows())
})

test_that("uncached ALTSTRING borrows survive GC torture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  for (i in seq_len(3L)) {
    x <- miniextendr:::rooted_uncached_strings()
    expect_true(miniextendr:::rooted_uncached_strings_check(x, x, x, x, x))
    expect_null(miniextendr:::gc_stress_uncached_string_borrows())
  }
})
