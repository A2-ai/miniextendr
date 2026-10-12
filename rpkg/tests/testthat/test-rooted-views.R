# Borrowed StrVec / RCow parameters bind to a root that lives for the call
# (#1427). Fixture: src/rust/rooted_view_tests.rs.

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

test_that("inherent methods bind borrowed views to their call", {
  host <- RootedViewHost$new()
  expect_identical(host$first(c("method", "later")), "method")
  x <- c(4L, 5L)
  expect_identical(host$echo(x), x)
  expect_error(host$first(1L))
  expect_error(host$echo(1.5))
})

test_that("trait methods bind borrowed views in the C wrapper and the vtable shim", {
  host <- RootedViewHost$new()
  expect_identical(host$RootedViews$view_first(c("trait", "later")), "trait")
  expect_identical(host$RootedViews$view_values(c(7L, NA_integer_)), c(7L, NA_integer_))
  expect_error(host$RootedViews$view_first(1L))
  expect_error(host$RootedViews$view_values("wrong storage"))
  expect_identical(miniextendr:::rooted_views_through_view(), c("through view", "1,2,3"))
})
