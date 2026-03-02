# Tests for r_name, r_entry, r_post_checks, and r_on_exit attributes

test_that("r_name renames standalone function", {
  # is.widget should exist (renamed from is_widget)
  expect_true(is.widget(5L))
  expect_false(is.widget(-1L))
})

test_that("r_entry injects code before checks", {
  # r_entry = "x <- as.integer(x)" should coerce numeric to integer
  expect_equal(r_entry_demo(3L), 6L)
})

test_that("r_post_checks injects code after checks", {
  # r_post_checks = 'message("validated")' should emit a message
  expect_message(r_post_checks_demo(5L), "validated")
  expect_equal(suppressMessages(r_post_checks_demo(5L)), 6L)
})

test_that("combined r_name + r_entry + r_post_checks", {
  # widget.create renamed from create_widget
  # r_entry coerces n; r_post_checks validates n > 0
  expect_equal(widget.create(3L), 30L)
  expect_error(widget.create(-1L))
})

test_that("R6 method r_name renames method", {
  w <- WrapperDemo$new(10L)
  # increment() is renamed to add_one()
  w$add_one()
  expect_equal(w$get_value(), 11L)
})

test_that("R6 method r_entry injects code in method body", {
  w <- WrapperDemo$new(10L)
  # r_entry = "by <- as.integer(by)" coerces the argument
  w$add_by(5L)
  expect_equal(w$get_value(), 15L)
})

# ── r_on_exit tests ──

test_that("r_on_exit short form runs on.exit cleanup on normal exit", {
  # on_exit_short has r_on_exit = "message('cleanup ran')"
  expect_message(on_exit_short(5L), "cleanup ran")
  expect_equal(suppressMessages(on_exit_short(5L)), 6L)
})

test_that("r_on_exit with add = false generates on.exit without add arg", {
  # on_exit_no_add has r_on_exit(expr = "message('cleanup overwrite')", add = false)
  expect_message(on_exit_no_add(5L), "cleanup overwrite")
  expect_equal(suppressMessages(on_exit_no_add(5L)), 7L)
})

test_that("r_on_exit with after = false generates on.exit with after = FALSE", {
  # on_exit_lifo has r_on_exit(expr = "message('cleanup lifo')", after = false)
  expect_message(on_exit_lifo(5L), "cleanup lifo")
  expect_equal(suppressMessages(on_exit_lifo(5L)), 8L)
})

test_that("R6 method r_on_exit injects on.exit in method body", {
  w <- WrapperDemo$new(10L)
  # get_value() has r_on_exit = "message('method cleanup')"
  expect_message(w$get_value(), "method cleanup")
  expect_equal(suppressMessages(w$get_value()), 10L)
})
